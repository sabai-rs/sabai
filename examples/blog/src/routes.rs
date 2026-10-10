use sabai::http::Router;

/// The blog's routes, like Laravel's `routes/web.php`.
pub fn web() -> Router {
    Router::new().get("/", home)
}

async fn home() -> &'static str {
    "Hello, Sabai"
}
