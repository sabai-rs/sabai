use std::sync::Arc;

use sabai_core::{App, Result};

use crate::{Database, Migrations};

/// Runs the pending migrations and prints each one, like `php artisan migrate`.
pub fn migrate(app: &Arc<App>, migrations: &Migrations) -> Result<()> {
    let database = app.get::<Database>()?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let ran = runtime.block_on(app.scope(database.migrate(migrations)))?;
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
