use std::fmt;
use std::sync::Arc;

use crate::{App, Error, Result, StatusCode};

/// A console command from the app or a package, like a Laravel `Command` class.
/// A provider adds it with [`App::command`].
///
/// ```ignore
/// impl Command for Inspire {
///     fn name(&self) -> &str { "inspire" }
///     fn description(&self) -> &str { "Show an inspiring quote" }
///     fn handle(&self, _app: &Arc<App>, _args: &[String]) -> Result<()> {
///         println!("Simplicity is the ultimate sophistication.");
///         Ok(())
///     }
/// }
/// ```
pub trait Command: Send + Sync {
    /// The name typed on the command line, like `inspire` or `cache:clear`.
    fn name(&self) -> &str;

    /// One line shown by `list`.
    fn description(&self) -> &str;

    /// Runs the command with the arguments typed after its name.
    fn handle(&self, app: &Arc<App>, args: &[String]) -> Result<()>;
}

/// The commands providers added, in the order they were added.
#[derive(Default)]
pub(crate) struct Commands(Vec<Box<dyn Command>>);

impl Commands {
    pub(crate) fn add(&mut self, command: Box<dyn Command>) -> Result<()> {
        if self.find(command.name()).is_some() {
            return Err(Error::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                format!(
                    "two commands are named `{}`; a package and the app may both add it, so rename one",
                    command.name()
                ),
            ));
        }
        self.0.push(command);
        Ok(())
    }

    pub(crate) fn find(&self, name: &str) -> Option<&dyn Command> {
        self.iter().find(|command| command.name() == name)
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = &dyn Command> {
        self.0.iter().map(Box::as_ref)
    }
}

impl fmt::Debug for Commands {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_list()
            .entries(self.iter().map(Command::name))
            .finish()
    }
}
