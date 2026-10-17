use std::collections::HashMap;
use std::sync::Arc;

use sabai_core::{Config, Error, Result, StatusCode};

use crate::Database;
use crate::config::targets;

/// Every connection in `config/database.toml`, like Laravel's `DB::connection('report')`.
/// Each has its own pool, opened on first use.
pub struct DatabaseManager {
    default: String,
    connections: HashMap<String, Arc<Database>>,
}

impl DatabaseManager {
    /// Reads every connection, checked now; nothing connects until the first query.
    pub fn from_config(config: &Config) -> Result<Self> {
        let (default, targets) = targets(config)?;
        let connections = targets
            .into_iter()
            .map(|(name, target)| Ok((name, Arc::new(Database::from_target(target)?))))
            .collect::<Result<_>>()?;
        Ok(Self {
            default,
            connections,
        })
    }

    /// The connection named `name`, as `#[model(connection = "report")]` uses it.
    pub fn connection(&self, name: &str) -> Result<Arc<Database>> {
        self.connections.get(name).cloned().ok_or_else(|| {
            Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(format!(
                "no database connection is named `{name}`; add `[connections.{name}]` to config/database.toml"
            ))
        })
    }

    /// The connection `default` names.
    pub fn default_connection(&self) -> Result<Arc<Database>> {
        self.connection(&self.default)
    }
}
