use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, Error, Result, StatusCode};
use sqlx::any::{AnyArguments, AnyRow};
use sqlx::query::Query;

use crate::{Database, Driver, Row, SqlValue};

/// A table row as a struct, like an Eloquent model. Write `#[derive(Model)]`, not this impl.
pub trait Model: Sized + Send + 'static {
    /// The table, such as `posts` for `Post`.
    const TABLE: &'static str;

    /// The columns, in field order.
    const COLUMNS: &'static [&'static str];

    /// Builds the model from one row.
    fn from_row(row: &Row) -> Result<Self>;

    /// The value of each column, in [`COLUMNS`](Model::COLUMNS) order.
    fn values(&self) -> Result<Vec<SqlValue>>;

    /// The primary key; `0` until the row is saved.
    fn id(&self) -> u64;

    /// Sets the primary key the database gave a new row.
    fn set_id(&mut self, id: u64);

    /// Every row of the table, like Laravel's `Post::all()`.
    fn all() -> BoxFuture<'static, Result<Vec<Self>>> {
        let rows = select(Self::TABLE, Self::COLUMNS, None);
        Box::pin(async move { rows.await?.iter().map(Self::from_row).collect() })
    }

    /// The row with this primary key, or `None`, like Laravel's `Post::find($id)`.
    fn find(id: u64) -> BoxFuture<'static, Result<Option<Self>>> {
        let rows = select(Self::TABLE, Self::COLUMNS, Some(id));
        Box::pin(async move { rows.await?.first().map(Self::from_row).transpose() })
    }

    /// The row with this primary key, or a `404`, like Laravel's `Post::findOrFail($id)`.
    fn find_or_fail(id: u64) -> BoxFuture<'static, Result<Self>> {
        let found = Self::find(id);
        Box::pin(async move { found.await?.ok_or_else(|| not_found(Self::TABLE, id)) })
    }

    /// Inserts the row when its id is `0` and fills the id in, or updates it, like `$post->save()`.
    fn save(&mut self) -> BoxFuture<'_, Result<()>> {
        let values = self.values();
        let id = self.id();
        Box::pin(async move {
            match id {
                0 => self.set_id(insert(Self::TABLE, Self::COLUMNS, values?).await?),
                id => update(Self::TABLE, Self::COLUMNS, values?, id).await?,
            }
            Ok(())
        })
    }
}

fn select(
    table: &'static str,
    columns: &'static [&'static str],
    id: Option<u64>,
) -> BoxFuture<'static, Result<Vec<Row>>> {
    Box::pin(async move {
        let database = current_database()?;
        let driver = database.driver();
        let columns: Vec<_> = columns.iter().map(|column| driver.quote(column)).collect();
        let mut sql = format!("select {} from {}", columns.join(", "), driver.quote(table));
        let mut values = Vec::new();
        if let Some(id) = id {
            sql.push_str(&format!(" where {} = ? limit 1", driver.quote("id")));
            values.push(SqlValue::Integer(i64::try_from(id)?));
        }
        let rows: Vec<AnyRow> = bind_all(sqlx::query(&sql), values)
            .fetch_all(database.pool())
            .await?;
        Ok(rows.into_iter().map(Row).collect())
    })
}

fn insert(
    table: &'static str,
    columns: &'static [&'static str],
    values: Vec<SqlValue>,
) -> BoxFuture<'static, Result<u64>> {
    Box::pin(async move {
        let database = current_database()?;
        let driver = database.driver();
        let (names, values): (Vec<_>, Vec<_>) = columns
            .iter()
            .zip(values)
            .filter(|(column, _)| **column != "id")
            .map(|(column, value)| (driver.quote(column), value))
            .unzip();
        let placeholders = vec!["?"; names.len()].join(", ");
        let sql = format!(
            "insert into {} ({}) values ({placeholders})",
            driver.quote(table),
            names.join(", ")
        );
        let failed = |error: sqlx::Error| {
            Error::from(error).context(format!("could not insert into `{table}`"))
        };
        let id = match driver {
            // sqlx's `Any` driver reports no last insert id for SQLite, so ask for it back.
            Driver::Sqlite => {
                let sql = format!("{sql} returning {}", driver.quote("id"));
                let row = bind_all(sqlx::query(&sql), values)
                    .fetch_one(database.pool())
                    .await
                    .map_err(failed)?;
                Row(row).get::<i64>("id")?
            }
            Driver::MySql => bind_all(sqlx::query(&sql), values)
                .execute(database.pool())
                .await
                .map_err(failed)?
                .last_insert_id()
                .ok_or_else(|| no_id(table))?,
        };
        Ok(u64::try_from(id)?)
    })
}

fn update(
    table: &'static str,
    columns: &'static [&'static str],
    values: Vec<SqlValue>,
    id: u64,
) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let database = current_database()?;
        let driver = database.driver();
        let (sets, mut values): (Vec<_>, Vec<_>) = columns
            .iter()
            .zip(values)
            .filter(|(column, _)| **column != "id")
            .map(|(column, value)| (format!("{} = ?", driver.quote(column)), value))
            .unzip();
        values.push(SqlValue::Integer(i64::try_from(id)?));
        let sql = format!(
            "update {} set {} where {} = ?",
            driver.quote(table),
            sets.join(", "),
            driver.quote("id")
        );
        bind_all(sqlx::query(&sql), values)
            .execute(database.pool())
            .await
            .map_err(|error| {
                Error::from(error).context(format!("could not update `{table}` {id}"))
            })?;
        Ok(())
    })
}

fn bind_all<'q>(
    mut query: Query<'q, sqlx::Any, AnyArguments<'q>>,
    values: Vec<SqlValue>,
) -> Query<'q, sqlx::Any, AnyArguments<'q>> {
    for value in values {
        query = match value {
            SqlValue::Null => query.bind(None::<i64>),
            SqlValue::Integer(value) => query.bind(value),
            SqlValue::Float(value) => query.bind(value),
            SqlValue::Bool(value) => query.bind(value),
            SqlValue::Text(value) => query.bind(value),
        };
    }
    query
}

fn current_database() -> Result<Arc<Database>> {
    App::current()?.get::<Database>()
}

fn no_id(table: &str) -> Error {
    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        .context(format!("`{table}` gave no id for the new row"))
}

fn not_found(table: &str, id: u64) -> Error {
    Error::not_found().context(format!("no `{table}` row has id {id}"))
}
