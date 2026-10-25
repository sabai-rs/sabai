use sabai::orm::Model;
use sabai::{App, FromApp, Result};

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
        let mut post = Post {
            title,
            body,
            ..Default::default()
        };
        post.save().await?;
        Ok(post)
    }
}
