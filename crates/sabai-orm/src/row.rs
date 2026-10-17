use sabai_core::{Error, Result};
use sqlx::Row as _;
use sqlx::any::AnyRow;

/// One row of a query result; `#[derive(Model)]` reads each field from it.
pub struct Row(pub(crate) AnyRow);

impl Row {
    /// The value of `column` as `T`.
    pub fn get<T: FromColumn>(&self, column: &str) -> Result<T> {
        T::from_column(self, column)
            .map_err(|error| error.context(format!("could not read column `{column}`")))
    }
}

/// A Rust type a column can be read into.
pub trait FromColumn: Sized {
    /// Reads `column` from `row`.
    fn from_column(row: &Row, column: &str) -> Result<Self>;
}

macro_rules! read_directly {
    ($($type:ty),+) => {$(
        impl FromColumn for $type {
            fn from_column(row: &Row, column: &str) -> Result<Self> {
                Ok(row.0.try_get(column)?)
            }
        }
    )+};
}

// `sqlx::Any` has no unsigned integers, so they arrive as `i64`; a negative one is an error.
macro_rules! read_through_i64 {
    ($($type:ty),+) => {$(
        impl FromColumn for $type {
            fn from_column(row: &Row, column: &str) -> Result<Self> {
                let value: i64 = row.0.try_get(column)?;
                Ok(<$type>::try_from(value)?)
            }
        }

        impl FromColumn for Option<$type> {
            fn from_column(row: &Row, column: &str) -> Result<Self> {
                let value: Option<i64> = row.0.try_get(column)?;
                value.map(<$type>::try_from).transpose().map_err(Error::from)
            }
        }
    )+};
}

read_directly!(
    i64,
    i32,
    f64,
    bool,
    Option<i64>,
    Option<i32>,
    Option<f64>,
    Option<bool>
);
read_through_i64!(u64, u32);

// sqlx's `Any` driver hands MySQL `TEXT` columns over as `BLOB`, so text may arrive as bytes.
impl FromColumn for String {
    fn from_column(row: &Row, column: &str) -> Result<Self> {
        match row.0.try_get::<String, _>(column) {
            Ok(text) => Ok(text),
            Err(_) => Ok(String::from_utf8(row.0.try_get::<Vec<u8>, _>(column)?)?),
        }
    }
}

impl FromColumn for Option<String> {
    fn from_column(row: &Row, column: &str) -> Result<Self> {
        match row.0.try_get::<Option<String>, _>(column) {
            Ok(text) => Ok(text),
            Err(_) => {
                let bytes: Option<Vec<u8>> = row.0.try_get(column)?;
                Ok(bytes.map(String::from_utf8).transpose()?)
            }
        }
    }
}
