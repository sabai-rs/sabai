//! Endpoints for programs, like Laravel's `routes/api.php`. Paths spell out `/api` until
//! route groups (M1-02) let `routes.rs` add the prefix for the whole file.

use std::time::UNIX_EPOCH;

use sabai::Result;
use sabai::contracts::Clock;
use sabai::http::{Router, Service};

use crate::app::http::controllers::PostController;

pub fn routes(route: &mut Router) {
    route.get("/api/health", health);
    route.post("/api/posts", PostController::store);
    route.get("/api/posts/{id}", PostController::show);
}

async fn health(clock: Service<dyn Clock>) -> Result<String> {
    let now = clock.now().duration_since(UNIX_EPOCH)?;
    Ok(format!("ok at {}", now.as_secs()))
}
