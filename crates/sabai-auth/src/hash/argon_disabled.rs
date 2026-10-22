use sabai_core::{Result, log};

use super::settings::argon2_needs_its_feature;

pub(super) fn make(_password: &str, _memory: u32, _time: u32, _threads: u32) -> Result<String> {
    Err(argon2_needs_its_feature())
}

pub(super) fn matches(_password: &str, _hashed: &str) -> bool {
    log::error!(
        "an Argon2id password hash was checked: {}",
        argon2_needs_its_feature()
    );
    false
}

pub(super) fn costs(_hashed: &str) -> Option<(u32, u32, u32)> {
    None
}
