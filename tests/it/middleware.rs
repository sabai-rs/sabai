use pollster::block_on;
use sabai::header::{HeaderValue, X_FRAME_OPTIONS};
use sabai::http::middleware::{ConvertEmptyStringsToNull, FrameGuard};
use sabai::http::{Middleware, Next, Router};
use sabai::{Body, Error, IntoResponse, Json, Method, Request, Response, Result};
use serde_json::{Value, json};

async fn index() -> &'static str {
    "All posts"
}

/// Adds its name to the `x-tag` header on the way out.
struct Tag(&'static str);

impl Middleware for Tag {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let mut response = next.run(request).await;
        response
            .headers_mut()
            .append("x-tag", self.0.parse().unwrap());
        response
    }
}

struct Deny;

impl Middleware for Deny {
    async fn handle(&self, _request: Request, _next: Next) -> Response {
        Error::forbidden().into_response()
    }
}

async fn echo(request: Request) -> Result<Json<Value>> {
    Ok(Json(Value::Object(request.all()?.clone())))
}

async fn never_framed() -> Response {
    let mut response = "Never framed".into_response();
    let deny = HeaderValue::from_static("DENY");
    response.headers_mut().insert(X_FRAME_OPTIONS, deny);
    response
}

#[test]
fn global_middleware_runs_around_the_handler() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(Tag("first")).append(Tag("second"));
    });
    let routes = router.compile().unwrap();

    let response = block_on(routes.dispatch(Request::create(Method::GET, "/posts").unwrap()));

    let tags: Vec<_> = response.headers().get_all("x-tag").iter().collect();
    assert_eq!(response.status(), 200);
    // The last one added is innermost, so it changes the response first.
    assert_eq!(tags, ["second", "first"]);
}

#[test]
fn middleware_can_answer_without_running_the_handler() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(Deny);
    });
    let routes = router.compile().unwrap();

    let response = block_on(routes.dispatch(Request::create(Method::GET, "/posts").unwrap()));

    assert_eq!(response.status(), 403);
}

#[test]
fn global_middleware_also_runs_on_a_404() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(Tag("global"));
    });
    let routes = router.compile().unwrap();

    let response = block_on(routes.dispatch(Request::create(Method::GET, "/missing").unwrap()));

    let tags: Vec<_> = response.headers().get_all("x-tag").iter().collect();
    assert_eq!(response.status(), 404);
    assert_eq!(tags, ["global"]);
}

#[test]
fn group_middleware_runs_only_on_its_routes() {
    let mut router = Router::new();
    router.get("/", index);
    router
        .prefix("/admin")
        .middleware(Tag("admin"))
        .group(|route| {
            route.get("/posts", index);
        });
    router.global_middleware(|middleware| {
        middleware.append(Tag("global"));
    });
    let routes = router.compile().unwrap();

    let outside = block_on(routes.dispatch(Request::create(Method::GET, "/").unwrap()));
    let inside = block_on(routes.dispatch(Request::create(Method::GET, "/admin/posts").unwrap()));

    let outside_tags: Vec<_> = outside.headers().get_all("x-tag").iter().collect();
    let inside_tags: Vec<_> = inside.headers().get_all("x-tag").iter().collect();
    assert_eq!(outside_tags, ["global"]);
    assert_eq!(inside_tags, ["admin", "global"]);
}

#[test]
fn middleware_runs_outermost_first_global_then_groups_then_the_route() {
    let mut router = Router::new();
    router
        .prefix("/admin")
        .middleware(Tag("admin"))
        .group(|route| {
            route.middleware(Tag("audit")).group(|route| {
                route.get("/users", index).middleware(Tag("route"));
            });
        });
    router.global_middleware(|middleware| {
        middleware.append(Tag("global"));
    });
    let routes = router.compile().unwrap();

    let response = block_on(routes.dispatch(Request::create(Method::GET, "/admin/users").unwrap()));

    let tags: Vec<_> = response.headers().get_all("x-tag").iter().collect();
    // Read back to front: global saw the request first, the route's own middleware last.
    assert_eq!(tags, ["route", "audit", "admin", "global"]);
}

#[test]
fn by_default_input_strings_are_trimmed_at_any_depth_except_passwords() {
    let mut router = Router::new();
    router.post("/echo", echo);
    let routes = router.compile().unwrap();
    let request = http::Request::builder()
        .method(Method::POST)
        .uri("/echo")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"title": "  Sabai  ", "tags": [" rust "], "user": {"name": " Test User ", "password": " secret "}, "password": " p "}"#,
        ))
        .unwrap();

    let response = block_on(routes.dispatch(Request::from(request)));

    let input: Value = serde_json::from_slice(response.body().as_bytes()).unwrap();
    assert_eq!(
        input,
        json!({"title": "Sabai", "tags": ["rust"], "user": {"name": "Test User", "password": " secret "}, "password": " p "})
    );
}

#[test]
fn by_default_blank_strings_become_null_after_trimming() {
    let mut router = Router::new();
    router.post("/echo", echo);
    let routes = router.compile().unwrap();
    let request = http::Request::builder()
        .method(Method::POST)
        .uri("/echo")
        .header("content-type", "application/json")
        .body(Body::from(
            r#"{"subtitle": "", "summary": "   ", "count": 0}"#,
        ))
        .unwrap();

    let response = block_on(routes.dispatch(Request::from(request)));

    let input: Value = serde_json::from_slice(response.body().as_bytes()).unwrap();
    assert_eq!(
        input,
        json!({"subtitle": null, "summary": null, "count": 0})
    );
}

#[test]
fn input_that_cannot_be_read_reaches_the_handler_unchanged() {
    let mut router = Router::new();
    router.post("/echo", echo);
    let routes = router.compile().unwrap();
    let request = http::Request::builder()
        .method(Method::POST)
        .uri("/echo")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"title": "#))
        .unwrap();

    let response = block_on(routes.dispatch(Request::from(request)));

    assert_eq!(response.status(), 400);
}

#[test]
fn a_removed_default_no_longer_runs() {
    let mut router = Router::new();
    router.post("/echo", echo);
    router.global_middleware(|middleware| {
        middleware.remove::<ConvertEmptyStringsToNull>();
    });
    let routes = router.compile().unwrap();
    let request = http::Request::builder()
        .method(Method::POST)
        .uri("/echo")
        .header("content-type", "application/json")
        .body(Body::from(r#"{"subtitle": "  "}"#))
        .unwrap();

    let response = block_on(routes.dispatch(Request::from(request)));

    let input: Value = serde_json::from_slice(response.body().as_bytes()).unwrap();
    assert_eq!(input, json!({"subtitle": ""}));
}

#[test]
fn frame_guard_allows_only_the_same_origin() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(FrameGuard);
    });
    let routes = router.compile().unwrap();

    let response = block_on(routes.dispatch(Request::create(Method::GET, "/posts").unwrap()));

    assert_eq!(response.headers()[X_FRAME_OPTIONS], "SAMEORIGIN");
}

#[test]
fn frame_guard_keeps_the_frame_options_a_route_set() {
    let mut router = Router::new();
    router.get("/never-framed", never_framed);
    router.global_middleware(|middleware| {
        middleware.append(FrameGuard);
    });
    let routes = router.compile().unwrap();

    let request = Request::create(Method::GET, "/never-framed").unwrap();
    let response = block_on(routes.dispatch(request));

    assert_eq!(response.headers()[X_FRAME_OPTIONS], "DENY");
}

#[test]
fn by_default_every_response_carries_its_own_request_id() {
    let mut router = Router::new();
    router.get("/posts", index);
    let routes = router.compile().unwrap();

    let first = block_on(routes.dispatch(Request::create(Method::GET, "/posts").unwrap()));
    let second = block_on(routes.dispatch(Request::create(Method::GET, "/posts").unwrap()));

    let first_id = first.headers()["x-request-id"].to_str().unwrap();
    assert_eq!(first_id.len(), 16, "{first_id}");
    assert_ne!(
        first.headers()["x-request-id"],
        second.headers()["x-request-id"]
    );
}

#[test]
fn a_plain_incoming_request_id_is_kept() {
    let mut router = Router::new();
    router.get("/posts", index);
    let routes = router.compile().unwrap();
    let request = http::Request::builder()
        .uri("/posts")
        .header("x-request-id", "lb-7f3a.1")
        .body(Body::empty())
        .unwrap();

    let response = block_on(routes.dispatch(Request::from(request)));

    assert_eq!(response.headers()["x-request-id"], "lb-7f3a.1");
}

#[test]
fn an_incoming_request_id_that_could_forge_log_lines_is_replaced() {
    let mut router = Router::new();
    router.get("/posts", index);
    let routes = router.compile().unwrap();
    let request = http::Request::builder()
        .uri("/posts")
        .header("x-request-id", "x\" forged=1")
        .body(Body::empty())
        .unwrap();

    let response = block_on(routes.dispatch(Request::from(request)));

    assert_ne!(response.headers()["x-request-id"], "x\" forged=1");
}
