use sabai::orm::Model;
use sabai::{App, DateTime, FromApp, Result};

use crate::app::models::Post;

/// Reads and writes posts.
pub struct PostRepository;

impl FromApp for PostRepository {
    fn new(_app: &App) -> Result<Self> {
        Ok(Self)
    }
}

impl PostRepository {
    pub async fn all(&self) -> Result<Vec<Post>> {
        Post::all().await
    }

    pub async fn find_or_fail(&self, id: u64) -> Result<Post> {
        Post::find_or_fail(id).await
    }

    pub async fn create(&self, title: String, body: String) -> Result<Post> {
        let now = Some(DateTime::now());
        let mut post = Post {
            id: 0,
            title,
            body,
            created_at: now,
            updated_at: now,
        };
        post.save().await?;
        Ok(post)
    }
}
