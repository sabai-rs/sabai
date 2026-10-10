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

// Generic over the return type only, so every handler returning the same type shares one
// copy of this state machine instead of each route compiling its own (LLVM lines per route).
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

// A struct rather than a closure: the vtable then points straight at `call_erased`,
// without the closure, `FnOnce` and vtable shims a boxed closure adds to every route.
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
