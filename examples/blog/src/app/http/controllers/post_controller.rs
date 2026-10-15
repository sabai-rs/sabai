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
    pub async fn index(self: Arc<Self>) -> Json<Vec<Post>> {
        Json(self.post_service.all())
    }

    pub async fn store(self: Arc<Self>, request: StorePostRequest) -> (StatusCode, Json<Post>) {
        (StatusCode::CREATED, Json(self.post_service.create(request)))
    }

    pub async fn show(self: Arc<Self>, Param(id): Param<u64>) -> Result<Json<Post>> {
        Ok(Json(self.post_service.find_or_fail(id)?))
    }
}
