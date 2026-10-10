//! Pages for people, like Laravel's `routes/web.php`.

use sabai::Request;
use sabai::http::Router;

pub fn routes(route: &mut Router) {
    route.get("/", home);
    route.get("/posts/{id}", show_post);
}

async fn home() -> &'static str {
    "Hello, Sabai"
}

async fn show_post(request: Request) -> String {
    format!("Post {}", request.param("id").unwrap_or_default())
}
