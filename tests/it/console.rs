use std::sync::{Arc, Mutex};

use sabai::console::Console;
use sabai::{App, Command, Config, Provider, Result};

fn console() -> Console {
    Console::new(App::builder(Config::default()).boot().unwrap())
}

fn run(console: Console, args: &[&str]) -> String {
    let args: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
    console.run_with(&args).err().unwrap().to_string()
}

#[test]
fn an_unknown_command_says_how_to_see_the_commands() {
    let error = run(console(), &["migrat"]);

    assert_eq!(
        error,
        "there is no `migrat` command; run with `--help` to see the commands"
    );
}

#[test]
fn serve_without_routes_says_how_to_add_them() {
    let error = run(console(), &[]);

    assert!(error.starts_with("`serve` has no routes"), "{error}");
}

#[test]
fn help_lists_every_command() {
    let help = console().routes(sabai::http::Router::new()).help();

    assert!(help.contains("  serve "), "{help}");
    assert!(help.contains("  list "), "{help}");
}

#[cfg(feature = "orm")]
#[test]
fn migrate_without_migrations_says_how_to_add_them() {
    let error = run(console(), &["migrate"]);

    assert!(error.starts_with("`migrate` has no migrations"), "{error}");
}

#[test]
fn the_console_keeps_the_app_it_was_given() {
    let app = App::builder(Config::default()).boot().unwrap();
    let console = Console::new(Arc::clone(&app));

    assert!(console.run_with(&["list".to_owned()]).is_ok());
}

/// A command a package would ship: it keeps the arguments it ran with.
struct Greet {
    name: &'static str,
    ran_with: Arc<Mutex<Vec<String>>>,
}

impl Command for Greet {
    fn name(&self) -> &str {
        self.name
    }

    fn description(&self) -> &str {
        "Greet someone"
    }

    fn handle(&self, _app: &Arc<App>, args: &[String]) -> Result<()> {
        *self.ran_with.lock().unwrap() = args.to_vec();
        Ok(())
    }
}

/// A package's provider, like `$this->commands([...])` in Laravel.
struct GreetingProvider {
    names: Vec<&'static str>,
    ran_with: Arc<Mutex<Vec<String>>>,
}

impl Provider for GreetingProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        for name in &self.names {
            app.command(Greet {
                name,
                ran_with: Arc::clone(&self.ran_with),
            })?;
        }
        Ok(())
    }
}

fn app_with_commands(names: &[&'static str]) -> (Result<Arc<App>>, Arc<Mutex<Vec<String>>>) {
    let ran_with = Arc::new(Mutex::new(Vec::new()));
    let provider = GreetingProvider {
        names: names.to_vec(),
        ran_with: Arc::clone(&ran_with),
    };
    (
        App::builder(Config::default()).provider(provider).boot(),
        ran_with,
    )
}

#[test]
fn a_package_command_runs_with_the_arguments_after_its_name() {
    let (app, ran_with) = app_with_commands(&["greet"]);
    let args = ["greet", "Somchai"].map(String::from);

    Console::new(app.unwrap()).run_with(&args).unwrap();

    assert_eq!(*ran_with.lock().unwrap(), ["Somchai"]);
}

#[test]
fn help_lists_package_commands_after_the_built_in_ones() {
    let (app, _) = app_with_commands(&["greet"]);
    let help = Console::new(app.unwrap()).help();

    let greet = help
        .lines()
        .find(|line| line.trim_start().starts_with("greet"));
    assert!(
        greet.is_some_and(|line| line.ends_with("Greet someone")),
        "{help}"
    );
    assert!(
        help.find("list").unwrap() < help.find("greet").unwrap(),
        "{help}"
    );
}

#[test]
fn two_commands_with_one_name_fail_at_boot() {
    let (app, _) = app_with_commands(&["greet", "greet"]);

    let error = app.unwrap_err().to_string();

    assert!(error.contains("two commands are named `greet`"), "{error}");
}

#[test]
fn a_package_command_cannot_hide_a_built_in_one() {
    let (app, _) = app_with_commands(&["serve"]);
    let console = Console::new(app.unwrap()).routes(sabai::http::Router::new());

    let error = run(console, &["serve"]);

    assert!(
        error.starts_with("two commands are named `serve`"),
        "{error}"
    );
}
