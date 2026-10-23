use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, Error, Result, StatusCode};

use crate::{Database, DatabaseManager, Query, Row, SqlValue, ToColumn};

/// A table row as a struct, like an Eloquent model. Write `#[derive(Model)]`, not this impl.
pub trait Model: Sized + Send + 'static {
    /// The table, such as `posts` for `Post`.
    const TABLE: &'static str;

    /// The columns, in field order.
    const COLUMNS: &'static [&'static str];

    /// The connection in `config/database.toml`, like Laravel's `$connection`; `None` is `default`.
    const CONNECTION: Option<&'static str> = None;

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
        Query::new().get()
    }

    /// Rows where `column` equals `value`, like Laravel's `User::where('email', $email)`: add more
    /// with `.where_eq(...)`, then call `.first()` or `.get()`.
    fn where_eq(column: &str, value: impl ToColumn) -> Query<Self> {
        Query::new().where_eq(column, value)
    }

    /// The row with this primary key, or `None`, like Laravel's `Post::find($id)`.
    fn find(id: u64) -> BoxFuture<'static, Result<Option<Self>>> {
        Self::where_eq("id", id).first()
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
                0 => self
                    .set_id(insert(Self::CONNECTION, Self::TABLE, Self::COLUMNS, values?).await?),
                id => update(Self::CONNECTION, Self::TABLE, Self::COLUMNS, values?, id).await?,
            }
            Ok(())
        })
    }
}

pub(crate) fn select(
    connection: Option<&'static str>,
    table: &'static str,
    columns: &'static [&'static str],
    equals: Vec<(String, SqlValue)>,
    limit: Option<u64>,
) -> BoxFuture<'static, Result<Vec<Row>>> {
    Box::pin(async move {
        let database = database(connection)?;
        let driver = database.driver();
        let names: Vec<_> = columns.iter().map(|column| driver.quote(column)).collect();
        let mut sql = format!("select {} from {}", names.join(", "), driver.quote(table));
        let mut conditions = Vec::new();
        let mut values = Vec::new();
        for (column, value) in equals {
            // `= NULL` matches nothing in SQL, so `None` asks for `is null`, like Laravel.
            match value {
                SqlValue::Null => conditions.push(format!("{} is null", driver.quote(&column))),
                value => {
                    conditions.push(format!("{} = ?", driver.quote(&column)));
                    values.push(value);
                }
            }
        }
        if !conditions.is_empty() {
            sql.push_str(&format!(" where {}", conditions.join(" and ")));
        }
        if let Some(limit) = limit {
            sql.push_str(&format!(" limit {limit}"));
        }
        database.pool()?.fetch_all(&sql, values).await
    })
}

fn insert(
    connection: Option<&'static str>,
    table: &'static str,
    columns: &'static [&'static str],
    values: Vec<SqlValue>,
) -> BoxFuture<'static, Result<u64>> {
    Box::pin(async move {
        let database = database(connection)?;
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
        database
            .pool()?
            .execute(&sql, values)
            .await
            .map_err(|error| error.context(format!("could not insert into `{table}`")))?
            .ok_or_else(|| no_id(table))
    })
}

fn update(
    connection: Option<&'static str>,
    table: &'static str,
    columns: &'static [&'static str],
    values: Vec<SqlValue>,
    id: u64,
) -> BoxFuture<'static, Result<()>> {
    Box::pin(async move {
        let database = database(connection)?;
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
        database
            .pool()?
            .execute(&sql, values)
            .await
            .map_err(|error| error.context(format!("could not update `{table}` {id}")))?;
        Ok(())
    })
}

fn database(connection: Option<&str>) -> Result<Arc<Database>> {
    let app = App::current()?;
    match connection {
        None => app.get::<Database>(),
        Some(name) => app.get::<DatabaseManager>()?.connection(name),
    }
}

fn not_found(table: &str, id: u64) -> Error {
    Error::not_found().context(format!("no `{table}` row has id {id}"))
}

fn no_id(table: &str) -> Error {
    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR)
        .context(format!("`{table}` gave no id for the new row"))
}
