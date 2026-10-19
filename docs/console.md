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

Run them through Cargo for now:

```bash
cargo run -p blog -- migrate
cargo run -p blog            # serve
cargo run -p blog -- list
```

A mistyped command says what to do:

```text
Error: there is no `migrat` command; run with `--help` to see the commands
```

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
cargo run -p blog -- inspire
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

The `sabai` command (`sabai migrate`, `sabai serve`, `sabai new`) is coming next.
