//! Endpoints for programs, like Laravel's `routes/api.php`. Paths spell out `/api` until
//! route groups (M1-02) let `routes.rs` add the prefix for the whole file.

use std::time::UNIX_EPOCH;

use sabai::contracts::Clock;
use sabai::http::{Router, Service};
use sabai::{Json, Result, StatusCode};

use crate::models::Post;
use crate::requests::StorePostRequest;

pub fn routes(route: &mut Router) {
    route.get("/api/health", health);
    route.post("/api/posts", store_post);
}

async fn health(clock: Service<dyn Clock>) -> Result<String> {
    let now = clock.now().duration_since(UNIX_EPOCH)?;
    Ok(format!("ok at {}", now.as_secs()))
}

async fn store_post(request: StorePostRequest) -> (StatusCode, Json<Post>) {
    // Echoes the post back with a made-up id until the ORM (M2) saves it.
    let post = Post {
        id: 1,
        title: request.title,
        body: request.body,
    };
    (StatusCode::CREATED, Json(post))
}
