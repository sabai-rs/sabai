# Configuration

## Configuration Files

Each file in `config/` is one section, named after the file, like Laravel's `config/*.php`. Files are TOML:

```toml
# config/app.toml
name = "${APP_NAME:-Blog}"
host = "${APP_HOST:-127.0.0.1}"
port = "${APP_PORT:-8000}"
```

`${NAME}` reads an environment variable and `${NAME:-default}` falls back to a default, like `env('APP_NAME', 'Blog')`. A value that looks like a number or a boolean becomes one, so `port` above is the number `8000`.

## Environment Variables

Variables come from the process environment first, then from `.env` in the app's folder:

```bash
# .env
APP_NAME=Blog
APP_PORT=8000
LOG_LEVEL=info,sabai=debug
```

When `APP_ENV` is set, `.env.{APP_ENV}` is read instead of `.env`, so `APP_ENV=staging` reads `.env.staging`. Keep `.env` out of Git and commit an `.env.example` instead.

### The Testing Environment

Feature tests load config with `Config::load_for(base, "testing")`, which reads `.env.testing` and forces `APP_ENV=testing`, like `phpunit.xml`. A shell that sets `APP_ENV=local` can therefore never point tests at the development database.

```bash
# .env.testing
APP_ENV=testing
DB_CONNECTION=sqlite
DB_DATABASE=:memory:
```

## Reading Configuration

`config.get` reads a section or a dotted key into any type that implements `Deserialize`, like Laravel's `config('app.name')`:

```rust
let name: String = config.get("app.name")?;
let port: u16 = config.get_or("app.port", 8000)?;
```

`get_or` gives the default only when the key is missing. A key that is set with the wrong type is still an error, so a typo in a config file never goes unnoticed.

### Typed Sections

For a whole section, derive `ConfigSection`. `MailConfig` reads `config/mail.toml`:

```rust
#[derive(Deserialize, ConfigSection)]
pub struct MailConfig {
    pub host: String,
    pub port: u16,
}

let mail = config.section::<MailConfig>()?;
```

Name the struct `<Section>Config`, or set the file with `#[config("mail")]`.

## Errors

A missing variable, a broken `.env` line or an unclosed `${` stops the app at startup and names the file and key.
