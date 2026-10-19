use std::sync::Arc;

use sabai_core::{App, Command, Result};

use crate::{Database, Migrations};

/// `migrate`: runs the pending migrations, like `php artisan migrate`.
pub struct MigrateCommand {
    migrations: Migrations,
}

impl MigrateCommand {
    /// A `migrate` command for `migrations`.
    pub fn new(migrations: Migrations) -> Self {
        Self { migrations }
    }
}

impl Command for MigrateCommand {
    fn name(&self) -> &str {
        "migrate"
    }

    fn description(&self) -> &str {
        "Run the migrations that have not run yet"
    }

    fn handle(&self, app: &Arc<App>, _args: &[String]) -> Result<()> {
        migrate(app, &self.migrations)
    }
}

/// Runs the pending migrations and prints each one, like `php artisan migrate`.
pub fn migrate(app: &Arc<App>, migrations: &Migrations) -> Result<()> {
    let database = app.get::<Database>()?;
    let ran = sabai_core::block_on(app.scope(database.migrate(migrations)))??;
    if ran.is_empty() {
        println!("Nothing to migrate.");
        return Ok(());
    }
    println!("Running migrations.");
    for name in ran {
        println!("  {name} DONE");
    }
    Ok(())
}
