use sabai_core::{DateTime, Error, Result, StatusCode};
use sqlx::Row as _;

/// One row of a query result; `#[derive(Model)]` reads each field from it.
pub struct Row(pub(crate) DriverRow);

pub(crate) enum DriverRow {
    #[cfg(feature = "mysql")]
    MySql(sqlx::mysql::MySqlRow),
    #[cfg(feature = "sqlite")]
    Sqlite(sqlx::sqlite::SqliteRow),
}

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

// Each driver decodes the type the field asks for; the macro writes the `match` once. The type
// annotation lets the `match` compile with no driver, when it has no arms.
macro_rules! decode {
    ($row:expr, $column:expr) => {{
        let value: std::result::Result<_, sqlx::Error> = match $row.0 {
            #[cfg(feature = "mysql")]
            DriverRow::MySql(ref row) => row.try_get($column),
            #[cfg(feature = "sqlite")]
            DriverRow::Sqlite(ref row) => row.try_get($column),
        };
        value
    }};
}

macro_rules! read_directly {
    ($($type:ty),+) => {$(
        impl FromColumn for $type {
            fn from_column(row: &Row, column: &str) -> Result<Self> {
                Ok(decode!(row, column)?)
            }
        }
    )+};
}

read_directly!(f64, bool, String, Option<f64>, Option<bool>, Option<String>);

// MySQL keeps signed and unsigned integers apart (`id` is `bigint unsigned`), so a field reads
// whichever the column is and converts; a value that does not fit is an error, never wrapped.
fn signed(row: &Row, column: &str) -> Result<Option<i64>> {
    match row.0 {
        #[cfg(feature = "mysql")]
        DriverRow::MySql(ref row) => match row.try_get::<Option<i64>, _>(column) {
            Ok(value) => Ok(value),
            Err(_) => {
                let value: Option<u64> = row.try_get(column)?;
                Ok(value.map(i64::try_from).transpose()?)
            }
        },
        #[cfg(feature = "sqlite")]
        DriverRow::Sqlite(ref row) => Ok(row.try_get(column)?),
    }
}

fn unsigned(row: &Row, column: &str) -> Result<Option<u64>> {
    match row.0 {
        #[cfg(feature = "mysql")]
        DriverRow::MySql(ref row) => match row.try_get::<Option<u64>, _>(column) {
            Ok(value) => Ok(value),
            Err(_) => {
                let value: Option<i64> = row.try_get(column)?;
                Ok(value.map(u64::try_from).transpose()?)
            }
        },
        // SQLite has no unsigned integers; reading through `i64` refuses a negative one.
        #[cfg(feature = "sqlite")]
        DriverRow::Sqlite(ref row) => {
            let value: Option<i64> = row.try_get(column)?;
            Ok(value.map(u64::try_from).transpose()?)
        }
    }
}

macro_rules! read_integer {
    ($read:ident: $($type:ty),+) => {$(
        impl FromColumn for $type {
            fn from_column(row: &Row, column: &str) -> Result<Self> {
                let value = $read(row, column)?.ok_or_else(null)?;
                Ok(<$type>::try_from(value)?)
            }
        }

        impl FromColumn for Option<$type> {
            fn from_column(row: &Row, column: &str) -> Result<Self> {
                let value = $read(row, column)?;
                Ok(value.map(<$type>::try_from).transpose()?)
            }
        }
    )+};
}

read_integer!(signed: i64, i32);
read_integer!(unsigned: u64, u32);

impl FromColumn for DateTime {
    fn from_column(row: &Row, column: &str) -> Result<Self> {
        date_time(row, column)?.ok_or_else(null)
    }
}

impl FromColumn for Option<DateTime> {
    fn from_column(row: &Row, column: &str) -> Result<Self> {
        date_time(row, column)
    }
}

fn date_time(row: &Row, column: &str) -> Result<Option<DateTime>> {
    match row.0 {
        #[cfg(feature = "mysql")]
        DriverRow::MySql(ref row) => {
            let time: Option<crate::mysql_timestamp::MySqlTimestamp> = row.try_get(column)?;
            Ok(time.map(|time| time.0))
        }
        // SQLite has no date type: a `datetime` column holds the text `2026-10-23 10:00:00`.
        #[cfg(feature = "sqlite")]
        DriverRow::Sqlite(ref row) => {
            let text: Option<String> = row.try_get(column)?;
            text.map(|text| text.parse()).transpose()
        }
    }
}

fn null() -> Error {
    Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "the column is NULL; make the field an `Option`",
    )
}
