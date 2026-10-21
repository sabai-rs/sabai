//! Middleware, and the ones Sabai ships, like Laravel's `Illuminate\Http\Middleware`.

mod convert_empty_strings_to_null;
mod frame_guard;
mod global_middleware;
mod transforms_request;
mod trim_strings;

use std::future::Future;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Request, Response};

use crate::handler::BoxedHandler;

pub use convert_empty_strings_to_null::ConvertEmptyStringsToNull;
pub use frame_guard::FrameGuard;
pub use global_middleware::GlobalMiddleware;
pub use trim_strings::TrimStrings;

/// Code that runs around a request, like a Laravel middleware's `handle($request, $next)`.
/// Call `next.run(request).await` to go on, or return a response without calling it to stop there.
/// Implement it with `async fn handle(&self, request: Request, next: Next) -> Response`.
pub trait Middleware: Send + Sync + 'static {
    /// Handles `request`, usually by passing it to `next` and changing what comes back.
    fn handle(&self, request: Request, next: Next) -> impl Future<Output = Response> + Send;
}

/// A middleware with its future type erased, so different middleware fit in one list.
pub(crate) trait ErasedMiddleware: Send + Sync {
    fn call(self: Arc<Self>, request: Request, next: Next) -> BoxFuture<'static, Response>;
}

impl<M: Middleware> ErasedMiddleware for M {
    fn call(self: Arc<Self>, request: Request, next: Next) -> BoxFuture<'static, Response> {
        Box::pin(async move { self.handle(request, next).await })
    }
}

pub(crate) type BoxedMiddleware = Arc<dyn ErasedMiddleware>;

/// A route's middleware in order, then its handler.
#[derive(Clone)]
pub(crate) struct Pipeline {
    middleware: Arc<[BoxedMiddleware]>,
    handler: BoxedHandler,
}

/// The rest of the pipeline, given to [`Middleware::handle`].
pub struct Next {
    pipeline: Pipeline,
    position: usize,
}

impl Pipeline {
    pub(crate) fn new(middleware: Arc<[BoxedMiddleware]>, handler: BoxedHandler) -> Self {
        Self {
            middleware,
            handler,
        }
    }

    pub(crate) fn handler(&self) -> &BoxedHandler {
        &self.handler
    }

    pub(crate) fn run(self, request: Request) -> BoxFuture<'static, Response> {
        Next {
            pipeline: self,
            position: 0,
        }
        .run(request)
    }
}

impl Next {
    /// Passes `request` to the next middleware, or to the handler after the last one.
    pub fn run(self, request: Request) -> BoxFuture<'static, Response> {
        let Some(middleware) = self.pipeline.middleware.get(self.position).cloned() else {
            return self.pipeline.handler.call_erased(request);
        };
        let next = Next {
            position: self.position + 1,
            pipeline: self.pipeline,
        };
        middleware.call(request, next)
    }
}
