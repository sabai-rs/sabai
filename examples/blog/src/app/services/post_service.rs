use sabai::{App, FromApp, Result};

use crate::app::http::requests::StorePostRequest;
use crate::app::models::Post;
use crate::app::repositories::PostRepository;

/// Creating and finding posts.
pub struct PostService {
    post_repository: PostRepository,
}

impl FromApp for PostService {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_repository: app.make::<PostRepository>()?,
        })
    }
}

impl PostService {
    pub async fn all(&self) -> Result<Vec<Post>> {
        self.post_repository.all().await
    }

    pub async fn create(&self, request: StorePostRequest) -> Result<Post> {
        self.post_repository
            .create(request.title, request.body)
            .await
    }

    pub async fn find_or_fail(&self, id: u64) -> Result<Post> {
        self.post_repository.find_or_fail(id).await
    }
}
