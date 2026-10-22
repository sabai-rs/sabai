# HTTP Tests

Feature tests call your routes without a server, like Laravel's `$this->getJson(...)`.

## Setup

Add `sabai-testing` as a dev dependency:

```toml
[dev-dependencies]
sabai-testing = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }
```

Make `tests/feature/main.rs`, like Laravel's `TestCase`:

```rust
mod api;

use sabai::Config;
use sabai_testing::TestApp;

fn app() -> TestApp {
    let config = Config::load_for(env!("CARGO_MANIFEST_DIR"), "testing").expect("the config loads");
    TestApp::new(bootstrap::app(config).expect("the blog boots"), bootstrap::routes())
        .refresh_database(&bootstrap::migrations())
}
```

- `load_for(..., "testing")` reads `.env.testing`. Point it at SQLite in memory.
- `refresh_database` runs your migrations, like `RefreshDatabase`. Every test gets a new, empty database.

## Write a Test

One file per action: `tests/feature/api/post_controller/store.rs`.

```rust
use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "posts.store";

#[test]
fn store_creates_a_post() {
    let response = app().post_json(route(ROUTE), json!({"title": "Sabai", "body": "b"}));

    response
        .assert_status(201)
        .assert_json_fragment(json!({"id": 1, "title": "Sabai"}));
}
```

::: laravel
```php
public function test_store_creates_a_post(): void
{
    $response = $this->postJson(route(self::ROUTE), ['title' => 'Sabai', 'body' => 'b']);

    $response->assertStatus(201)
        ->assertJsonFragment(['id' => 1, 'title' => 'Sabai']);
}
```
:::

Tests are plain `#[test]` functions. You do not write `async` or `.await`.

## Requests

| Code | Request |
| --- | --- |
| `app.get_json(route("posts.show").with("id", 1))` | `GET /api/posts/1` |
| `app.get_json(route("posts.index").with("page", 2))` | `GET /api/posts?page=2` |
| `app.post_json(route("posts.store"), json!({...}))` | `POST` with a JSON body |
| `app.get_json("/api/health")` | a plain URL also works |

Use one `app` for several requests in a test, so they share a database:

```rust
let app = app();
app.post_json(route("posts.store"), json!({"title": "Sabai", "body": "b"}));

let response = app.get_json(route("posts.show").with("id", 1));
```

To read the database the way the app does, like calling Eloquent in a Laravel test, pass a future to `app.run`:

```rust
let user = app.run(User::find_or_fail(1))?;
assert!(app.run(Hash::check("correct horse", &user.password)));
```

`.env.testing` sets `BCRYPT_ROUNDS=4`, like Laravel's `phpunit.xml`, so tests that hash passwords stay fast.

## Assertions

| Method | Checks |
| --- | --- |
| `assert_status(422)` | the status |
| `assert_ok()` | `200` |
| `assert_created()` | `201` |
| `assert_not_found()` | `404` |
| `assert_json_fragment(json!({...}))` | one object in the JSON has every key and value |
| `assert_header("x-frame-options", "SAMEORIGIN")` | the header's value |
| `assert_header_missing("cache-control")` | no such header |

To send a header, add `with_header` before the request, like Laravel's `withHeader`:

```rust
app().with_header("x-request-id", "lb-7f3a").get_json("/api/health");
```

Each one returns the response, so you can chain them. `response.json()` and `response.text()` give the body.

A failed assertion shows the body:

```text
Expected response status code [201] but received 422.
{"message":"The title field is required. (and 1 more error)", ...}
```

## Run Tests

```bash
cargo nextest run -p blog
```
