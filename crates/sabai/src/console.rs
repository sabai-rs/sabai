//! The app's own commands, like `php artisan`: `serve`, `migrate` and the commands providers add.

use std::collections::HashSet;
use std::sync::Arc;

use sabai_core::{App, Command, Error, Result, StatusCode};
use sabai_http::{Router, ServeCommand};

/// Runs the command named on the command line. `main` builds it once:
///
/// ```ignore
/// Console::new(blog::app(config)?)
///     .routes(blog::routes())
///     .migrations(blog::migrations())
///     .run()
/// ```
pub struct Console {
    app: Arc<App>,
    commands: Vec<Box<dyn Command>>,
}

impl Console {
    /// A console for `app`, with the commands its providers added.
    pub fn new(app: Arc<App>) -> Self {
        Self {
            app,
            commands: Vec::new(),
        }
    }

    /// The routes `serve` answers.
    pub fn routes(self, routes: Router) -> Self {
        self.with(Box::new(ServeCommand::new(routes)))
    }

    /// The migrations `migrate` runs.
    #[cfg(feature = "orm")]
    pub fn migrations(self, migrations: sabai_orm::Migrations) -> Self {
        self.with(Box::new(sabai_orm::MigrateCommand::new(migrations)))
    }

    fn with(mut self, command: Box<dyn Command>) -> Self {
        self.commands.push(command);
        self
    }

    /// Runs the command in the process's arguments; with none, `serve`.
    pub fn run(self) -> Result<()> {
        let args: Vec<String> = std::env::args().skip(1).collect();
        self.run_with(&args)
    }

    /// Runs the command in `args`, which leave out the program name.
    pub fn run_with(self, args: &[String]) -> Result<()> {
        let name = args.first().map_or("serve", String::as_str);
        if matches!(name, "list" | "help" | "--help" | "-h") {
            print!("{}", self.help());
            return Ok(());
        }
        self.reject_duplicate_names()?;
        let command = self.find(name).ok_or_else(|| command_not_found(name))?;
        command.handle(&self.app, args.get(1..).unwrap_or_default())
    }

    /// What `--help` prints, like `php artisan list`: Sabai's commands, then the app's and packages'.
    pub fn help(&self) -> String {
        let commands: Vec<(&str, &str)> = self
            .commands
            .iter()
            .map(|command| described(command.as_ref()))
            .chain([("list", "Show these commands")])
            .chain(self.app.commands().map(described))
            .collect();
        let width = commands
            .iter()
            .map(|(name, _)| name.len())
            .max()
            .unwrap_or(0)
            + 3;
        let lines: String = commands
            .iter()
            .map(|(name, description)| format!("  {name:width$}{description}\n"))
            .collect();
        format!("Usage: <command> [arguments]\n\nCommands:\n{lines}")
    }

    fn all(&self) -> impl Iterator<Item = &dyn Command> {
        self.commands
            .iter()
            .map(Box::as_ref)
            .chain(self.app.commands())
    }

    fn find(&self, name: &str) -> Option<&dyn Command> {
        self.all().find(|command| command.name() == name)
    }

    // A package's `serve` would never run, so say so instead of ignoring it.
    fn reject_duplicate_names(&self) -> Result<()> {
        let mut seen = HashSet::from(["list", "help"]);
        match self.all().find(|command| !seen.insert(command.name())) {
            Some(command) => Err(console_error(format!(
                "two commands are named `{}`; a package may add one that Sabai or the app already has, so rename it",
                command.name()
            ))),
            None => Ok(()),
        }
    }
}

fn described(command: &dyn Command) -> (&str, &str) {
    (command.name(), command.description())
}

fn command_not_found(name: &str) -> Error {
    match name {
        "serve" => {
            console_error("`serve` has no routes; add them with `Console::new(app).routes(...)`")
        }
        "migrate" => console_error(
            "`migrate` has no migrations; add them with `.migrations(...)` (needs the `orm` feature)",
        ),
        _ => console_error(format!(
            "there is no `{name}` command; run with `--help` to see the commands"
        )),
    }
}

fn console_error(problem: impl Into<String>) -> Error {
    Error::new(StatusCode::INTERNAL_SERVER_ERROR, problem.into())
}
