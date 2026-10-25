use std::time::Duration;

use sabai_core::{Error, Result};

use crate::config::Target;
use crate::row::DriverRow;
use crate::{Driver, Row, SqlValue};

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// How to reach a database, checked at boot. Each driver is its own Cargo feature, so an app
/// compiles only the databases it uses.
// Built once per connection at boot, so a smaller `Missing` variant costs nothing worth boxing.
#[allow(clippy::large_enum_variant)]
pub(crate) enum Options {
    // Like a Laravel connection whose PDO driver is not installed: listing it in config is fine,
    // using it is the error.
    #[cfg(not(all(feature = "mysql", feature = "sqlite")))]
    Missing(&'static str),
    #[cfg(feature = "mysql")]
    MySql(sqlx::mysql::MySqlConnectOptions),
    #[cfg(feature = "sqlite")]
    Sqlite {
        options: sqlx::sqlite::SqliteConnectOptions,
        in_memory: bool,
    },
}

/// An open pool for one of the drivers the app was built with.
pub(crate) enum Pool {
    #[cfg(feature = "mysql")]
    MySql(sqlx::MySqlPool),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::SqlitePool),
}

// Both drivers bind the same values; the macro writes the loop once for each query type.
macro_rules! bind {
    ($query:expr, $values:expr) => {{
        let mut query = $query;
        for value in $values {
            query = match value {
                SqlValue::Null => query.bind(None::<i64>),
                SqlValue::Integer(value) => query.bind(value),
                SqlValue::Float(value) => query.bind(value),
                SqlValue::Bool(value) => query.bind(value),
                SqlValue::Text(value) => query.bind(value),
            };
        }
        query
    }};
}

impl Options {
    pub(crate) fn parse(target: &Target) -> Result<Self> {
        let invalid = |error: sqlx::Error| {
            Error::from(error).context("config/database.toml has an invalid connection")
        };
        match target.driver {
            #[cfg(feature = "mysql")]
            Driver::MySql => {
                let options: sqlx::mysql::MySqlConnectOptions =
                    target.url.parse().map_err(invalid)?;
                // `DateTime` is UTC, and a `timestamp` column converts to the session's zone.
                Ok(Self::MySql(options.timezone(Some("+00:00".to_owned()))))
            }
            #[cfg(not(feature = "mysql"))]
            Driver::MySql => Ok(Self::Missing("mysql")),
            #[cfg(feature = "sqlite")]
            Driver::Sqlite => Ok(Self::Sqlite {
                options: target.url.parse().map_err(invalid)?,
                in_memory: target.url == crate::config::SQLITE_MEMORY,
            }),
            #[cfg(not(feature = "sqlite"))]
            Driver::Sqlite => Ok(Self::Missing("sqlite")),
        }
    }

    // Opens nothing yet: the first query connects, once the tokio runtime runs.
    pub(crate) fn lazy_pool(&self) -> Result<Pool> {
        match *self {
            #[cfg(not(all(feature = "mysql", feature = "sqlite")))]
            Self::Missing(feature) => Err(missing_feature(feature)),
            #[cfg(feature = "mysql")]
            Self::MySql(ref options) => Ok(Pool::MySql(
                sqlx::mysql::MySqlPoolOptions::new()
                    .acquire_timeout(ACQUIRE_TIMEOUT)
                    .connect_lazy_with(options.clone()),
            )),
            #[cfg(feature = "sqlite")]
            Self::Sqlite {
                ref options,
                in_memory,
            } => {
                let pool = match in_memory {
                    // Every `:memory:` connection is a separate database.
                    true => sqlx::sqlite::SqlitePoolOptions::new()
                        .max_connections(1)
                        .idle_timeout(None)
                        .max_lifetime(None),
                    false => {
                        sqlx::sqlite::SqlitePoolOptions::new().acquire_timeout(ACQUIRE_TIMEOUT)
                    }
                };
                Ok(Pool::Sqlite(pool.connect_lazy_with(options.clone())))
            }
        }
    }
}

impl Pool {
    /// Runs one statement with `values` bound in order; returns the id an insert gave its new row,
    /// if the database reports one that fits a `u64`.
    pub(crate) async fn execute(&self, sql: &str, values: Vec<SqlValue>) -> Result<Option<u64>> {
        match *self {
            #[cfg(feature = "mysql")]
            Self::MySql(ref pool) => {
                let done = bind!(sqlx::query::<sqlx::MySql>(sql), values)
                    .execute(pool)
                    .await?;
                Ok(Some(done.last_insert_id()))
            }
            #[cfg(feature = "sqlite")]
            Self::Sqlite(ref pool) => {
                let done = bind!(sqlx::query::<sqlx::Sqlite>(sql), values)
                    .execute(pool)
                    .await?;
                Ok(u64::try_from(done.last_insert_rowid()).ok())
            }
        }
    }

    /// Every row `sql` returns, with `values` bound in order.
    pub(crate) async fn fetch_all(&self, sql: &str, values: Vec<SqlValue>) -> Result<Vec<Row>> {
        match *self {
            #[cfg(feature = "mysql")]
            Self::MySql(ref pool) => {
                let rows = bind!(sqlx::query::<sqlx::MySql>(sql), values)
                    .fetch_all(pool)
                    .await?;
                Ok(rows
                    .into_iter()
                    .map(|row| Row(DriverRow::MySql(row)))
                    .collect())
            }
            #[cfg(feature = "sqlite")]
            Self::Sqlite(ref pool) => {
                let rows = bind!(sqlx::query::<sqlx::Sqlite>(sql), values)
                    .fetch_all(pool)
                    .await?;
                Ok(rows
                    .into_iter()
                    .map(|row| Row(DriverRow::Sqlite(row)))
                    .collect())
            }
        }
    }
}

#[cfg(not(all(feature = "mysql", feature = "sqlite")))]
fn missing_feature(feature: &str) -> Error {
    Error::new(
        sabai_core::StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "config/database.toml uses {feature}, but Sabai was built without it; \
             add the `{feature}` feature: sabai = {{ ..., features = [\"orm\", \"{feature}\"] }}"
        ),
    )
}
