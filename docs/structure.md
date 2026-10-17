# Directory Structure

A Sabai app uses Laravel's folders. They live under `src/` because Cargo only compiles what is there.

```text
blog/
  Cargo.toml
  .env.example
  .env.testing
  config/                      app.toml, database.toml, logging.toml
  src/
    main.rs                    serve the app, or run `migrate`
    lib.rs                     config(), app() and routes()
    app/
      http/
        controllers/           PostController
        requests/              StorePostRequest
      models/                  Post
      providers/               AppServiceProvider
      repositories/            PostRepository
      services/                PostService
    database/
      migrations/              m2026_10_15_000000_create_posts_table.rs
    routes/
      api.rs
      web.rs
  tests/
    feature/
      main.rs                  app(), like Laravel's TestCase
      api/post_controller/     index.rs, show.rs, store.rs
```

## One File per Folder

In Rust, every folder is a module. Each folder has a small file that lists what is inside. `src/app/http/controllers.rs`:

```rust
mod post_controller;

pub use post_controller::PostController;
```

Sabai never reads folders. You can arrange them your own way, for example `app/domain/{models,services,repositories}`.

## Why `lib.rs`

Feature tests are a separate crate. They can only use what the app exports. `lib.rs` exports `config()`, `app(config)` and `routes()`, and both `main.rs` and the tests use them.

## Tests in One Binary

Cargo builds every file directly under `tests/` as its own program. So feature tests live as modules of one `tests/feature/main.rs`, one file per controller action. Adding a test file does not add a build.
