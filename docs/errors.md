# Errors

## Return an Error

Actions return `Result<T>`. Any error becomes a JSON response:

```rust
pub async fn show(Param(id): Param<u64>) -> Result<Json<Post>> {
    Ok(Json(Post::find_or_fail(id).await?))   // 404 when there is no post
}
```

```json
{ "message": "Not Found" }
```

## Make an Error

| Code | Status | Laravel |
| --- | --- | --- |
| `Error::not_found()` | `404` | `abort(404)` |
| `Error::unauthorized()` | `401` | `abort(401)` |
| `Error::forbidden()` | `403` | `abort(403)` |
| `Error::from_status(StatusCode::CONFLICT)` | any | `abort(409)` |
| `Error::new(StatusCode::CONFLICT, "The post is locked.")` | any, with your message | `abort(409, 'The post is locked.')` |
| `Error::with_messages(&[("email", "...")])` | `422` | `ValidationException::withMessages` |

The client sees the message. Keep it safe to show.

## Any Error With `?`

`?` turns any other error into a `500`. The client only sees `Server Error`. The real cause goes to the log:

```rust
let now = clock.now().duration_since(UNIX_EPOCH)?;
```

## Add Context

`context` says what you were doing. It goes to the log, never to the client:

```rust
let post = Post::find_or_fail(id)
    .await
    .map_err(|error| error.context(format!("loading post {id} for the feed")))?;
```

## Logs

Every `5xx` is logged with its context and cause:

```text
ERROR could not connect to the database: pool timed out while waiting for an open connection
```

`4xx` errors are not logged. They are the client's mistake.

Coming next: the file and line of each error, Laravel's debug JSON when `APP_DEBUG=true`, and Sentry.
