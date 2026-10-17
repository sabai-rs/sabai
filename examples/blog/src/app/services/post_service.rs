use sabai::{App, FromApp, Result};

use crate::app::http::requests::StorePostRequest;
use crate::app::models::Post;
use crate::app::repositories::{PostRepository, PostViewRepository};

/// Creating and finding posts.
pub struct PostService {
    post_repository: PostRepository,
    post_view_repository: PostViewRepository,
}

impl FromApp for PostService {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_repository: app.make::<PostRepository>()?,
            post_view_repository: app.make::<PostViewRepository>()?,
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

    /// The post, counting this read in the report database.
    pub async fn read(&self, id: u64) -> Result<Post> {
        let post = self.post_repository.find_or_fail(id).await?;
        self.post_view_repository.record(post.id).await?;
        Ok(post)
    }
}
