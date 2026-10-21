use sabai::http::FormRequest;
use sabai::http::rules::{Rules, array, max, required, string};
use serde::Deserialize;

use super::StorePostRequest;

/// Several posts at once, such as when moving a blog from another platform.
#[derive(Deserialize)]
pub struct ImportPostsRequest {
    pub posts: Vec<StorePostRequest>,
}

impl FormRequest for ImportPostsRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("posts", [required(), array(), max(50)])
            .field("posts.*.title", [required(), string(), max(255)])
            .field("posts.*.body", [required(), string()])
    }
}
