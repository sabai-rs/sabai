//! Endpoints for programs, like Laravel's `routes/api.php`; `routes.rs` puts them under `/api`.

use std::time::UNIX_EPOCH;

use sabai::Result;
use sabai::contracts::Clock;
use sabai::http::{Router, Service};

use crate::app::http::controllers::PostController;

pub fn routes(route: &mut Router) {
    route.get("/health", health).name("health");
    route
        .post("/posts", PostController::store)
        .name("posts.store");
    route
        .get("/posts/{id}", PostController::show)
        .name("posts.show");
}

async fn health(clock: Service<dyn Clock>) -> Result<String> {
    let now = clock.now().duration_since(UNIX_EPOCH)?;
    Ok(format!("ok at {}", now.as_secs()))
}
