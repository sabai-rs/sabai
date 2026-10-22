use bcrypt::{BcryptError, Version};
use sabai_core::{Error, Result, StatusCode};

use super::HashError;

// bcrypt reads only the first 72 bytes, so a longer password is refused rather than cut short.
pub(super) fn make(password: &str, rounds: u32) -> Result<String> {
    match bcrypt::non_truncating_hash_with_result(password, rounds) {
        Ok(hashed) => Ok(hashed.format_for_version(Version::TwoY)),
        Err(BcryptError::Truncation(_)) => Err(Error::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "The password must not be longer than 72 bytes.",
        )),
        Err(error) => Err(Error::internal(HashError(error.to_string()))),
    }
}

// Like PHP's `password_verify`, which also reads 72 bytes, so every Laravel hash checks the same.
pub(super) fn matches(password: &str, hashed: &str) -> bool {
    bcrypt::verify(password, hashed).unwrap_or(false)
}

/// The cost in `$2y$12$...`.
pub(super) fn cost(hashed: &str) -> Option<u32> {
    hashed.get(4..6)?.parse().ok()
}
