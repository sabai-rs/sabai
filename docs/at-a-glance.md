# At a glance

Sabai is a web framework for Rust that keeps Laravel's names and folders. This page walks through one feature of the [example blog](https://github.com/sabai-rs/sabai/tree/main/examples/blog), from the route to the test.

## A route, a controller and a model

::: code-group

```rust [routes/api.rs]
route.api_resource("posts", |posts| {
    posts.show(PostController::show);
    posts.store(PostController::store);
});
```

```rust [post_controller.rs]
pub struct PostController;

impl PostController {
    pub async fn show(Param(id): Param<u64>) -> Result<Json<Post>> {
        Ok(Json(Post::find_or_fail(id).await?))
    }
}
```

```rust [post.rs]
#[derive(Model, Serialize)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
}
```

:::

::: laravel
```php
Route::apiResource('posts', PostController::class)->only(['show', 'store']);
```
:::

`Post::find_or_fail(id)` finds the row or answers 404. The model finds its database by itself, so the controller passes nothing but the id.

## Try it

Pick a request. These are the answers the blog gives.

<TryIt />

::: tip
`abc` never reaches the handler: `Param<u64>` answers 404 for anything that is not a number, like `whereNumber('id')`.
:::

## Validation

A form request is a struct with its rules. The handler only runs when they pass, and a failure answers `422` in Laravel's shape.

```rust
#[derive(Deserialize)]
pub struct StorePostRequest {
    pub title: String,
    pub body: String,
}

impl FormRequest for StorePostRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("title", [required(), string(), max(255)])
            .field("body", [required(), string()])
    }
}
```

## A feature test

Each test boots the blog on its own SQLite database in memory, migrated first, like Laravel's `RefreshDatabase`.

```rust
const ROUTE: &str = "posts.store";

#[test]
fn store_requires_a_title_and_a_body() {
    let response = app().post_json(route(ROUTE), json!({}));

    response
        .assert_status(422)
        .assert_json_fragment(json!({"title": ["The title field is required."]}));
}
```

Next, [install it](./installation) or read about [routing](./routing).
