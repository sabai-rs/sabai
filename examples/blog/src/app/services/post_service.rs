use std::sync::Arc;

use sabai::{App, Error, FromApp, Result};

use crate::app::http::requests::StorePostRequest;
use crate::app::models::Post;
use crate::app::repositories::PostRepository;

/// Creating and finding posts, whichever repository stores them.
pub struct PostService {
    post_repository: Arc<dyn PostRepository>,
}

impl FromApp for PostService {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_repository: app.get::<dyn PostRepository>()?,
        })
    }
}

impl PostService {
    pub fn create(&self, request: StorePostRequest) -> Post {
        self.post_repository.create(request.title, request.body)
    }

    pub fn find_or_fail(&self, id: u64) -> Result<Post> {
        self.post_repository.find(id).ok_or_else(Error::not_found)
    }
}
