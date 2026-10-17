# Service Container

The container holds your app's services, like Laravel's `app()`.

## Bind a Service

Bind services in a provider's `register`. Bind an interface (a trait) to a real type:

```rust
app.bind::<dyn Clock>(Arc::new(SystemClock));
```

::: laravel
```php
$this->app->bind(Clock::class, SystemClock::class);
```
:::

## Get a Service

```rust
let clock = app.get::<dyn Clock>()?;
```

A service nobody bound is an error that shows the line to add:

```text
no service is bound to `dyn Clock`; bind it in a provider's `register`:
`app.bind::<dyn Clock>(Arc::new(...))`
```

## Build a Class

`make` builds a type from the app, like Laravel's autowiring. The type says what it needs in `FromApp`:

```rust
pub struct PostService {
    post_repository: PostRepository,
}

impl FromApp for PostService {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_repository: app.make::<PostRepository>()?,
        })
    }
}

let posts = app.make::<PostService>()?;
```

| Method | Gives you | Laravel |
| --- | --- | --- |
| `app.get::<dyn Trait>()` | the bound service | `app(Contract::class)` |
| `app.make::<T>()` | a new `T`, every call | `app(Service::class)` |
| `app.singleton::<T>()` | the same `T`, built once | a `singleton` binding |

## In an Action

Take a bound service as an argument with `Service<T>`:

```rust
async fn health(clock: Service<dyn Clock>) -> Result<String> {
    let now = clock.now().duration_since(UNIX_EPOCH)?;
    Ok(format!("ok at {}", now.as_secs()))
}
```

If it is not bound, the app does not start.

## The Current App

`App::current()` gives the app of the current request, test or command, like `app()`:

```rust
let clock = App::current()?.get::<dyn Clock>()?;
```

For background work, use `sabai::spawn` instead of `tokio::spawn`. It takes the app along.
