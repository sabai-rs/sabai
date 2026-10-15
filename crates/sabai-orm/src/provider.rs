use std::sync::Arc;

use sabai_core::{App, Provider, Result};

use crate::Database;

/// Binds [`Database`] from `config/database.toml`, like Laravel's `DatabaseServiceProvider`.
pub struct DatabaseServiceProvider;

impl Provider for DatabaseServiceProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        let database = Database::from_config(app.config())?;
        app.bind(Arc::new(database));
        Ok(())
    }
}
