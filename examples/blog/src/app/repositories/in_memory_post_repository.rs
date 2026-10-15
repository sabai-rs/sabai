use std::sync::{Mutex, PoisonError};

use crate::app::models::Post;
use crate::app::repositories::PostRepository;

/// Keeps posts in memory until the ORM (M2) stores them in a database; they are lost on restart.
#[derive(Default)]
pub struct InMemoryPostRepository {
    posts: Mutex<Vec<Post>>,
}

impl PostRepository for InMemoryPostRepository {
    fn all(&self) -> Vec<Post> {
        self.posts
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .clone()
    }

    fn create(&self, title: String, body: String) -> Post {
        let mut posts = self.posts.lock().unwrap_or_else(PoisonError::into_inner);
        let post = Post {
            id: posts.len() as u64 + 1,
            title,
            body,
        };
        posts.push(post.clone());
        post
    }

    fn find(&self, id: u64) -> Option<Post> {
        let posts = self.posts.lock().unwrap_or_else(PoisonError::into_inner);
        posts.iter().find(|post| post.id == id).cloned()
    }
}
