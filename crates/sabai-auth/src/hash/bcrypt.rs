use bcrypt::Version;
use sabai_core::{Error, Result, StatusCode};

use super::HashError;

/// bcrypt reads only a password's first 72 bytes, so by default a longer one is refused instead of
/// cut short; `limit = 0` cuts it short like Laravel's default.
pub(super) const DEFAULT_LIMIT: usize = 72;

pub(super) fn make(password: &str, rounds: u32, limit: usize) -> Result<String> {
    if limit > 0 && password.len() > limit {
        return Err(Error::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            format!("The password must not be longer than {limit} bytes."),
        ));
    }
    let hashed = bcrypt::hash_with_result(password, rounds)
        .map_err(|error| Error::internal(HashError(error.to_string())))?;
    Ok(hashed.format_for_version(Version::TwoY))
}

// Like PHP's `password_verify`, which also reads 72 bytes, so every Laravel hash checks the same.
pub(super) fn matches(password: &str, hashed: &str) -> bool {
    bcrypt::verify(password, hashed).unwrap_or(false)
}

/// The cost in `$2y$12$...`.
pub(super) fn cost(hashed: &str) -> Option<u32> {
    hashed.get(4..6)?.parse().ok()
}
