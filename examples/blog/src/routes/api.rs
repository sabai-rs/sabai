//! Endpoints for programs, like Laravel's `routes/api.php`. Paths spell out `/api` until
//! route groups (M1-02) let `routes.rs` add the prefix for the whole file.

use sabai::http::Router;

pub fn routes(route: &mut Router) {
    route.get("/api/health", health);
}

async fn health() -> &'static str {
    "ok"
}
