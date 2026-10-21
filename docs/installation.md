# Installation

## Requirements

- Rust 1.88 or newer. Run `rustup update`.
- Docker, only if you want MySQL. Tests use SQLite and need nothing.

## Install the `sabai` Command

`sabai` creates apps and runs their commands, like `laravel new` and `php artisan`:

```bash
cargo install --git https://github.com/sabai-rs/sabai sabai-cli
```

Working on Sabai itself? Install it from your clone instead, so it uses your changes: `cargo install --path crates/sabai-cli`.

## Run the Example Blog

The blog in the repository uses every feature of Sabai. Clone it first:

```bash
git clone https://github.com/sabai-rs/sabai
cd sabai
```

The blog uses two databases: `blog` for posts and `report` for post views.

### With Docker (MySQL)

```bash
docker compose up -d
cp examples/blog/.env.example examples/blog/.env
cd examples/blog
sabai migrate
sabai serve
```

### Without Docker (SQLite)

```bash
export DB_CONNECTION=sqlite DB_DATABASE=blog.sqlite
export REPORT_DB_CONNECTION=sqlite REPORT_DB_DATABASE=report.sqlite
cd examples/blog
sabai migrate
sabai serve
```

Without the `sabai` command, `cargo run -p blog -- migrate` and `cargo run -p blog` do the same from the repository root.

The blog runs on `http://127.0.0.1:8000`. Try `curl http://127.0.0.1:8000/api/posts`.

### Run the Tests

```bash
cargo nextest run -p blog
```

Tests use SQLite in memory, so they need no database.

## Start a New App

```bash
sabai new my-app
cd my-app
sabai migrate
sabai serve
```

Open `http://127.0.0.1:8000` to see `Welcome to Sabai`.

The app starts on SQLite in `database/database.sqlite`, so there is nothing to set up. `sabai migrate` creates the file and Laravel's `users` table. To use MySQL, set the `DB_` lines in `.env`; `.env.example` lists them.

`sabai new` writes Laravel's layout in Rust:

```text
my-app/
  Cargo.toml
  .env, .env.example, .env.testing
  config/            app.toml, database.toml, logging.toml
  database/          database.sqlite, once you migrate
  src/
    main.rs          runs the app's commands
    lib.rs           config(), app(), routes() and migrations(), like bootstrap/app.php
    app/models/user.rs
    app/providers/app_service_provider.rs
    database/migrations/m0001_01_01_000000_create_users_table.rs
    routes.rs, routes/web.rs
  tests/feature/     main.rs and example_test.rs
```

`cargo test` runs the example test on a SQLite database in memory, like Laravel's `ExampleTest`.

The app depends on Sabai from GitHub. To try a local clone instead, pass its folder:

```bash
sabai new my-app --sabai-path ~/code/sabai
```

::: laravel
`sabai new` is `laravel new`. Like Laravel 11, it copies `.env.example` to `.env`, starts on SQLite, and ships the `users` migration and a `User` model whose `password` never appears in JSON.
:::
