use argon2::password_hash::phc::PasswordHash;
use argon2::password_hash::{PasswordHasher, PasswordVerifier};
use argon2::{Algorithm, Argon2, Params, Version};
use sabai_core::{Error, Result};

use super::HashError;

pub(super) fn make(password: &str, memory: u32, time: u32, threads: u32) -> Result<String> {
    let failed = |error: &dyn std::fmt::Display| Error::internal(HashError(error.to_string()));
    let params = Params::new(memory, time, threads, None).map_err(|error| failed(&error))?;
    let argon2 = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
    let hashed = argon2
        .hash_password(password.as_bytes())
        .map_err(|error| failed(&error))?;
    Ok(hashed.to_string())
}

// The costs come from the hash itself, so hashes made with other settings still check.
pub(super) fn matches(password: &str, hashed: &str) -> bool {
    let Ok(hash) = PasswordHash::new(hashed) else {
        return false;
    };
    Argon2::default()
        .verify_password(password.as_bytes(), &hash)
        .is_ok()
}

/// The memory, time and threads `hashed` was made with.
pub(super) fn costs(hashed: &str) -> Option<(u32, u32, u32)> {
    let hash = PasswordHash::new(hashed).ok()?;
    let params = Params::try_from(&hash).ok()?;
    Some((params.m_cost(), params.t_cost(), params.p_cost()))
}
