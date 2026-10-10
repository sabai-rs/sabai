use std::sync::Arc;

use http::{HeaderMap, Method, StatusCode, Uri};

use crate::{App, Body, Error, Result};

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

    // Only the server and router call this, before any clone exists, so it never copies.
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

    #[test]
    fn the_body_and_the_http_request_stay_reachable() {
        let request = request("/posts");

        assert_eq!(request.body().as_bytes(), b"title=Hello");
        assert_eq!(request.into_http().uri(), "/posts");
    }
}
