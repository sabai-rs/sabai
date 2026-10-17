use std::sync::Arc;

use sabai_core::{App, Provider, Result};

use crate::{Database, DatabaseManager};

/// Binds every connection from `config/database.toml` as a [`DatabaseManager`], and the default
/// one as [`Database`], like Laravel's `DatabaseServiceProvider`.
pub struct DatabaseServiceProvider;

impl Provider for DatabaseServiceProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        let manager = DatabaseManager::from_config(app.config())?;
        app.bind::<Database>(manager.default_connection()?);
        app.bind(Arc::new(manager));
        Ok(())
    }
}
