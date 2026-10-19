# Console

Your app's binary runs its own commands, like `php artisan`.

## Set It Up

`src/main.rs` builds the console and runs the command on the command line:

```rust
use sabai::Result;
use sabai::console::Console;
use sabai::log::Logging;

fn main() -> Result<()> {
    let config = blog::config()?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    Console::new(blog::app(config)?)
        .routes(blog::routes())
        .migrations(blog::migrations())
        .run()
}
```

`.migrations(...)` needs the `orm` feature.

## Commands

| Command | Does | Laravel |
| --- | --- | --- |
| `serve` | serves the app on `app.host:app.port`; the default | `php artisan serve` |
| `migrate` | runs the migrations that have not run yet | `php artisan migrate` |
| `list` | shows the commands, yours and your packages' too; also `--help` | `php artisan list` |

Run them with the `sabai` command from anywhere inside your app's folder ([install it](/installation)):

```bash
sabai migrate
sabai serve
sabai           # lists the commands
```

`sabai` finds the nearest `Cargo.toml` that uses Sabai and runs `cargo run -q -- <command>` there, so the app is always built from the code on disk first. Without it, run `cargo run -- migrate` yourself.

::: laravel
`sabai` is `php artisan`. It has no commands of its own yet: every command, `serve` included, comes from your app and its packages.
:::

A mistyped command says what to do:

```text
Error: there is no `migrat` command; did you mean `migrate`?
```

Like `php artisan`, it suggests every command that is a small typo away or contains what you typed. With nothing close, it says to run `list`.

## Running Commands on a Server

A server or CI job needs neither `sabai` nor Cargo. Your app's binary is its own `artisan`:

```bash
cargo build --release
./target/release/blog migrate
./target/release/blog serve
```

::: laravel
`./target/release/blog migrate` is `php artisan migrate --force` in a deploy script.
:::

## Writing Commands

A command is a type that implements `Command`. `handle` gets the app and the arguments typed after the command's name:

```rust
use sabai::{App, Command, Result};

pub struct Inspire;

impl Command for Inspire {
    fn name(&self) -> &str {
        "inspire"
    }

    fn description(&self) -> &str {
        "Show an inspiring quote"
    }

    fn handle(&self, _app: &App, _args: &[String]) -> Result<()> {
        println!("Simplicity is the ultimate sophistication.");
        Ok(())
    }
}
```

Add it in a service provider's `register`:

```rust
impl Provider for AppServiceProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.command(Inspire)
    }
}
```

```bash
sabai inspire
```

::: laravel
`app.command(Inspire)` is `$this->commands([Inspire::class])`. The blog keeps its commands in `src/app/console/`, like `app/Console/Commands`.
:::

## Commands From Packages

A package adds its commands the same way, in its own provider. Say a backup package ships `backup:run`; the app only adds its provider:

```rust
App::builder(config)
    .provider(AppServiceProvider)
    .provider(acme_backup::BackupServiceProvider)
    .boot()
```

Its commands then show up in `list` and run like your own. Two commands can't share a name: the app fails to boot and names the clash. A command named after a built-in one, like `serve`, is refused too.

`sabai new` is coming next.
