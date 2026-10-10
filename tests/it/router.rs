use pollster::block_on;
use sabai::http::{Router, Routes};
use sabai::{Body, Error, Method, Request, Response, Result, StatusCode};

async fn index() -> &'static str {
    "All posts"
}

async fn health() -> Result<&'static str> {
    Ok("ok")
}

async fn broken() -> Result<&'static str> {
    Err(Error::forbidden())
}

fn routes() -> Routes {
    Router::new()
        .get("/posts", index)
        .get("/health", health)
        .get("/admin", broken)
        .compile()
        .unwrap()
}

fn send(routes: &Routes, method: Method, path: &str) -> Response {
    let request = http::Request::builder()
        .method(method)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    block_on(routes.dispatch(Request::from(request)))
}

#[test]
fn a_matching_route_runs_its_handler() {
    let response = send(&routes(), Method::GET, "/posts");

    assert_eq!(response.status(), StatusCode::OK);
    assert_eq!(response.body().as_bytes(), b"All posts");
}

#[test]
fn handlers_can_return_results() {
    let routes = routes();

    assert_eq!(
        send(&routes, Method::GET, "/health").body().as_bytes(),
        b"ok"
    );
    assert_eq!(
        send(&routes, Method::GET, "/admin").status(),
        StatusCode::FORBIDDEN
    );
}

#[test]
fn an_unknown_path_is_a_json_404() {
    let response = send(&routes(), Method::GET, "/nope");

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.body().as_bytes(), br#"{"message":"Not Found"}"#);
}

#[test]
fn a_known_path_with_the_wrong_method_is_a_405() {
    assert_eq!(
        send(&routes(), Method::POST, "/posts").status(),
        StatusCode::METHOD_NOT_ALLOWED
    );
}

#[test]
fn route_mistakes_are_reported_at_boot() {
    let duplicate = Router::new()
        .get("/posts", index)
        .get("/posts", index)
        .compile();
    let malformed = Router::new().get("/posts/{id", index).compile();

    let duplicate = duplicate.err().unwrap().to_string();
    let malformed = malformed.err().unwrap().to_string();
    assert_eq!(
        duplicate,
        "route `GET /posts` is invalid: it is defined twice; remove one of them"
    );
    assert!(
        malformed.starts_with("route `GET /posts/{id` is invalid: "),
        "{malformed}"
    );
}
