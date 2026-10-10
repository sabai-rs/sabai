//! Pages for people, like Laravel's `routes/web.php`.

use sabai::http::{Param, Router};

pub fn routes(route: &mut Router) {
    route.get("/", home);
    route.get("/posts/{id}", show_post);
}

async fn home() -> &'static str {
    "Hello, Sabai"
}

async fn show_post(Param(id): Param<u64>) -> String {
    format!("Post {id}")
}
