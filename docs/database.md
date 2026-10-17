# Database

Sabai supports MySQL and SQLite. Postgres comes later.

## Configuration

`config/database.toml` works like Laravel's `config/database.php`. `default` picks one of the `connections`:

```toml
default = "${DB_CONNECTION:-sqlite}"

[connections.sqlite]
driver = "sqlite"
database = "${DB_DATABASE:-:memory:}"

[connections.mysql]
driver = "mysql"
host = "${DB_HOST:-127.0.0.1}"
port = "${DB_PORT:-3306}"
database = "${DB_DATABASE:-blog}"
username = "${DB_USERNAME:-sabai}"
password = "${DB_PASSWORD:-}"
```

Set the connection in `.env`:

```bash
DB_CONNECTION=mysql
DB_HOST=127.0.0.1
DB_PORT=3306
DB_DATABASE=blog
DB_USERNAME=sabai
DB_PASSWORD=secret
```

For SQLite, `DB_DATABASE` is a file path, or `:memory:` for a database that lives only as long as the app.

## Register the Provider

Add `DatabaseServiceProvider` in `src/lib.rs`:

```rust
App::builder(config)
    .provider(DatabaseServiceProvider)
    .provider(AppServiceProvider)
    .boot()
```

It reads the config when the app starts. A wrong value stops the app there. Nothing connects until the first query.

## MySQL With Docker

The repository has a `docker-compose.yml` for development:

```bash
docker compose up -d
```

It starts two MySQL 8.4 servers, both with user `sabai` and password `secret`:

- `mysql` on port `3306`, with the `blog` database
- `mysql-report` on port `3307`, with the `report` database

## Several Connections

Add more `[connections.*]` sections. Each one has its own pool:

```toml
[connections.report]
driver = "mysql"
host = "${REPORT_DB_HOST:-127.0.0.1}"
port = "${REPORT_DB_PORT:-3307}"
database = "report"
username = "sabai"
password = "${REPORT_DB_PASSWORD:-}"
```

A model picks its connection with `#[model(connection = "report")]`. See [Eloquent](./eloquent#connections).

Get a connection by name, like `DB::connection('report')`:

```rust
let report = App::current()?.get::<DatabaseManager>()?.connection("report")?;
```

## Raw SQL

Run one statement, like `DB::statement(...)`:

```rust
let database = App::current()?.get::<Database>()?;
database.statement("delete from post_views").await?;
```

## When the Database Is Down

A query waits up to 5 seconds for a connection, then fails with a `500`. The log says:

```text
ERROR could not connect to the database: pool timed out while waiting for an open connection
```
