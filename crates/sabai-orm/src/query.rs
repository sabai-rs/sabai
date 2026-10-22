use std::marker::PhantomData;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, Result, StatusCode};

use crate::model::select;
use crate::{Model, SqlValue, ToColumn};

/// Rows of a model that match conditions, like Laravel's query builder:
/// `User::where_eq("email", email).first()`. Values are always bound, never written into the SQL.
pub struct Query<M> {
    conditions: Conditions,
    model: PhantomData<fn() -> M>,
}

/// The `where` conditions, kept apart from the model so building and running them compiles once.
pub(crate) struct Conditions {
    pub(crate) equals: Vec<(String, SqlValue)>,
    problem: Option<Error>,
}

impl<M: Model> Query<M> {
    pub(crate) fn new() -> Self {
        Self {
            conditions: Conditions {
                equals: Vec::new(),
                problem: None,
            },
            model: PhantomData,
        }
    }

    /// Keeps rows where `column` equals `value`, like `->where('email', $email)`; each call adds
    /// an `and`. A column the model does not have is an error when the query runs.
    pub fn where_eq(mut self, column: &str, value: impl ToColumn) -> Self {
        self.conditions
            .push(M::TABLE, M::COLUMNS, column, value.to_column());
        self
    }

    /// Every matching row, like `->get()`.
    pub fn get(self) -> BoxFuture<'static, Result<Vec<M>>> {
        let rows = run(M::CONNECTION, M::TABLE, M::COLUMNS, self.conditions, None);
        Box::pin(async move { rows.await?.iter().map(M::from_row).collect() })
    }

    /// The first matching row, or `None`, like `->first()`.
    pub fn first(self) -> BoxFuture<'static, Result<Option<M>>> {
        let rows = run(
            M::CONNECTION,
            M::TABLE,
            M::COLUMNS,
            self.conditions,
            Some(1),
        );
        Box::pin(async move { rows.await?.first().map(M::from_row).transpose() })
    }
}

impl Conditions {
    fn push(&mut self, table: &str, columns: &[&str], column: &str, value: Result<SqlValue>) {
        if self.problem.is_some() {
            return;
        }
        // Only the model's own columns, so a name that came from user input can never reach the SQL.
        if !columns.contains(&column) {
            self.problem = Some(unknown_column(table, columns, column));
            return;
        }
        match value {
            Ok(value) => self.equals.push((column.to_owned(), value)),
            Err(error) => self.problem = Some(error),
        }
    }
}

fn run(
    connection: Option<&'static str>,
    table: &'static str,
    columns: &'static [&'static str],
    conditions: Conditions,
    limit: Option<u64>,
) -> BoxFuture<'static, Result<Vec<crate::Row>>> {
    match conditions.problem {
        Some(problem) => Box::pin(async move { Err(problem) }),
        None => select(connection, table, columns, conditions.equals, limit),
    }
}

fn unknown_column(table: &str, columns: &[&str], column: &str) -> Error {
    Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "`{table}` has no column `{column}`; its columns are {}",
            columns.join(", ")
        ),
    )
}
