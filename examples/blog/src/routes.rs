//! Every route file, collected in one place like Laravel's `withRouting(web: ..., api: ...)`.
//! Add a file under `routes/` and one line here, such as `api_client::routes(&mut route)`.

mod api;
mod web;

use sabai::http::Router;

/// All of the blog's routes.
pub fn all() -> Router {
    let mut route = Router::new();
    web::routes(&mut route);
    api::routes(&mut route);
    route
}
