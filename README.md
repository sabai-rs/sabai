<h1 align="center">Sabai</h1>

<p align="center"><strong>Rust, but sabai.</strong></p>

<p align="center">
<a href="https://github.com/sabai-rs/sabai/actions/workflows/ci.yml"><img src="https://github.com/sabai-rs/sabai/actions/workflows/ci.yml/badge.svg" alt="Build Status"></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License"></a>
</p>

> Sabai is in early development and not yet published to crates.io. APIs change from one commit to the next.

## About Sabai

Sabai is a web application framework for Rust, built to feel like Laravel. If you know Laravel, you should already know where things go and what they are called. Sabai keeps the parts that make Laravel enjoyable, and adds what Rust gives for free: a single fast binary, and mistakes caught when the app starts instead of in production.

Three things guide every decision, in this order:

1. **Developer experience.** APIs read like Laravel, and errors say how to fix them.
2. **Compile speed.** Every change is measured for its effect on build time.
3. **Correctness and security.**

What works today:

- Routing with route groups and named routes: `route.prefix("/api").group(api::routes)`, `route.get("/posts/{id}", PostController::show).name("posts.show")`.
- Controllers built once at startup from a service container, like Laravel's constructor injection.
- Handler arguments like Laravel's method injection: `FormRequest` structs, route `Param`s and container `Service`s.
- Validation with Laravel's rules, messages and `422` shape.
- Config from `.env` and `config/*.toml`, logging, service providers and events.
- Migrations with a schema builder, and Eloquent-style models (`Post::all()`, `find_or_fail`, `save`) on MySQL or SQLite, with several connections.
- A feature-test client: `app().get_json(route("posts.show").with("id", 1)).assert_status(200)`.

On the way: middleware, auth, queues, the `sabai` console, storage, cache and mail. See the [roadmap](ROADMAP.md).

## A Taste

Routes, like `routes/api.php`:

```rust
pub fn routes(route: &mut Router) {
    route.post("/posts", PostController::store).name("posts.store");
    route.get("/posts/{id}", PostController::show).name("posts.show");
}
```

A form request, like `app/Http/Requests/StorePostRequest.php`:

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

A controller, with models straight from Eloquent:

```rust
pub struct PostController;

impl PostController {
    pub async fn index() -> Result<Json<Vec<Post>>> {
        Ok(Json(Post::all().await?))
    }

    pub async fn show(Param(id): Param<u64>) -> Result<Json<Post>> {
        Ok(Json(Post::find_or_fail(id).await?))
    }
}
```

When every action needs the same service, take it in the constructor instead, like Laravel's constructor injection:

```rust
pub struct PostController {
    post_service: PostService,
}

impl FromApp for PostController {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_service: app.make::<PostService>()?,
        })
    }
}

impl Controller for PostController {}

impl PostController {
    pub async fn store(self: Arc<Self>, request: StorePostRequest) -> Result<(StatusCode, Json<Post>)> {
        Ok((StatusCode::CREATED, Json(self.post_service.create(request).await?)))
    }
}
```

A feature test, like `tests/Feature/PostController/StoreTest.php`:

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

A missing service binding stops the app at startup and names the route, instead of failing a request (type paths shortened):

```text
Error: route `GET /api/health` cannot start: no service is bound to `dyn Clock`;
bind it in a provider's `register`: `app.bind::<dyn Clock>(Arc::new(...))`
```

## Learning Sabai

The [example blog](examples/blog) uses every feature as it lands, laid out like a Laravel app (`src/app/http/controllers`, `src/app/models`, `src/routes`, `tests/feature`). It is the best place to see how the pieces fit together.

```bash
cargo run -p blog          # serves on http://127.0.0.1:8000
cargo nextest run -p blog  # runs the blog's feature tests
```

The [documentation](https://sabai-rs.github.io/sabai/) covers everything above, from installation and routing to migrations, models and tests; its source is in [`docs/`](docs) (`bun run dev` there to preview it). The [roadmap](ROADMAP.md) lists every feature, what is done, and what comes next.

## Contributing

Thank you for considering contributing to Sabai! Work follows the [roadmap](ROADMAP.md), one ticket at a time:

- One branch per ticket, named after it: `m1-03-named-routes`.
- Commit messages start with the ticket: `feat(M1-03a): named routes ...`.
- Use the feature in `examples/blog` before calling it done.
- Report the build-time impact (`cargo build --timings`, `cargo llvm-lines`) of changes to public APIs or macros.
- CI fails when a crate goes over its LLVM-lines budget in `compile-budgets.toml`. The total is the `(TOTAL)` line of `cargo llvm-lines -p <crate> --lib`. Raising a budget is fine when the growth is worth it: change the number in the same pull request and say why.

Before opening a pull request:

```bash
cargo fmt --all
cargo lint   # clippy on every crate and target, warnings are errors
cargo t      # nextest on the whole workspace
```

[`CLAUDE.md`](CLAUDE.md) has the full conventions, including the compile-time rules every crate follows.

## Security Vulnerabilities

If you discover a security vulnerability within Sabai, please report it privately through [GitHub security advisories](https://github.com/sabai-rs/sabai/security/advisories/new) instead of a public issue.

## License

Sabai is open-sourced software licensed under the [MIT license](LICENSE).
