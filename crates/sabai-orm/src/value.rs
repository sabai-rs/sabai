use sabai_core::Result;

/// A value bound into a query.
#[derive(Clone, Debug, PartialEq)]
pub enum SqlValue {
    /// `NULL`.
    Null,
    /// Any integer column.
    Integer(i64),
    /// A floating-point column.
    Float(f64),
    /// A boolean column.
    Bool(bool),
    /// A text column.
    Text(String),
}

/// A Rust type that can be written into a column.
pub trait ToColumn {
    /// The value to bind.
    fn to_column(&self) -> Result<SqlValue>;
}

macro_rules! to_column {
    ($($type:ty => $variant:ident),+) => {$(
        impl ToColumn for $type {
            fn to_column(&self) -> Result<SqlValue> {
                Ok(SqlValue::$variant(self.clone().into()))
            }
        }
    )+};
}

to_column!(i64 => Integer, i32 => Integer, f64 => Float, bool => Bool, String => Text);

// `sqlx::Any` binds no unsigned integers; values above `i64::MAX` are refused, not wrapped.
macro_rules! to_column_through_i64 {
    ($($type:ty),+) => {$(
        impl ToColumn for $type {
            fn to_column(&self) -> Result<SqlValue> {
                Ok(SqlValue::Integer(i64::try_from(*self)?))
            }
        }
    )+};
}

to_column_through_i64!(u64, u32);

impl<T: ToColumn> ToColumn for Option<T> {
    fn to_column(&self) -> Result<SqlValue> {
        match self {
            Some(value) => value.to_column(),
            None => Ok(SqlValue::Null),
        }
    }
}

impl ToColumn for str {
    fn to_column(&self) -> Result<SqlValue> {
        Ok(SqlValue::Text(self.to_owned()))
    }
}

// So `where_eq("email", "a@b.c")` and `where_eq("email", &request.email)` work without `.to_owned()`.
impl<T: ToColumn + ?Sized> ToColumn for &T {
    fn to_column(&self) -> Result<SqlValue> {
        (**self).to_column()
    }
}
