# Configuration

## Config Files

Each file in `config/` is one section. `config/app.toml` is the `app` section, like Laravel's `config/app.php`.

```toml
# config/app.toml
name = "${APP_NAME:-Blog}"
host = "${APP_HOST:-127.0.0.1}"
port = "${APP_PORT:-8000}"
timezone = "${APP_TIMEZONE:-UTC}"
```

- `${NAME}` reads an environment variable.
- `${NAME:-default}` uses the default when the variable is not set, like `env('APP_NAME', 'Blog')`.
- A value that looks like a number or `true`/`false` becomes one. `port` above is the number `8000`.

## The `.env` File

```bash
# .env
APP_NAME=Blog
APP_PORT=8000
LOG_LEVEL=info,sabai=debug
```

- Real environment variables win over `.env`.
- When `APP_ENV` is set, Sabai reads `.env.{APP_ENV}` instead. `APP_ENV=staging` reads `.env.staging`.
- Do not commit `.env`. Commit `.env.example`.

### Tests

Feature tests read `.env.testing`, like Laravel's `phpunit.xml`:

```rust
let config = Config::load_for(env!("CARGO_MANIFEST_DIR"), "testing")?;
```

```bash
# .env.testing
APP_ENV=testing
DB_CONNECTION=sqlite
DB_DATABASE=:memory:
```

`load_for` always sets `APP_ENV=testing`, even if your shell sets something else. Tests never touch your dev database.

## Read Config

```rust
let name: String = config.get("app.name")?;
let port: u16 = config.get_or("app.port", 8000)?;
```

`get` reads a key into any type, like `config('app.name')`. `get_or` uses the default only when the key is missing. A key with the wrong type is still an error.

### Typed Sections

Derive `ConfigSection` to read a whole file into a struct:

```rust
#[derive(Deserialize, ConfigSection)]
pub struct MailConfig {
    pub host: String,
    pub port: u16,
}

let mail = config.section::<MailConfig>()?;   // reads config/mail.toml
```

Name the struct `<File>Config`, or set the file with `#[config("mail")]`.

## Errors

A missing variable, a bad `.env` line or an unclosed `${` stops the app at startup. The message names the file and the key.
