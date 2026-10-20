//! Every route file, collected in one place like Laravel's `withRouting(web: ..., api: ...)`.
//! Add a file under `routes/` and one line here, such as `route.prefix("/client").group(client::routes)`.

mod api;
mod web;

use sabai::http::Router;

use crate::app::http::middleware::FrameGuard;

/// All of the blog's routes.
pub fn all() -> Router {
    let mut route = Router::new();
    web::routes(&mut route);
    route.prefix("/api").group(api::routes);
    route.middleware(FrameGuard);
    route
}
