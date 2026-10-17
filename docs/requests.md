# Requests

## Form Requests

A form request is a struct with the input of one action. Write it like Laravel's `StorePostRequest`:

```rust
use sabai::http::FormRequest;
use sabai::http::rules::{Rules, max, required, string};
use serde::Deserialize;

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

Take it as an action argument. The action only runs when the rules pass:

```rust
pub async fn store(request: StorePostRequest) -> Result<(StatusCode, Json<Post>)> {
    let mut post = Post { id: 0, title: request.title, body: request.body };
    post.save().await?;
    Ok((StatusCode::CREATED, Json(post)))
}
```

::: laravel
```php
public function store(StorePostRequest $request)
{
    return Post::create($request->validated());
}
```
:::

See [Validation](./validation) for the rules.

### Where the Input Comes From

| Request | Input |
| --- | --- |
| `GET`, `HEAD` | the query string: `?title=Sabai` |
| `Content-Type: application/json` | the JSON body |
| `Content-Type: application/x-www-form-urlencoded` | the form body |

Any other `Content-Type` answers `422`. This blocks a cross-site form that sends JSON as `text/plain`. Broken JSON answers `400`.

## The Request

Take `Request` to read anything else:

```rust
pub async fn show(request: Request) -> String {
    format!("{} {}", request.method(), request.path())
}
```

| Method | Returns |
| --- | --- |
| `request.method()` | `GET`, `POST`, ... |
| `request.path()` | `/posts/7` |
| `request.query_string()` | `Some("page=2")` |
| `request.header("accept")` | `Some("application/json")` |
| `request.param("id")` | the route parameter as text |
| `request.body().as_bytes()` | the raw body |

`Request` is cheap to clone, so you can take it next to other arguments.
