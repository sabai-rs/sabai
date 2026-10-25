# Dates

`sabai::DateTime` is a moment in time, kept in UTC, like a Carbon instance in Laravel. It is what a `timestamp` column holds.

```rust
use sabai::DateTime;

let now = DateTime::now();
let published: DateTime = "2026-10-23 10:00:00".parse()?;
```

## Reading Text

`parse` takes either form:

| Input | Read as |
| --- | --- |
| `2026-10-23 10:00:00` | the database form, in UTC |
| `2026-10-23T17:00:00+07:00` | a time with an offset, turned into UTC |

Text that is neither is an error that shows the form it expects.

## Printing

| Call | Gives |
| --- | --- |
| `time.to_string()` or `{time}` | `2026-10-23 10:00:00`, what the database stores |
| `time.to_json()` | `2026-10-23T10:00:00.000000Z` |

A `DateTime` field serializes to JSON in the second form, with microseconds and a `Z`, the same as Laravel's models do. JSON input reads either form.

## The App's Clock

To read the time from the bound `Clock`, so tests can freeze it, convert what it gives:

```rust
async fn health(clock: Service<dyn Clock>) -> Result<String> {
    let now = DateTime::try_from(clock.now())?;
    Ok(format!("ok at {now}"))
}
```

Times are in UTC for now. A configurable `APP_TIMEZONE` comes later.
