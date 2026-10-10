use serde::Serialize;

/// A blog post.
#[derive(Clone, Serialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
}
