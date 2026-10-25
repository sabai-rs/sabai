# Dates

`sabai::DateTime` is a moment in time, like a Carbon instance in Laravel. It is what a `timestamp` column holds. Inside it is UTC; text is read and printed in the app's time zone, and JSON is always UTC.

```rust
use sabai::DateTime;

let now = DateTime::now();
let published: DateTime = "2026-10-23 10:00:00".parse()?;
```

## Reading Text

`parse` takes either form:

| Input | Read as |
| --- | --- |
| `2026-10-23 10:00:00` | the database form, in the app's time zone |
| `2026-10-23T17:00:00+07:00` | a time with an offset, which says its own zone |

Text that is neither is an error that shows the form it expects.

## Printing

| Call | Gives |
| --- | --- |
| `time.to_string()` or `{time}` | `2026-10-23 10:00:00`, what the database stores, in the app's time zone |
| `time.to_json()` | `2026-10-23T10:00:00.000000Z`, in UTC |

A `DateTime` field serializes to JSON in the second form, with microseconds and a `Z`, the same as Laravel's models do. JSON input reads either form.

## The App's Clock

To read the time from the bound `Clock`, so tests can freeze it, convert what it gives:

```rust
async fn health(clock: Service<dyn Clock>) -> Result<String> {
    let now = DateTime::try_from(clock.now())?;
    Ok(format!("ok at {now}"))
}
```

## Time Zone

Set the app's zone in `config/app.toml`, like Laravel's `app.timezone`:

```toml
timezone = "${APP_TIMEZONE:-UTC}"
```

```bash
APP_TIMEZONE=Asia/Bangkok
```

With `Asia/Bangkok`, a post created at 12:00 UTC is stored as `2026-10-23 19:00:00`, prints as `19:00:00`, and its JSON is still `2026-10-23T12:00:00.000000Z`. That is what a Laravel app with the same zone writes and reads, so both can share a database.

The zone names come from the system's zone data in `/usr/share/zoneinfo`. A name that is not a zone stops the app at boot. Slim Docker images may have no zone data: install the `tzdata` package, or keep `UTC`, which needs none.

Outside an app, such as in a plain unit test, times are in UTC.

## Store UTC, Work in Local Time

On MySQL, the database can keep UTC while the app works in Bangkok time. Set the app's zone, and set the connection's session zone to the same offset:

```bash
# .env
APP_TIMEZONE=Asia/Bangkok
```

```toml
# config/database.toml
[connections.mysql]
# ...
timezone = "+07:00"
```

The app writes `19:00:00`, MySQL converts it from the session's `+07:00` and stores `12:00:00` UTC, and converts it back when the app reads it. This is the same as Laravel's `app.timezone` with the connection's `'timezone'` option.

| Where | Shows |
| --- | --- |
| The `timestamp` column, read in UTC | `12:00:00` |
| `{time}` in the app | `19:00:00` |
| JSON | `12:00:00.000000Z`, always UTC, like Laravel's `serializeDate` |

- Only `timestamp` columns convert, which `timestamps()` makes. A `datetime` column, and every SQLite column, keeps the app's text as is.
- Use an offset like `+07:00` for the session. A zone name like `Asia/Bangkok` works only once the MySQL server has loaded its zone tables, which the official Docker image has not.
- Leave both unset and everything is UTC.
