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

fn send(router: Router, path: &str) -> Response {
    let request = http::Request::builder()
        .method(Method::GET)
        .uri(path)
        .body(Body::empty())
        .unwrap();
    block_on(router.compile().unwrap().dispatch(Request::from(request)))
}

fn tags(response: &Response) -> Vec<&str> {
    let tags = response.headers().get_all("x-tag").iter();
    tags.map(|tag| tag.to_str().unwrap()).collect()
}

#[test]
fn global_middleware_runs_around_the_handler() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(Tag("first")).append(Tag("second"));
    });

    let response = send(router, "/posts");

    assert_eq!(response.status(), 200);
    // The last one added is innermost, so it changes the response first.
    assert_eq!(tags(&response), ["second", "first"]);
}

#[test]
fn middleware_can_answer_without_running_the_handler() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(Deny);
    });

    assert_eq!(send(router, "/posts").status(), 403);
}

#[test]
fn global_middleware_also_runs_on_404_and_405() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.global_middleware(|middleware| {
        middleware.append(Tag("global"));
    });

    let response = send(router, "/missing");

    assert_eq!(response.status(), 404);
    assert_eq!(tags(&response), ["global"]);
}

/// Global, then the `/admin` group, then a group nested in it, then one route.
fn site_with_admin_group() -> Router {
    let mut router = Router::new();
    router.get("/", index);
    router
        .prefix("/admin")
        .middleware(Tag("admin"))
        .group(|route| {
            route.get("/posts", index);
            route.middleware(Tag("audit")).group(|route| {
                route.get("/users", index).middleware(Tag("route"));
            });
        });
    router.global_middleware(|middleware| {
        middleware.append(Tag("global"));
    });
    router
}

#[test]
fn group_middleware_runs_only_on_its_routes() {
    let outside = send(site_with_admin_group(), "/");
    let inside = send(site_with_admin_group(), "/admin/posts");

    assert_eq!(tags(&outside), ["global"]);
    assert_eq!(tags(&inside), ["admin", "global"]);
}

#[test]
fn middleware_runs_outermost_first_global_then_groups_then_the_route() {
    let response = send(site_with_admin_group(), "/admin/users");

    // Read back to front: global saw the request first, the route's own middleware last.
    assert_eq!(tags(&response), ["route", "audit", "admin", "global"]);
}

async fn echo(request: Request) -> Result<Json<Value>> {
    Ok(Json(Value::Object(request.all()?.clone())))
}

fn post_json(router: Router, body: &'static str) -> Response {
    let request = http::Request::builder()
        .method(Method::POST)
        .uri("/echo")
        .header("content-type", "application/json")
        .body(Body::from(body))
        .unwrap();
    block_on(router.compile().unwrap().dispatch(Request::from(request)))
}

fn echo_router() -> Router {
    let mut router = Router::new();
    router.post("/echo", echo);
    router
}

fn json_of(response: Response) -> Value {
    serde_json::from_slice(response.body().as_bytes()).unwrap()
}

#[test]
fn by_default_input_strings_are_trimmed_at_any_depth_except_passwords() {
    let body = r#"{"title": "  Sabai  ", "tags": [" rust "], "user": {"name": " Sang ", "password": " secret "}, "password": " p "}"#;

    let input = json_of(post_json(echo_router(), body));

    assert_eq!(
        input,
        json!({"title": "Sabai", "tags": ["rust"], "user": {"name": "Sang", "password": " secret "}, "password": " p "})
    );
}

#[test]
fn by_default_blank_strings_become_null_after_trimming() {
    let body = r#"{"subtitle": "", "summary": "   ", "count": 0}"#;

    let input = json_of(post_json(echo_router(), body));

    assert_eq!(
        input,
        json!({"subtitle": null, "summary": null, "count": 0})
    );
}

#[test]
fn input_that_cannot_be_read_reaches_the_handler_unchanged() {
    let response = post_json(echo_router(), r#"{"title": "#);

    assert_eq!(response.status(), 400);
}

#[test]
fn a_removed_default_no_longer_runs() {
    let mut router = echo_router();
    router.global_middleware(|middleware| {
        middleware.remove::<ConvertEmptyStringsToNull>();
    });

    let input = json_of(post_json(router, r#"{"subtitle": "  "}"#));

    assert_eq!(input, json!({"subtitle": ""}));
}

async fn never_framed() -> Response {
    let mut response = "Never framed".into_response();
    let deny = HeaderValue::from_static("DENY");
    response.headers_mut().insert(X_FRAME_OPTIONS, deny);
    response
}

#[test]
fn frame_guard_allows_only_the_same_origin_unless_the_route_says_otherwise() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.get("/never-framed", never_framed);
    router.global_middleware(|middleware| {
        middleware.append(FrameGuard);
    });
    let routes = router.compile().unwrap();
    let frame_options = |path| {
        let request = http::Request::builder()
            .uri(path)
            .body(Body::empty())
            .unwrap();
        let response = block_on(routes.dispatch(Request::from(request)));
        response.headers()[X_FRAME_OPTIONS].clone()
    };

    assert_eq!(frame_options("/posts"), "SAMEORIGIN");
    assert_eq!(frame_options("/never-framed"), "DENY");
}
