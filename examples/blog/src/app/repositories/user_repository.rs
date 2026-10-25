use sabai::orm::{Model, is_unique_violation};
use sabai::{App, Error, FromApp, Result};

use crate::app::models::User;

/// Reads and writes users.
pub struct UserRepository;

impl FromApp for UserRepository {
    fn new(_app: &App) -> Result<Self> {
        Ok(Self)
    }
}

impl UserRepository {
    /// Fails with the `422` of a taken email when a user already has `email`.
    pub async fn ensure_email_is_free(&self, email: &str) -> Result<()> {
        match User::where_eq("email", email).first().await? {
            Some(_) => Err(email_taken()),
            None => Ok(()),
        }
    }

    /// Saves a new user. The unique index on `email` answers a taken email the way Laravel's
    /// `unique:users` rule does, without the race between checking and saving.
    pub async fn create(
        &self,
        name: String,
        email: String,
        hashed_password: String,
    ) -> Result<User> {
        let mut user = User {
            name,
            email,
            password: hashed_password,
            ..Default::default()
        };
        match user.save().await {
            Err(error) if is_unique_violation(&error) => Err(email_taken()),
            saved => saved.map(|()| user),
        }
    }
}

fn email_taken() -> Error {
    Error::with_messages(&[("email", "The email has already been taken.")])
}
