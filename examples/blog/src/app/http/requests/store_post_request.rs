use sabai::http::FormRequest;
use sabai::http::rules::{Rules, max, required, string};
use serde::Deserialize;

/// The input for creating a post.
#[derive(Deserialize)]
pub struct StorePostRequest {
    pub title: String,
    pub body: String,
}

impl FormRequest for StorePostRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("title", [required(), string(), max(255)])
            .field("body", [required(), string()])
    }
}
