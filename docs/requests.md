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
    let mut post = Post { title: request.title, body: request.body, ..Default::default() };
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
| `Content-Type: application/json` | the query string and the JSON body |
| `Content-Type: application/x-www-form-urlencoded` | the query string and the form body |

When the query string and the body have the same field, the body wins. Any other `Content-Type` answers `422`. This blocks a cross-site form that sends JSON as `text/plain`. Broken JSON answers `400`.

Query strings and forms only carry text, so text fills a typed field when it can: `draft=true` or `draft=1` fills `draft: bool`, `page=2` fills `page: u32`, and `status=published` fills a unit enum variant. JSON text does the same, like Laravel, which does not tell the two apart. Text that does not fit answers `422`.

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
| `request.input("title")` | one input field as a JSON value: `Some(&json!("Sabai"))` |
| `request.all()` | every input field, or the error if the body cannot be read |
| `request.is_json()` | whether the `Content-Type` is JSON |
| `request.body().as_bytes()` | the raw body |

`input` and `all` read the same fields a form request does, from the table above. The body is parsed once, the first time anything asks, so a route that never reads input never parses it.

::: laravel
`request.input("title")` is `$request->input('title')`, `request.all()` is `$request->all()` and `request.is_json()` is `$request->isJson()`. Values from a query string or a form are always strings, as in Laravel.
:::

`Request` is cheap to clone, so you can take it next to other arguments.

To build one by hand, for a test or to call a route from code, use `Request::create`, like Laravel's `Request::create('/posts/7', 'GET')`:

```rust
let request = Request::create(Method::GET, "/posts/7")?;
let response = routes.dispatch(request).await;
```

## Input Trimming and Normalization

Two default middleware clean the input before any handler reads it:

| Middleware | Does |
| --- | --- |
| `TrimStrings` | trims spaces around every string, at any depth, except `password`, `password_confirmation` and `current_password` |
| `ConvertEmptyStringsToNull` | turns `""` into `null`, so a blank optional field arrives as `None` |

Every app runs them, trimming first, so `"   "` becomes `null`. To keep blank strings, remove the second in `src/lib.rs`:

```rust
use sabai::http::middleware::ConvertEmptyStringsToNull;

route.global_middleware(|middleware| {
    middleware.remove::<ConvertEmptyStringsToNull>();
});
```

They change what `request.input`, `request.all` and form requests see; `request.body()` stays as it was sent.

::: laravel
Both are Laravel's default global middleware of the same names, and `remove::<...>()` is `$middleware->remove(...)`.
:::

## Request Limits

`sabai serve` protects itself from clients that send too much or too slowly:

| Limit | When it is passed |
| --- | --- |
| A body of at most 2 MB | `413 Payload Too Large`, before any handler runs |
| The client sends all headers within 30 seconds | the connection is closed |
| The client sends the whole body within 30 seconds of its headers | `408 Request Timeout` |

These limits cover receiving the request only. How long a handler runs is not limited yet (M1-34b).

Change them in `config/app.toml`, or in `.env` through the variables it names:

```toml
max_body_kb = "${APP_MAX_BODY_KB:-2048}"     # 2 MB
header_timeout = "${APP_HEADER_TIMEOUT:-30}" # seconds
body_timeout = "${APP_BODY_TIMEOUT:-30}"     # seconds
```

```bash
# .env: accept uploads of up to 10 MB on slow connections
APP_MAX_BODY_KB=10240
APP_BODY_TIMEOUT=120
```

A missing key uses the default, and `0` stops the app at startup. These limits apply to every route, so raise them only as far as your largest upload needs; per-route limits are coming (M1-04).

The time limits stop slowloris attacks, where a client sends one byte at a time to hold its connection open; a few thousand such clients would otherwise use up the server. Per-route body limits are coming (M1-04).

::: laravel
In Laravel, PHP's `post_max_size` and nginx or Apache set these limits. A Sabai app is its own server, so it sets them itself.
:::
