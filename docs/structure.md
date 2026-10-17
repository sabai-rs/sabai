# Directory Structure

A Sabai app is laid out like a Laravel app. Folders live under `src/`, because Cargo compiles only what is there, but the names and roles are the same.

```text
blog/
  Cargo.toml
  .env.example
  .env.testing
  config/
    app.toml
    database.toml
    logging.toml
  src/
    main.rs                    serves the app, or runs `migrate`
    lib.rs                     config(), app() and routes(), shared by main.rs and the tests
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

## Modules

Every folder is a Rust module, so each one has a small file that declares what is inside it. `src/app/http/controllers.rs` lists the controllers:

```rust
mod post_controller;

pub use post_controller::PostController;
```

The framework never reads folders, so you are free to arrange modules your own way, for example `app/domain/{models,services,repositories}`.

## Why `lib.rs`

Feature tests are a separate crate, so they can only reach what the app exports. `lib.rs` exports `config()`, `app(config)` and `routes()`, and both `main.rs` and `tests/feature/main.rs` boot the app through them.

## Tests in One Binary

Cargo compiles every file directly under `tests/` as its own program. Feature tests therefore live as modules of a single `tests/feature/main.rs`, with one file per controller action, so adding a test file does not add a link step.
