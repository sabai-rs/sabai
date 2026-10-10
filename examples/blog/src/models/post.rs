use serde::Serialize;

/// A blog post.
#[derive(Serialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
}
