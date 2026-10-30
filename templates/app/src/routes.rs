//! Every route file, collected in one place.

mod web;

use sabai::http::Router;

/// All of the app's routes.
pub fn all() -> Router {
    let mut route = Router::new();
    web::routes(&mut route);
    route
}
