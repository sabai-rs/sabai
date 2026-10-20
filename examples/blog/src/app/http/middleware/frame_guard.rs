use sabai::header::{HeaderValue, X_FRAME_OPTIONS};
use sabai::http::{Middleware, Next};
use sabai::{Request, Response};

/// Lets only the blog's own pages show it in a frame, against clickjacking, like Laravel's `FrameGuard`.
/// A route that sets its own `X-Frame-Options` keeps it.
pub struct FrameGuard;

impl Middleware for FrameGuard {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let mut response = next.run(request).await;
        let same_origin = HeaderValue::from_static("SAMEORIGIN");
        response
            .headers_mut()
            .entry(X_FRAME_OPTIONS)
            .or_insert(same_origin);
        response
    }
}
