# Installation

## Requirements

- Rust 1.88 or newer (`rustup update`)
- Docker, if you want MySQL for development; tests run on SQLite and need nothing

## Try the Example Blog

The fastest way to see Sabai is the example blog in the repository. It uses every feature as it lands.

```bash
git clone https://github.com/sabai-rs/sabai
cd sabai
cargo run -p blog
```

The blog serves on `http://127.0.0.1:8000`. It keeps posts in one database and post views in a second one, `report`.

With Docker, use MySQL for both, as in development:

```bash
docker compose up -d
cp examples/blog/.env.example examples/blog/.env
cargo run -p blog -- migrate
cargo run -p blog
```

Without Docker, point both connections at SQLite files:

```bash
export DB_CONNECTION=sqlite DB_DATABASE=blog.sqlite
export REPORT_DB_CONNECTION=sqlite REPORT_DB_DATABASE=report.sqlite
cargo run -p blog -- migrate
cargo run -p blog
```

Run its feature tests with:

```bash
cargo nextest run -p blog
```

## A New Application

There is no `sabai new` yet. Until it lands, start from a copy of `examples/blog`, or add Sabai to a binary crate from Git:

```toml
[dependencies]
sabai = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
sabai-testing = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }
```

The `orm` feature brings the database layer. Leave it out for an app without a database; it then never compiles sqlx.
