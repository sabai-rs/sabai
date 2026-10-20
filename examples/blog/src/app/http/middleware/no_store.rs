use sabai::header::{CACHE_CONTROL, HeaderValue};
use sabai::http::{Middleware, Next};
use sabai::{Request, Response};

/// Keeps browsers and proxies from storing API answers, which change with every new post.
/// A route that sets its own `Cache-Control` keeps it.
pub struct NoStore;

impl Middleware for NoStore {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let mut response = next.run(request).await;
        let no_store = HeaderValue::from_static("no-store");
        response
            .headers_mut()
            .entry(CACHE_CONTROL)
            .or_insert(no_store);
        response
    }
}
