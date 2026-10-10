use std::future::Future;
use std::marker::PhantomData;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{IntoResponse, Request, Response};

/// An async function the router can call, such as `async fn index() -> impl IntoResponse`.
/// `Args` lists its extractor types, so functions with different arguments are different handlers.
pub trait Handler<Args>: Clone + Send + Sync + 'static {
    /// Runs the handler for one request.
    fn call(&self, request: Request) -> BoxFuture<'static, Response>;
}

impl<F, Fut, R> Handler<()> for F
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + 'static,
{
    fn call(&self, _request: Request) -> BoxFuture<'static, Response> {
        respond_with(Box::pin(self()))
    }
}

impl<F, Fut, R> Handler<(Request,)> for F
where
    F: Fn(Request) -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoResponse + 'static,
{
    fn call(&self, request: Request) -> BoxFuture<'static, Response> {
        respond_with(Box::pin(self(request)))
    }
}

// Generic over `R` only, so handlers with the same return type share this code.
fn respond_with<R: IntoResponse + 'static>(
    future: BoxFuture<'static, R>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move { future.await.into_response() })
}

/// A handler with its argument types erased, so routes with different handlers fit in one list.
pub(crate) trait ErasedHandler: Send + Sync {
    fn call_erased(&self, request: Request) -> BoxFuture<'static, Response>;
}

pub(crate) type BoxedHandler = Arc<dyn ErasedHandler>;

// A struct, not a boxed closure, which would add three shims to every route.
struct Erased<H, Args> {
    handler: H,
    _args: PhantomData<fn() -> Args>,
}

impl<H: Handler<Args>, Args: 'static> ErasedHandler for Erased<H, Args> {
    fn call_erased(&self, request: Request) -> BoxFuture<'static, Response> {
        self.handler.call(request)
    }
}

pub(crate) fn boxed<H: Handler<Args>, Args: 'static>(handler: H) -> BoxedHandler {
    Arc::new(Erased {
        handler,
        _args: PhantomData,
    })
}
