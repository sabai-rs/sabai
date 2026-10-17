use sabai_core::contracts::BoxFuture;
use sabai_core::{App, Result};

use crate::{Database, Row};

/// A table row as a struct, like an Eloquent model. Write `#[derive(Model)]`, not this impl.
pub trait Model: Sized + Send + 'static {
    /// The table, such as `posts` for `Post`.
    const TABLE: &'static str;

    /// The columns, in field order.
    const COLUMNS: &'static [&'static str];

    /// Builds the model from one row.
    fn from_row(row: &Row) -> Result<Self>;

    /// Every row of the table, like Laravel's `Post::all()`.
    fn all() -> BoxFuture<'static, Result<Vec<Self>>> {
        let rows = select_all(Self::TABLE, Self::COLUMNS);
        Box::pin(async move { rows.await?.iter().map(Self::from_row).collect() })
    }
}

fn select_all(
    table: &'static str,
    columns: &'static [&'static str],
) -> BoxFuture<'static, Result<Vec<Row>>> {
    Box::pin(async move {
        let database = App::current()?.get::<Database>()?;
        let driver = database.driver();
        let columns: Vec<_> = columns.iter().map(|column| driver.quote(column)).collect();
        let sql = format!("select {} from {}", columns.join(", "), driver.quote(table));
        let rows = sqlx::query(&sql).fetch_all(database.pool()).await?;
        Ok(rows.into_iter().map(Row).collect())
    })
}
