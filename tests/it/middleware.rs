use pollster::block_on;
use sabai::http::{Middleware, Next, Router};
use sabai::{Body, Error, IntoResponse, Method, Request, Response};

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
fn middleware_runs_around_the_handler() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.middleware(Tag("first")).middleware(Tag("second"));

    let response = send(router, "/posts");

    assert_eq!(response.status(), 200);
    // The last one added is innermost, so it changes the response first.
    assert_eq!(tags(&response), ["second", "first"]);
}

#[test]
fn middleware_can_answer_without_running_the_handler() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.middleware(Deny);

    assert_eq!(send(router, "/posts").status(), 403);
}

#[test]
fn the_top_router_middleware_also_runs_on_404_and_405() {
    let mut router = Router::new();
    router.get("/posts", index);
    router.middleware(Tag("global"));

    let response = send(router, "/missing");

    assert_eq!(response.status(), 404);
    assert_eq!(tags(&response), ["global"]);
}

fn site_with_admin_group() -> Router {
    let mut router = Router::new();
    router.get("/", index);
    router.prefix("/admin").group(|route| {
        route.get("/posts", index);
        route.middleware(Tag("admin"));
    });
    router.middleware(Tag("global"));
    router
}

#[test]
fn group_middleware_runs_only_on_the_group_inside_the_outer_middleware() {
    let inside = send(site_with_admin_group(), "/admin/posts");
    let outside = send(site_with_admin_group(), "/");

    assert_eq!(tags(&inside), ["admin", "global"]);
    assert_eq!(tags(&outside), ["global"]);
}
