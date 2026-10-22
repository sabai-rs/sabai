//! Endpoints for programs, like Laravel's `routes/api.php`; `routes.rs` puts them under `/api`.

use std::time::UNIX_EPOCH;

use sabai::Result;
use sabai::contracts::Clock;
use sabai::http::{Router, Service};
use sabai::orm::Database;

use crate::app::http::controllers::{PostController, RegisteredUserController, StatsController};

pub fn routes(route: &mut Router) {
    route.get("/health", health).name("health");
    route
        .post("/register", RegisteredUserController::store)
        .name("register");
    route
        .post("/posts/import", PostController::import)
        .name("posts.import");
    route.api_resource("posts", |posts| {
        posts.index(PostController::index);
        posts.store(PostController::store);
        posts.show(PostController::show);
    });
    route
        .get("/stats", StatsController::index)
        .name("stats.index");
}

async fn health(clock: Service<dyn Clock>, db: Service<Database>) -> Result<String> {
    db.ping().await?;
    let now = clock.now().duration_since(UNIX_EPOCH)?;
    Ok(format!("ok at {}", now.as_secs()))
}
