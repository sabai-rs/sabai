# Routing

## Basic Routing

Routes live in `src/routes/`, one file per group like Laravel's `routes/web.php` and `routes/api.php`. Each route is one statement:

```rust
use sabai::http::Router;

pub fn routes(route: &mut Router) {
    route.get("/", home);
}

async fn home() -> &'static str {
    "Hello, Sabai"
}
```

A handler is any `async fn` that returns something that can become a response: a string, `Json(value)`, `(StatusCode, T)` or a `Result` of those.

### Available Router Methods

```rust
route.get(path, handler);
route.post(path, handler);
route.put(path, handler);
route.patch(path, handler);
route.delete(path, handler);
```

A path with the wrong method answers `405`, and an unknown path answers `404`, both as JSON.

::: laravel
```php
Route::get('/', fn () => 'Hello, Laravel');
```
:::

## Route Parameters

Wrap a parameter in braces, and take it as `Param<T>`:

```rust
route.get("/posts/{id}", show);

async fn show(Param(id): Param<u64>) -> Result<Json<Post>> {
    Ok(Json(Post::find_or_fail(id).await?))
}
```

A value that does not parse as `T` answers `404`, like Laravel's `whereNumber('id')`, so `/posts/abc` never reaches the handler.

`Param` reads a route with exactly one parameter. For routes with several, take the `Request` and read them by name: `request.param("comment")`.

### Wildcards

`{*path}` matches the rest of the path, slashes included:

```rust
route.get("/files/{*path}", file);
```

## Named Routes

Name a route to build its URL later:

```rust
route.get("/posts/{id}", PostController::show).name("posts.show");
```

Parameters fill the path and any others become the query string, all percent-encoded. In feature tests:

```rust
app().get_json(route("posts.show").with("id", 1));
app().get_json(route("posts.index").with("page", 2));   // /posts?page=2
```

Two routes with one name stop the app at startup.

## Route Groups

`prefix` puts every route of a group under a path, like `Route::prefix('api')->group(...)`:

```rust
route.prefix("/api").group(|route| {
    route.get("/health", health);
});
```

A group can take a whole routes file, which is how the blog puts `routes/api.rs` under `/api`:

```rust
route.prefix("/api").group(api::routes);
```

Groups nest, and `prefix("api")`, `prefix("/api")` and `prefix("/api/")` mean the same.

::: laravel
```php
Route::prefix('api')->group(base_path('routes/api.php'));
```
:::

## Resource Routes

`api_resource` routes a controller's REST actions with Laravel's paths and names. Declare only the actions the controller has, which is Laravel's `->only([...])`:

```rust
route.api_resource("posts", |posts| {
    posts.index(PostController::index);
    posts.store(PostController::store);
    posts.show(PostController::show);
    posts.update(PostController::update);
    posts.destroy(PostController::destroy);
});
```

| Action | Method | Path | Name |
| --- | --- | --- | --- |
| `index` | `GET` | `/posts` | `posts.index` |
| `store` | `POST` | `/posts` | `posts.store` |
| `show` | `GET` | `/posts/{id}` | `posts.show` |
| `update` | `PUT`, `PATCH` | `/posts/{id}` | `posts.update` |
| `destroy` | `DELETE` | `/posts/{id}` | `posts.destroy` |

The parameter is `{id}`, not Laravel's `{post}`, since there is no route model binding yet.

::: laravel
```php
Route::apiResource('posts', PostController::class)->only(['index', 'store', 'show']);
```
:::

## Mistakes Are Found at Startup

Routes are checked when the app starts, before it listens. A duplicate route, a duplicate name, a malformed path, or a handler that needs a service nobody bound, all stop `serve` with a message that names the route:

```text
Error: route `GET /api/health` cannot start: no service is bound to `dyn Clock`;
bind it in a provider's `register`: `app.bind::<dyn Clock>(Arc::new(...))`
```
