use std::sync::Arc;

use sabai::http::{Controller, Param};
use sabai::{App, FromApp, Json, Result, StatusCode};

use crate::app::http::requests::StorePostRequest;
use crate::app::models::Post;
use crate::app::services::PostService;

/// Post endpoints; the work happens in [`PostService`].
pub struct PostController {
    post_service: PostService,
}

impl FromApp for PostController {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_service: app.make::<PostService>()?,
        })
    }
}

impl Controller for PostController {}

impl PostController {
    pub async fn index(self: Arc<Self>) -> Result<Json<Vec<Post>>> {
        Ok(Json(self.post_service.all().await?))
    }

    pub async fn store(
        self: Arc<Self>,
        request: StorePostRequest,
    ) -> Result<(StatusCode, Json<Post>)> {
        Ok((
            StatusCode::CREATED,
            Json(self.post_service.create(request).await?),
        ))
    }

    pub async fn show(self: Arc<Self>, Param(id): Param<u64>) -> Result<Json<Post>> {
        Ok(Json(self.post_service.read(id).await?))
    }
}
