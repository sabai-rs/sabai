use std::fmt;

/// An email address with an optional display name: `"Ann" <ann@example.com>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Address {
    email: String,
    name: Option<String>,
}

impl Address {
    /// An address with a display name.
    pub fn named(email: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            email: email.into(),
            name: Some(name.into()),
        }
    }

    /// The bare address, such as `ann@example.com`.
    pub fn email(&self) -> &str {
        &self.email
    }

    /// The display name, if any.
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }
}

impl From<&str> for Address {
    fn from(email: &str) -> Self {
        Self {
            email: email.to_owned(),
            name: None,
        }
    }
}

impl From<String> for Address {
    fn from(email: String) -> Self {
        Self { email, name: None }
    }
}

impl fmt::Display for Address {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.name {
            Some(name) => write!(f, "\"{name}\" <{}>", self.email),
            None => f.write_str(&self.email),
        }
    }
}
