use http::{HeaderMap, Method, Uri};

use crate::Body;

/// An incoming HTTP request, with Laravel-style helpers on top of [`http::Request`].
#[derive(Debug)]
pub struct Request {
    inner: http::Request<Body>,
}

impl Request {
    /// The HTTP method, such as `GET` or `POST`.
    pub fn method(&self) -> &Method {
        self.inner.method()
    }

    /// The full URI, including the query string.
    pub fn uri(&self) -> &Uri {
        self.inner.uri()
    }

    /// The path without the query string, such as `/posts/1`.
    pub fn path(&self) -> &str {
        self.inner.uri().path()
    }

    /// The raw query string without the `?`, if there is one.
    pub fn query_string(&self) -> Option<&str> {
        self.inner.uri().query()
    }

    /// All request headers.
    pub fn headers(&self) -> &HeaderMap {
        self.inner.headers()
    }

    /// One header as text; `None` when it is missing or not valid UTF-8.
    pub fn header(&self, name: &str) -> Option<&str> {
        self.inner.headers().get(name)?.to_str().ok()
    }

    /// The request body.
    pub fn body(&self) -> &Body {
        self.inner.body()
    }

    /// The underlying `http::Request`, for anything the helpers above do not cover.
    pub fn into_http(self) -> http::Request<Body> {
        self.inner
    }
}

impl From<http::Request<Body>> for Request {
    fn from(inner: http::Request<Body>) -> Self {
        Self { inner }
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
    fn the_body_and_the_http_request_stay_reachable() {
        let request = request("/posts");

        assert_eq!(request.body().as_bytes(), b"title=Hello");
        assert_eq!(request.into_http().uri(), "/posts");
    }
}
