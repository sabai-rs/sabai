use sabai::auth::Authenticatable;
use sabai::{Error, Json, Result};

use crate::app::http::requests::LoginRequest;
use crate::app::models::User;

/// Signs people in, like Laravel Breeze's `AuthenticatedSessionController`.
pub struct AuthController;

impl AuthController {
    /// One message for an unknown email and a wrong password, so neither tells which it was.
    pub async fn login(request: LoginRequest) -> Result<Json<User>> {
        match User::validate_credentials(&request.email, &request.password).await? {
            Some(user) => Ok(Json(user)),
            None => Err(Error::with_messages(&[(
                "email",
                "These credentials do not match our records.",
            )])),
        }
    }
}
