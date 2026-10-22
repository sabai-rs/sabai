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
    /// Saves a new user. The unique index on `email` answers a taken email the way Laravel's
    /// `unique:users` rule does, without the race between checking and saving.
    pub async fn create(
        &self,
        name: String,
        email: String,
        hashed_password: String,
    ) -> Result<User> {
        let mut user = User {
            id: 0,
            name,
            email,
            password: hashed_password,
        };
        match user.save().await {
            Err(error) if is_unique_violation(&error) => Err(Error::with_messages(&[(
                "email",
                "The email has already been taken.",
            )])),
            saved => saved.map(|()| user),
        }
    }
}
