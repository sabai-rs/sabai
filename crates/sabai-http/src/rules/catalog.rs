use super::Rule;
use super::checks::Check;

/// The field must be present and not empty: not missing, `null`, `""`, `[]` or `{}`.
pub fn required() -> Rule {
    Rule(Check::Required)
}

/// Check the field only when it is present; it may be left out.
pub fn sometimes() -> Rule {
    Rule(Check::Sometimes)
}

/// The field must be a string.
pub fn string() -> Rule {
    Rule(Check::String)
}

/// The field must be a list; check its items with a `*` field such as `posts.*.title`.
pub fn array() -> Rule {
    Rule(Check::Array)
}

/// The field must be an email address: one `@`, with no spaces, and a domain of letters,
/// digits, `-` and `.`, like Laravel's default `email` rule.
pub fn email() -> Rule {
    Rule(Check::Email)
}

/// The field must have no uppercase letters.
pub fn lowercase() -> Rule {
    Rule(Check::Lowercase)
}

/// The field must equal `{field}_confirmation`, such as `password_confirmation`.
pub fn confirmed() -> Rule {
    Rule(Check::Confirmed)
}

/// At least `limit` characters for a string, `limit` for a number, `limit` items for a list.
pub fn min(limit: u64) -> Rule {
    Rule(Check::Min(limit))
}

/// At most `limit` characters for a string, `limit` for a number, `limit` items for a list.
pub fn max(limit: u64) -> Rule {
    Rule(Check::Max(limit))
}

/// At most `limit` bytes for a string, where `max` counts characters: a Thai letter is three.
/// `max_bytes(72)` keeps a password within what bcrypt reads.
pub fn max_bytes(limit: u64) -> Rule {
    Rule(Check::MaxBytes(limit))
}
