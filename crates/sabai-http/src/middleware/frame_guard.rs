use std::future::Future;

use http::header::{HeaderValue, X_FRAME_OPTIONS};
use sabai_core::{Request, Response};

use crate::{Middleware, Next};

/// Lets only your own pages show the app in a frame, against clickjacking, like Laravel's `FrameGuard`.
/// A route that sets its own `X-Frame-Options` keeps it.
pub struct FrameGuard;

impl Middleware for FrameGuard {
    // Boxed here so the body compiles once in Sabai, not into every app that appends it.
    fn handle(&self, request: Request, next: Next) -> impl Future<Output = Response> + Send {
        Box::pin(frame_same_origin(request, next))
    }
}

async fn frame_same_origin(request: Request, next: Next) -> Response {
    let mut response = next.run(request).await;
    let same_origin = HeaderValue::from_static("SAMEORIGIN");
    response
        .headers_mut()
        .entry(X_FRAME_OPTIONS)
        .or_insert(same_origin);
    response
}
