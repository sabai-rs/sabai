//! Endpoints for programs, like Laravel's `routes/api.php`. Paths spell out `/api` until
//! route groups (M1-02) let `routes.rs` add the prefix for the whole file.

use sabai::http::Router;
use sabai::{Json, StatusCode};

use crate::models::Post;
use crate::requests::StorePostRequest;

pub fn routes(route: &mut Router) {
    route.get("/api/health", health);
    route.post("/api/posts", store_post);
}

async fn health() -> &'static str {
    "ok"
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
