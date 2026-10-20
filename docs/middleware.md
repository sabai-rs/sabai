# Middleware

Middleware runs code around a request: before the handler, after it, or instead of it.

## Writing Middleware

A middleware is a type that implements `Middleware`. `next.run(request).await` runs the rest of the app and gives back its response:

```rust
use sabai::header::{HeaderValue, X_FRAME_OPTIONS};
use sabai::http::{Middleware, Next};
use sabai::{Request, Response};

pub struct FrameGuard;

impl Middleware for FrameGuard {
    async fn handle(&self, request: Request, next: Next) -> Response {
        let mut response = next.run(request).await;
        let same_origin = HeaderValue::from_static("SAMEORIGIN");
        response.headers_mut().insert(X_FRAME_OPTIONS, same_origin);
        response
    }
}
```

To stop a request, return a response without calling `next`:

```rust
async fn handle(&self, request: Request, next: Next) -> Response {
    if request.header("x-api-key") != Some("secret") {
        return Error::forbidden().into_response();
    }
    next.run(request).await
}
```

`next.run` takes `next` by value, so it can run at most once.

::: laravel
`async fn handle(&self, request, next) -> Response` is `handle(Request $request, Closure $next): Response`, and `next.run(request)` is `$next($request)`. The blog keeps its middleware in `src/app/http/middleware/`, like `app/Http/Middleware`.
:::

## Registering Middleware

`route.middleware(...)` runs a middleware around every route of that router:

```rust
pub fn all() -> Router {
    let mut route = Router::new();
    web::routes(&mut route);
    route.prefix("/api").group(api::routes);
    route.middleware(FrameGuard);
    route
}
```

On the router your app's `routes()` returns, middleware also runs on `404` and `405` responses.

::: laravel
That is a global middleware: `$middleware->append(FrameGuard::class)` in `bootstrap/app.php`.
:::

Middleware runs in the order it was added: the first one sees the request first and the response last.
