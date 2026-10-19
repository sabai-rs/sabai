# Installation

## Requirements

- Rust 1.88 or newer. Run `rustup update`.
- Docker, only if you want MySQL. Tests use SQLite and need nothing.

## Run the Example Blog

The blog in the repository uses every feature of Sabai. Clone it first:

```bash
git clone https://github.com/sabai-rs/sabai
cd sabai
```

Install the `sabai` command from the clone. It runs your app's commands, like `php artisan`:

```bash
cargo install --path crates/sabai-cli
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
sabai serve
```

Open `http://127.0.0.1:8000` to see `Welcome to Sabai`.

`sabai new` writes Laravel's layout in Rust:

```text
my-app/
  Cargo.toml
  .env, .env.example
  config/       app.toml, logging.toml
  src/
    main.rs     runs the app's commands
    lib.rs      config() and app(), like bootstrap/app.php
    app/providers/app_service_provider.rs
    routes.rs, routes/web.rs
```

The app depends on Sabai from GitHub. To try a local clone instead, pass its folder:

```bash
sabai new my-app --sabai-path ~/code/sabai
```

::: laravel
`sabai new` is `laravel new`. Like Laravel, it copies `.env.example` to `.env` for you.
:::

The new app has no database yet. To add one, turn on the `orm` feature and follow [Database](/database):

```toml
sabai = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }
```
