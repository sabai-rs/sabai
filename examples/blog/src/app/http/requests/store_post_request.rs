use sabai::http::FormRequest;
use serde::Deserialize;

/// The input for creating a post.
#[derive(Deserialize)]
pub struct StorePostRequest {
    pub title: String,
    pub body: String,
}

impl FormRequest for StorePostRequest {}
