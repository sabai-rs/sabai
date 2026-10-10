use pollster::block_on;
use sabai::contracts::Clock;
use sabai::http::{Param, Router, Routes, Service};
use sabai::{App, Body, Config, Error, Method, Request, Response, Result, StatusCode};

async fn index() -> &'static str {
    "All posts"
}

async fn health() -> Result<&'static str> {
    Ok("ok")
}

async fn broken() -> Result<&'static str> {
    Err(Error::forbidden())
}

async fn show(request: Request) -> String {
    format!("post {}", request.param("id").unwrap_or("?"))
}

async fn file(request: Request) -> String {
    format!("file {}", request.param("path").unwrap_or("?"))
}

async fn show_typed(Param(id): Param<u64>, request: Request) -> String {
    format!("post {id} via {}", request.method())
}

async fn uptime(clock: Service<dyn Clock>) -> String {
    format!("{:?}", clock.now())
}

async fn method_name(request: Request) -> String {
    request.method().to_string()
}

fn routes() -> Routes {
    let mut router = Router::new();
    router.get("/posts", index);
    router.get("/health", health);
    router.get("/admin", broken);
    router.compile().unwrap()
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
    let mut duplicate = Router::new();
    duplicate.get("/posts", index);
    duplicate.get("/posts", index);
    let mut malformed = Router::new();
    malformed.get("/posts/{id", index);

    let duplicate = duplicate.compile().err().unwrap().to_string();
    let malformed = malformed.compile().err().unwrap().to_string();

    assert_eq!(
        duplicate,
        "route `GET /posts` is invalid: it is defined twice; remove one of them"
    );
    assert!(
        malformed.starts_with("route `GET /posts/{id` is invalid: "),
        "{malformed}"
    );
}

#[test]
fn route_params_reach_the_handler() {
    let mut router = Router::new();
    router.get("/posts/{id}", show);
    router.get("/files/{*path}", file);
    let routes = router.compile().unwrap();

    assert_eq!(
        send(&routes, Method::GET, "/posts/7").body().as_bytes(),
        b"post 7"
    );
    assert_eq!(
        send(&routes, Method::GET, "/files/docs/terms.pdf")
            .body()
            .as_bytes(),
        b"file docs/terms.pdf"
    );
}

#[test]
fn every_method_routes_to_its_own_handler() {
    let mut router = Router::new();
    router.get("/posts/{id}", method_name);
    router.post("/posts/{id}", method_name);
    router.put("/posts/{id}", method_name);
    router.patch("/posts/{id}", method_name);
    router.delete("/posts/{id}", method_name);
    let routes = router.compile().unwrap();

    for method in [
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::PATCH,
        Method::DELETE,
    ] {
        let response = send(&routes, method.clone(), "/posts/1");
        assert_eq!(response.body().as_bytes(), method.as_str().as_bytes());
    }
}

#[test]
fn handlers_take_typed_params_alongside_the_request() {
    let mut router = Router::new();
    router.get("/posts/{id}", show_typed);
    let routes = router.compile().unwrap();

    let found = send(&routes, Method::GET, "/posts/7");
    let not_a_number = send(&routes, Method::GET, "/posts/abc");

    assert_eq!(found.body().as_bytes(), b"post 7 via GET");
    assert_eq!(not_a_number.status(), StatusCode::NOT_FOUND);
}

#[test]
fn a_route_whose_service_is_not_bound_stops_startup_and_names_the_route() {
    let mut router = Router::new();
    router.get("/uptime", uptime);
    let routes = router.compile().unwrap();
    let app = App::builder(Config::default()).boot().unwrap();

    let error = routes.prepare(&app).err().unwrap().to_string();

    assert!(
        error.starts_with("route `GET /uptime` cannot start: no service is bound to `dyn"),
        "{error}"
    );
}
