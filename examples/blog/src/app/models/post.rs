use sabai::DateTime;
use sabai::orm::Model;
use serde::Serialize;

/// A blog post.
#[derive(Clone, Default, Serialize, Model)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
    pub created_at: Option<DateTime>,
    pub updated_at: Option<DateTime>,
}
