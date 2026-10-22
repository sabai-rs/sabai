use std::future::{Future, ready};
use std::marker::PhantomData;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, IntoResponse, Request, Response, Result};

/// A type a handler can take as an argument, built from the request before the handler runs,
/// like a type-hinted argument in a Laravel controller. Failing rejects the request with that error.
pub trait FromRequest: Sized + Send + 'static {
    /// Builds the argument from `request`.
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>>;

    /// Checks once, before the server accepts requests, that the argument can be built,
    /// so a missing service stops startup instead of failing requests.
    fn prepare(_app: &App) -> Result<()> {
        Ok(())
    }
}

impl FromRequest for Request {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(Ok(request.clone())))
    }
}

/// An async function the router can call, such as `async fn index() -> impl IntoResponse`.
/// `Args` lists its argument types, so functions with different arguments are different handlers.
pub trait Handler<Args>: Clone + Send + Sync + 'static {
    /// Runs the handler for one request.
    fn call(&self, request: Request) -> BoxFuture<'static, Response>;

    /// Prepares every argument once at startup; see [`FromRequest::prepare`].
    fn prepare(_app: &App) -> Result<()> {
        Ok(())
    }
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

macro_rules! impl_handler {
    ($($arg:ident),+) => {
        impl<F, Fut, R, $($arg),+> Handler<($($arg,)+)> for F
        where
            F: Fn($($arg),+) -> Fut + Clone + Send + Sync + 'static,
            Fut: Future<Output = R> + Send + 'static,
            R: IntoResponse + 'static,
            $($arg: FromRequest),+
        {
            fn prepare(app: &App) -> Result<()> {
                $($arg::prepare(app)?;)+
                Ok(())
            }

            #[allow(non_snake_case)]
            fn call(&self, request: Request) -> BoxFuture<'static, Response> {
                let handler = self.clone();
                Box::pin(async move {
                    $(
                        let $arg = match $arg::from_request(&request).await {
                            Ok(argument) => argument,
                            Err(error) => return error.into_response(),
                        };
                    )+
                    handler($($arg),+).await.into_response()
                })
            }
        }
    };
}

impl_handler!(A1);
impl_handler!(A1, A2);
impl_handler!(A1, A2, A3);
impl_handler!(A1, A2, A3, A4);
impl_handler!(A1, A2, A3, A4, A5);
impl_handler!(A1, A2, A3, A4, A5, A6);
impl_handler!(A1, A2, A3, A4, A5, A6, A7);
impl_handler!(A1, A2, A3, A4, A5, A6, A7, A8);

// Generic over `R` only, so handlers with the same return type share this code.
fn respond_with<R: IntoResponse + 'static>(
    future: BoxFuture<'static, R>,
) -> BoxFuture<'static, Response> {
    Box::pin(async move { future.await.into_response() })
}

/// A handler with its argument types erased, so routes with different handlers fit in one list.
pub(crate) trait ErasedHandler: Send + Sync {
    fn call_erased(&self, request: Request) -> BoxFuture<'static, Response>;
    fn prepare(&self, app: &App) -> Result<()>;
    /// The handler's Rust path, such as `bootstrap::app::http::controllers::post_controller::PostController::index`.
    fn action(&self) -> &'static str;
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

    fn prepare(&self, app: &App) -> Result<()> {
        H::prepare(app)
    }

    fn action(&self) -> &'static str {
        std::any::type_name::<H>()
    }
}

pub(crate) fn boxed<H: Handler<Args>, Args: 'static>(handler: H) -> BoxedHandler {
    Arc::new(Erased {
        handler,
        _args: PhantomData,
    })
}
