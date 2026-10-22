use sabai::http::FormRequest;
use sabai::http::rules::{Rules, email, required, string};
use serde::Deserialize;

/// The input for signing in.
#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

impl FormRequest for LoginRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("email", [required(), string(), email()])
            .field("password", [required(), string()])
    }
}
