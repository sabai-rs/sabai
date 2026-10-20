mod input;

use std::sync::{Arc, OnceLock};

use http::{HeaderMap, Method, StatusCode, Uri};
use serde_json::{Map, Value};

use crate::{App, Body, Error, Result};

use input::{Fields, InputError};

/// An incoming HTTP request, with Laravel-style helpers on top of [`http::Request`].
/// Cloning it is cheap, so a handler can take `request: Request` alongside other arguments.
#[derive(Clone, Debug)]
pub struct Request {
    inner: Arc<RequestInner>,
}

// Behind one `Arc`: a clone costs one counter increment instead of copying every header.
#[derive(Clone, Debug)]
struct RequestInner {
    http: http::Request<Body>,
    route_params: Vec<(String, String)>,
    app: Option<Arc<App>>,
    // Read on first use, so a route that never reads input never parses the body.
    input: OnceLock<Result<Fields, InputError>>,
}

impl Request {
    /// The HTTP method, such as `GET` or `POST`.
    pub fn method(&self) -> &Method {
        self.inner.http.method()
    }

    /// The full URI, including the query string.
    pub fn uri(&self) -> &Uri {
        self.inner.http.uri()
    }

    /// The path without the query string, such as `/posts/1`.
    pub fn path(&self) -> &str {
        self.inner.http.uri().path()
    }

    /// The raw query string without the `?`, if there is one.
    pub fn query_string(&self) -> Option<&str> {
        self.inner.http.uri().query()
    }

    /// All request headers.
    pub fn headers(&self) -> &HeaderMap {
        self.inner.http.headers()
    }

    /// One header as text; `None` when it is missing or not valid UTF-8.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.inner.http.headers().get(name)?.to_str().ok()
    }

    /// Every input field, like Laravel's `$request->all()`: the query string merged with the JSON
    /// or form body, the body winning. The body is parsed once, on first use, and shared by clones.
    pub fn all(&self) -> Result<&Map<String, Value>> {
        let input = self.inner.input.get_or_init(|| input::read(self));
        input.as_ref().map_err(Error::from)
    }

    /// One input field, like `$request->input('title')`; `None` when it is missing
    /// or the body cannot be read.
    pub fn input(&self, name: &str) -> Option<&Value> {
        self.all().ok()?.get(name)
    }

    /// Replaces the input, for middleware that cleans it, like Laravel's `TrimStrings`.
    pub fn with_input(mut self, fields: Map<String, Value>) -> Self {
        self.inner_mut().input = OnceLock::from(Ok(fields));
        self
    }

    /// Whether the body is JSON by its `Content-Type`, like Laravel's `$request->isJson()`.
    pub fn is_json(&self) -> bool {
        let content_type = input::content_type(self);
        content_type.ends_with("/json") || content_type.ends_with("+json")
    }

    /// A parameter from the matched route, like Laravel's `$request->route('id')`:
    /// `{id}` in `/posts/{id}` gives `param("id") == Some("7")` for `/posts/7`.
    pub fn param(&self, name: &str) -> Option<&str> {
        self.inner
            .route_params
            .iter()
            .find(|(param, _)| param == name)
            .map(|(_, value)| value.as_str())
    }

    /// Every parameter of the matched route, in path order, as `(name, value)` pairs.
    pub fn params(&self) -> &[(String, String)] {
        &self.inner.route_params
    }

    /// Attaches the matched route's parameters; the router calls this before the handler runs.
    pub fn with_route_params(mut self, params: Vec<(String, String)>) -> Self {
        self.inner_mut().route_params = params;
        self
    }

    /// The app serving this request, like Laravel's `app()`: `request.app()?.get::<dyn Clock>()`.
    pub fn app(&self) -> Result<&Arc<App>> {
        self.inner.app.as_ref().ok_or_else(|| {
            Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(
                "this request has no app because it was built by hand; \
                 attach one with `request.with_app(app)`",
            )
        })
    }

    /// Attaches the app; the server calls this for every request.
    pub fn with_app(mut self, app: Arc<App>) -> Self {
        self.inner_mut().app = Some(app);
        self
    }

    /// The request body.
    pub fn body(&self) -> &Body {
        self.inner.http.body()
    }

    /// The underlying `http::Request`, for anything the helpers above do not cover.
    pub fn into_http(self) -> http::Request<Body> {
        match Arc::try_unwrap(self.inner) {
            Ok(inner) => inner.http,
            Err(inner) => inner.http.clone(),
        }
    }

    // Copies the request only if a clone exists; the server, router and input middleware call it
    // before handlers take their clones, so it usually does not.
    fn inner_mut(&mut self) -> &mut RequestInner {
        Arc::make_mut(&mut self.inner)
    }
}

impl From<http::Request<Body>> for Request {
    fn from(http: http::Request<Body>) -> Self {
        let inner = RequestInner {
            http,
            route_params: Vec::new(),
            app: None,
            input: OnceLock::new(),
        };
        Self {
            inner: Arc::new(inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(uri: &str) -> Request {
        let inner = http::Request::builder()
            .method(Method::POST)
            .uri(uri)
            .header("Authorization", "Bearer secret-token")
            .body(Body::from("title=Hello"))
            .unwrap();
        Request::from(inner)
    }

    #[test]
    fn path_and_query_string_are_split() {
        let request = request("/posts?page=2&sort=new");

        assert_eq!(request.method(), Method::POST);
        assert_eq!(request.path(), "/posts");
        assert_eq!(request.query_string(), Some("page=2&sort=new"));
    }

    #[test]
    fn query_string_is_none_without_a_question_mark() {
        assert_eq!(request("/posts").query_string(), None);
    }

    #[test]
    fn header_lookup_ignores_case() {
        let request = request("/posts");

        assert_eq!(request.header("authorization"), Some("Bearer secret-token"));
        assert_eq!(request.header("x-missing"), None);
    }

    #[test]
    fn a_clone_shares_the_request_and_a_later_change_copies_it() {
        let original = request("/posts/7");

        let clone = original.clone();
        let shared = Arc::ptr_eq(&original.inner, &clone.inner);
        let changed = clone.with_route_params(vec![("id".into(), "7".into())]);

        assert!(shared);
        assert_eq!(changed.param("id"), Some("7"));
        assert_eq!(original.param("id"), None);
    }

    fn sent(method: Method, uri: &str, content_type: &str, body: &'static str) -> Request {
        let inner = http::Request::builder()
            .method(method)
            .uri(uri)
            .header("Content-Type", content_type)
            .body(Body::from(body))
            .unwrap();
        Request::from(inner)
    }

    #[test]
    fn input_merges_the_query_string_with_the_body_and_the_body_wins() {
        let json = r#"{"title": "From the body", "draft": true}"#;

        let request = sent(
            Method::POST,
            "/posts?title=Query&page=2",
            "application/json",
            json,
        );

        assert_eq!(request.input("title"), Some(&Value::from("From the body")));
        assert_eq!(request.input("draft"), Some(&Value::Bool(true)));
        assert_eq!(request.input("page"), Some(&Value::from("2")));
        assert_eq!(request.input("missing"), None);
    }

    #[test]
    fn a_get_reads_only_the_query_string() {
        let form = "application/x-www-form-urlencoded";

        let request = sent(Method::GET, "/posts?page=2", form, "title=Ignored");

        assert_eq!(request.all().unwrap().len(), 1);
        assert_eq!(request.input("page"), Some(&Value::from("2")));
    }

    #[test]
    fn a_body_that_cannot_be_read_is_an_error_from_all_and_none_from_input() {
        let broken = sent(Method::POST, "/posts", "application/json", r#"{"title":"#);
        let listed = sent(Method::POST, "/posts", "application/json", "[1, 2]");
        let text = sent(Method::POST, "/posts", "text/plain", "title=Sabai");

        assert_eq!(broken.all().unwrap_err().status(), StatusCode::BAD_REQUEST);
        assert_eq!(broken.input("title"), None);
        let unprocessable = StatusCode::UNPROCESSABLE_ENTITY;
        assert_eq!(listed.all().unwrap_err().status(), unprocessable);
        assert_eq!(text.all().unwrap_err().status(), unprocessable);
    }

    #[test]
    fn the_body_is_parsed_once_and_shared_by_clones() {
        let request = sent(Method::POST, "/posts", "application/json", r#"{"a": 1}"#);
        let clone = request.clone();

        request.all().unwrap();

        assert!(clone.inner.input.get().is_some());
    }

    #[test]
    fn with_input_replaces_what_input_reads() {
        let request = sent(
            Method::POST,
            "/posts",
            "application/json",
            r#"{"title": " a "}"#,
        );
        let cleaned = Map::from_iter([("title".to_owned(), Value::from("a"))]);

        let request = request.with_input(cleaned);

        assert_eq!(request.input("title"), Some(&Value::from("a")));
    }

    #[test]
    fn is_json_reads_the_media_type_without_its_parameters() {
        let json = |content_type| sent(Method::POST, "/", content_type, "").is_json();

        assert!(json("application/json; charset=utf-8"));
        assert!(json("application/vnd.api+json"));
        assert!(!json("text/plain"));
    }

    #[test]
    fn the_body_and_the_http_request_stay_reachable() {
        let request = request("/posts");

        assert_eq!(request.body().as_bytes(), b"title=Hello");
        assert_eq!(request.into_http().uri(), "/posts");
    }
}
