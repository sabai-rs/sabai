use std::sync::Arc;

use sabai::auth::Hash;
use sabai::http::Controller;
use sabai::{App, FromApp, Json, Result, StatusCode};

use crate::app::http::requests::RegisterRequest;
use crate::app::models::User;
use crate::app::repositories::UserRepository;

/// Creates accounts, like Laravel Breeze's `RegisteredUserController`.
pub struct RegisteredUserController {
    user_repository: UserRepository,
}

impl FromApp for RegisteredUserController {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            user_repository: app.make::<UserRepository>()?,
        })
    }
}

impl Controller for RegisteredUserController {}

impl RegisteredUserController {
    pub async fn store(
        self: Arc<Self>,
        request: RegisterRequest,
    ) -> Result<(StatusCode, Json<User>)> {
        // Checking first answers a taken email without spending a hash; the index still settles a race.
        self.user_repository
            .ensure_email_is_free(&request.email)
            .await?;
        let password = Hash::make(&request.password).await?;
        let user = self
            .user_repository
            .create(request.name, request.email, password)
            .await?;
        Ok((StatusCode::CREATED, Json(user)))
    }
}
