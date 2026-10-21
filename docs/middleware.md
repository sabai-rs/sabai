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

## Global Middleware

Global middleware runs on every request, `404` and `405` responses included. Every app runs Sabai's defaults without asking:

| Default | Does |
| --- | --- |
| `TrimStrings` | trims spaces around input strings ([more](/requests#input-trimming-and-normalization)) |
| `ConvertEmptyStringsToNull` | turns `""` input into `null` |

Change the list where `src/lib.rs` returns your routes. `append` runs a middleware after the defaults, and `remove` stops one:

```rust
use sabai::http::middleware::{ConvertEmptyStringsToNull, FrameGuard};

pub fn routes() -> Router {
    let mut route = routes::all();
    route.global_middleware(|middleware| {
        middleware.append(FrameGuard);
        middleware.remove::<ConvertEmptyStringsToNull>();
    });
    route
}
```

::: laravel
`src/lib.rs` is `bootstrap/app.php`: `routes()` is `withRouting` and `withMiddleware` together.

```php
->withMiddleware(function (Middleware $middleware) {
    $middleware->append(FrameGuard::class);
    $middleware->remove(ConvertEmptyStringsToNull::class);
})
```
:::

### Middleware Sabai Ships

| Middleware | Does |
| --- | --- |
| `FrameGuard` | sends `X-Frame-Options: SAMEORIGIN` unless the route set its own, against clickjacking |

They live in `sabai::http::middleware`, like `Illuminate\Http\Middleware`.

## Assigning Middleware to Routes

Add `.middleware(...)` to one route, or start a group with it:

```rust
route.get("/profile", profile).middleware(Auth);

route.middleware(Auth).group(|route| {
    route.get("/settings", settings);
});

route.prefix("/admin").middleware(Auth).middleware(CheckAdmin).group(|route| {
    route.get("/posts", admin_posts);
});
```

The blog keeps API answers out of caches this way: `route.prefix("/api").middleware(NoStore).group(api::routes)`.

::: laravel
```php
Route::get('/profile', ProfileController::class)->middleware(Auth::class);

Route::prefix('admin')->middleware([Auth::class, CheckAdmin::class])->group(function () {
    Route::get('/posts', [AdminPostController::class, 'index']);
});
```
:::

## Order

Middleware runs from the outside in: global first, then each group from the outermost, then the route's own, each in the order added. The first one sees the request first and the response last.
