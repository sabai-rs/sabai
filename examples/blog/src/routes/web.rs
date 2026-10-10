//! Pages for people, like Laravel's `routes/web.php`.

use sabai::http::Router;

pub fn routes(route: &mut Router) {
    route.get("/", home).name("home");
}

async fn home() -> &'static str {
    "Hello, Sabai"
}
