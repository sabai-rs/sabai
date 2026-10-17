# Service Providers

A provider sets up part of the app, like Laravel's `AppServiceProvider`.

```rust
pub struct AppServiceProvider;

impl Provider for AppServiceProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.bind::<dyn Clock>(Arc::new(SystemClock));
        Ok(())
    }
}
```

## Register the Provider

Add it when you build the app, in `src/lib.rs`:

```rust
pub fn app(config: Config) -> Result<Arc<App>> {
    App::builder(config)
        .provider(DatabaseServiceProvider)
        .provider(AppServiceProvider)
        .boot()
}
```

::: laravel
```php
// bootstrap/providers.php
return [
    App\Providers\AppServiceProvider::class,
];
```
:::

## `register` and `boot`

| Method | Use it to | Runs |
| --- | --- | --- |
| `register` | bind services | first, for every provider |
| `boot` | use services from other providers | after every `register` |

`boot` is optional. Do not resolve services in `register`: another provider may not have bound them yet.

```rust
impl Provider for NotificationProvider {
    fn register(&self, _app: &mut App) -> Result<()> {
        Ok(())
    }

    fn boot(&self, app: &App) -> Result<()> {
        let mailer = app.get::<dyn Mailer>()?;   // bound by another provider
        Ok(())
    }
}
```

## Errors

A provider that fails stops the app and is named in the message:

```text
Error: provider `blog::app::providers::app_service_provider::AppServiceProvider` failed to register: ...
```

## Framework Providers

| Provider | Binds |
| --- | --- |
| `sabai::orm::DatabaseServiceProvider` | `Database` and `DatabaseManager` from `config/database.toml` |
