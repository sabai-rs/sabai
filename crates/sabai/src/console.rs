//! The app's own commands: `serve`, `migrate` and the commands providers add.

use std::collections::HashSet;
use std::sync::Arc;

use sabai_core::{App, Command, Error, Result, StatusCode};
use sabai_http::{RouteListCommand, Router, ServeCommand};

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
        let route_list = RouteListCommand::new(&routes);
        self.with(Box::new(ServeCommand::new(routes)))
            .with(Box::new(route_list))
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
        let command = self
            .find(name)
            .ok_or_else(|| self.command_not_found(name))?;
        let args = args.get(1..).unwrap_or_default();
        if args
            .iter()
            .any(|arg| matches!(arg.as_str(), "--help" | "-h"))
        {
            print!("{}", command_help(command));
            return Ok(());
        }
        reject_unknown_options(command, args)?;
        command.handle(&self.app, args)
    }

    /// What `--help` prints: Sabai's commands, then the app's and packages'.
    pub fn help(&self) -> String {
        let commands: Vec<(&str, &str)> = self
            .commands
            .iter()
            .map(|command| described(command.as_ref()))
            .chain([("list", "Show these commands")])
            .chain(self.app.commands().map(described))
            .collect();
        format!(
            "Usage: sabai <command> [arguments]\n\nCommands:\n{}",
            two_columns(&commands)
        )
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

    fn command_not_found(&self, name: &str) -> Error {
        match name {
            "serve" => console_error(
                "`serve` has no routes; add them with `Console::new(app).routes(...)`",
            ),
            "migrate" => console_error(
                "`migrate` has no migrations; add them with `.migrations(...)` (needs the `orm` feature)",
            ),
            _ => console_error(format!(
                "there is no `{name}` command; {}",
                self.did_you_mean(name)
            )),
        }
    }

    fn did_you_mean(&self, typed: &str) -> String {
        let names = self.all().map(Command::name).chain(["list"]);
        let similar: Vec<String> = names
            .filter(|name| looks_like(typed, name))
            .map(|name| format!("`{name}`"))
            .collect();
        if similar.is_empty() {
            return String::from("run `sabai list` to see them");
        }
        format!("did you mean {}?", similar.join(" or "))
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

fn command_help(command: &dyn Command) -> String {
    let options: Vec<(&str, &str)> = command
        .options()
        .iter()
        .map(|option| (option.name, option.description))
        .chain([("-h, --help", "Show this help")])
        .collect();
    format!(
        "{}\n\nUsage: sabai {} [options] [arguments]\n\nOptions:\n{}",
        command.description(),
        command.name(),
        two_columns(&options)
    )
}

fn two_columns(rows: &[(&str, &str)]) -> String {
    let width = rows.iter().map(|(name, _)| name.len()).max().unwrap_or(0) + 3;
    rows.iter()
        .map(|(name, description)| format!("  {name:width$}{description}\n"))
        .collect()
}

// A command never sees an option it does not take, so `migrate --pretend` cannot migrate for real.
fn reject_unknown_options(command: &dyn Command, args: &[String]) -> Result<()> {
    let mut names = args
        .iter()
        .filter(|arg| arg.starts_with('-'))
        .map(|arg| arg.split_once('=').map_or(arg.as_str(), |(name, _)| name));
    let takes = |name: &str| command.options().iter().any(|option| option.name == name);
    match names.find(|name| !takes(name)) {
        Some(unknown) => Err(console_error(format!(
            "`{0}` has no option `{unknown}`; run `sabai {0} --help` to see its options",
            command.name()
        ))),
        None => Ok(()),
    }
}

// Symfony Console's rule: a typo of up to a third of the name,
// or a name that contains what was typed.
fn looks_like(typed: &str, name: &str) -> bool {
    name.contains(typed) || edit_distance(typed, name) * 3 <= typed.len()
}

/// How many single-character inserts, deletes or swaps turn `from` into `to` (Levenshtein).
fn edit_distance(from: &str, to: &str) -> usize {
    let to: Vec<char> = to.chars().collect();
    let mut previous: Vec<usize> = (0..=to.len()).collect();
    for (i, from_char) in from.chars().enumerate() {
        let mut current = vec![i + 1];
        for (j, to_char) in to.iter().enumerate() {
            let swap = previous[j] + usize::from(from_char != *to_char);
            current.push(swap.min(previous[j + 1] + 1).min(current[j] + 1));
        }
        previous = current;
    }
    previous[to.len()]
}

fn console_error(problem: impl Into<String>) -> Error {
    Error::new(StatusCode::INTERNAL_SERVER_ERROR, problem.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sabai_core::CommandOption;

    struct Deploy;

    impl Command for Deploy {
        fn name(&self) -> &str {
            "deploy"
        }

        fn description(&self) -> &str {
            "Ship the app"
        }

        fn options(&self) -> &[CommandOption] {
            &[CommandOption {
                name: "--force",
                description: "Run in production",
            }]
        }

        fn handle(&self, _app: &Arc<App>, _args: &[String]) -> Result<()> {
            Ok(())
        }
    }

    #[test]
    fn a_command_help_lists_its_options_then_help_itself() {
        assert_eq!(
            command_help(&Deploy),
            "Ship the app\n\nUsage: sabai deploy [options] [arguments]\n\nOptions:\n\
             \x20 --force      Run in production\n\
             \x20 -h, --help   Show this help\n"
        );
    }
}
