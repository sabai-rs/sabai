use sabai::http::FormRequest;
use sabai::http::rules::{
    Rules, confirmed, email, lowercase, max, max_bytes, min, required, string,
};
use serde::Deserialize;

/// The input for creating an account, with Laravel Breeze's rules.
#[derive(Deserialize)]
pub struct RegisterRequest {
    pub name: String,
    pub email: String,
    pub password: String,
}

impl FormRequest for RegisterRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("name", [required(), string(), max(255)])
            .field(
                "email",
                [required(), string(), lowercase(), email(), max(255)],
            )
            .field("password", [required(), confirmed(), min(8), max_bytes(72)])
    }
}
