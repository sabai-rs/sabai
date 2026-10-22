# Routing

## Basic Routing

Routes live in `src/routes/`. One file per group, like `routes/web.php` and `routes/api.php`.

```rust
use sabai::http::Router;

pub fn routes(route: &mut Router) {
    route.get("/", home);
}

async fn home() -> &'static str {
    "Hello, Sabai"
}
```

A handler is an `async fn`. It can return a string, `Json(value)`, `(StatusCode, value)`, or a `Result` of those.

::: laravel
```php
Route::get('/', fn () => 'Hello, Laravel');
```
:::

### Router Methods

```rust
route.get(path, handler);
route.post(path, handler);
route.put(path, handler);
route.patch(path, handler);
route.delete(path, handler);
```

Every `get` route also answers `HEAD`, with the same headers and no body.

An unknown path answers `404`. A known path with the wrong method answers `405`, naming the methods it takes in the message and in the `Allow` header:

```text
HTTP/1.1 405 Method Not Allowed
allow: GET, HEAD, DELETE

{"message":"The POST method is not supported for route posts/7. Supported methods: GET, HEAD, DELETE."}
```

An `OPTIONS` request to a known path answers `200` with `allow: GET,HEAD,DELETE` and no body.

::: laravel
The same as Laravel: `Route::get` registers `GET` and `HEAD`, a wrong method throws `MethodNotAllowedHttpException` with that message, and `OPTIONS` is answered for you.
:::

## Route Parameters

Put the parameter in braces. Take it as `Param<T>`:

```rust
route.get("/posts/{id}", show);

async fn show(Param(id): Param<u64>) -> Result<Json<Post>> {
    Ok(Json(Post::find_or_fail(id).await?))
}
```

If the value does not parse as `T`, the answer is `404`. `/posts/abc` never reaches `show`. This is like `whereNumber('id')`.

Try it on the blog:

<TryIt :requests="[
  { label: 'GET /api/posts/1', status: '200 OK', ok: true, body: '{&quot;id&quot;:1,&quot;title&quot;:&quot;Sabai&quot;,&quot;body&quot;:&quot;Rust, but sabai&quot;}', why: 'Post::find_or_fail(1) found the post.' },
  { label: 'GET /api/posts/99', status: '404 Not Found', ok: false, body: '{&quot;message&quot;:&quot;Not Found&quot;}', why: 'No post has id 99, so find_or_fail answers 404.' },
  { label: 'GET /api/posts/abc', status: '404 Not Found', ok: false, body: '{&quot;message&quot;:&quot;Not Found&quot;}', why: 'abc is not a u64, so the handler never runs.' },
]" />

`Param` works when the route has one parameter. With more, take the `Request` and read each one: `request.param("comment")`.

### Wildcards

`{*path}` matches the rest of the path, including slashes:

```rust
route.get("/files/{*path}", file);
```

## Named Routes

```rust
route.get("/posts/{id}", PostController::show).name("posts.show");
```

Build the URL from the name in a feature test:

```rust
app().get_json(route("posts.show").with("id", 1));       // /posts/1
app().get_json(route("posts.index").with("page", 2));    // /posts?page=2
```

Extra values become the query string. Every value is URL-encoded. Two routes with the same name stop the app at startup.

## Route Groups

`prefix` puts every route in the group under a path:

```rust
route.prefix("/api").group(|route| {
    route.get("/health", health);    // GET /api/health
});
```

A group can take a whole routes file. This is how the blog puts `routes/api.rs` under `/api`:

```rust
route.prefix("/api").group(api::routes);
```

Groups can nest. `"api"`, `"/api"` and `"/api/"` all mean the same prefix.

A group can also share [middleware](/middleware#assigning-middleware-to-routes): `route.prefix("/admin").middleware(Auth).group(...)`, or `route.middleware(Auth).group(...)` with no prefix.

::: laravel
```php
Route::prefix('api')->group(base_path('routes/api.php'));
```
:::

## Resource Routes

`api_resource` adds the REST routes for a controller. List only the actions it has:

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

The parameter is `{id}`, not `{post}`. Sabai has no route model binding yet.

::: laravel
```php
Route::apiResource('posts', PostController::class)->only(['index', 'store', 'show']);
```
:::

## Listing Routes

`sabai route:list` shows every route, sorted by path, like `php artisan route:list`:

```text
  GET|HEAD       / ...................................................... home › routes::web::home
  GET|HEAD       api/posts .................................... posts.index › PostController@index
  POST           api/posts .................................... posts.store › PostController@store
  GET|HEAD       api/posts/{id} ................................. posts.show › PostController@show

                                                                                Showing [4] routes
```

A controller action shows as `PostController@index`, a function by its path in the app, and a closure by its route name only, as Laravel shows them. Lines fit the terminal's width, and colors appear only in a terminal; set `NO_COLOR` to turn them off.

## Startup Checks

Sabai checks every route before it starts listening. These stop the app with a message that names the route:

- the same method and path twice
- the same name twice
- a bad path
- a handler that needs a service nobody bound

```text
Error: route `GET /api/health` cannot start: no service is bound to `dyn Clock`;
bind it in a provider's `register`: `app.bind::<dyn Clock>(Arc::new(...))`
```
