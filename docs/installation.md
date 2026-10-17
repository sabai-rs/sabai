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

The blog uses two databases: `blog` for posts and `report` for post views.

### With Docker (MySQL)

```bash
docker compose up -d
cp examples/blog/.env.example examples/blog/.env
cargo run -p blog -- migrate
cargo run -p blog
```

### Without Docker (SQLite)

```bash
export DB_CONNECTION=sqlite DB_DATABASE=blog.sqlite
export REPORT_DB_CONNECTION=sqlite REPORT_DB_DATABASE=report.sqlite
cargo run -p blog -- migrate
cargo run -p blog
```

The blog runs on `http://127.0.0.1:8000`. Try `curl http://127.0.0.1:8000/api/posts`.

### Run the Tests

```bash
cargo nextest run -p blog
```

Tests use SQLite in memory, so they need no database.

## Start a New App

`sabai new` does not exist yet. For now, copy `examples/blog`, or add Sabai to a new binary crate:

```toml
[dependencies]
sabai = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
sabai-testing = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }
```

`orm` adds the database layer. Skip it if your app has no database. Then sqlx is never compiled.
