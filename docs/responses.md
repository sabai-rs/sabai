# Responses

An action returns anything that can become a response.

| Return | Response |
| --- | --- |
| `"Hello"` or a `String` | `200`, `text/plain` |
| `Json(value)` | `200`, `application/json` |
| `StatusCode::NO_CONTENT` | that status, empty body |
| `(StatusCode::CREATED, Json(post))` | that status with the body |
| `Result<T>` | `T`, or the [error](./errors) as JSON |
| `()` | `200`, empty body |

## JSON

`Json` takes any type that implements `Serialize`:

```rust
#[derive(Serialize)]
pub struct Stats {
    post_views: usize,
}

pub async fn index() -> Json<Stats> {
    Json(Stats { post_views: 3 })
}
```

::: laravel
```php
return response()->json(['post_views' => 3]);
```
:::

## Status Codes

Put the status first:

```rust
pub async fn store(request: StorePostRequest) -> Result<(StatusCode, Json<Post>)> {
    let mut post = Post { id: 0, title: request.title, body: request.body };
    post.save().await?;
    Ok((StatusCode::CREATED, Json(post)))
}
```

## Not Yet

These are on the [roadmap](https://github.com/sabai-rs/sabai/blob/main/ROADMAP.md): response headers, redirects (`redirect()->route()`), views, file downloads and cookies.
