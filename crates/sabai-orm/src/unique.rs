use sabai_core::Error;

/// Whether `error` came from a unique index, such as a second user with the same email, like
/// Laravel's `UniqueConstraintViolationException`. Answer it with the `422` a `unique` rule gives:
/// `Error::with_messages(&[("email", "The email has already been taken.")])`.
pub fn is_unique_violation(error: &Error) -> bool {
    let database_error = error
        .source()
        .and_then(|source| source.downcast_ref::<sqlx::Error>())
        .and_then(sqlx::Error::as_database_error);
    database_error.is_some_and(|error| error.is_unique_violation())
}
