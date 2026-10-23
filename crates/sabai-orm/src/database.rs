use std::sync::OnceLock;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Config, Error, Result};

use crate::Driver;
use crate::config::{Target, targets};
use crate::pool::{Options, Pool};

/// The app's database, like Laravel's `DB`: a pool for the default connection in
/// `config/database.toml`. Handlers take it as `db: Service<Database>`.
pub struct Database {
    options: Options,
    driver: Driver,
    pool: OnceLock<Pool>,
}

impl Database {
    #[cfg(all(test, feature = "sqlite"))]
    pub(crate) fn in_memory() -> Self {
        Self::from_target(Target {
            url: crate::config::SQLITE_MEMORY.to_owned(),
            driver: Driver::Sqlite,
        })
        .expect("an in-memory SQLite database is always valid")
    }

    /// The default connection from config, checked now; nothing connects until the first query.
    pub fn from_config(config: &Config) -> Result<Self> {
        let (default, mut targets) = targets(config)?;
        let index = targets
            .iter()
            .position(|(name, _)| *name == default)
            .unwrap_or(0);
        Self::from_target(targets.swap_remove(index).1)
    }

    pub(crate) fn from_target(target: Target) -> Result<Self> {
        Ok(Self {
            options: Options::parse(&target)?,
            driver: target.driver,
            pool: OnceLock::new(),
        })
    }

    /// The SQL dialect of the default connection.
    pub fn driver(&self) -> Driver {
        self.driver
    }

    // sqlx needs the tokio runtime, which starts after boot.
    pub(crate) fn pool(&self) -> Result<&Pool> {
        if let Some(pool) = self.pool.get() {
            return Ok(pool);
        }
        let pool = self.options.lazy_pool()?;
        Ok(self.pool.get_or_init(|| pool))
    }

    /// Runs one SQL statement as written, like Laravel's `DB::statement(...)`.
    pub fn statement<'a>(&'a self, sql: &'a str) -> BoxFuture<'a, Result<()>> {
        Box::pin(self.execute(sql))
    }

    async fn execute(&self, sql: &str) -> Result<()> {
        self.pool()?.execute(sql, Vec::new()).await?;
        Ok(())
    }

    /// Runs `SELECT 1`, so a health check can tell that the database answers.
    pub fn ping(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(self.select_one())
    }

    async fn select_one(&self) -> Result<()> {
        self.pool()?
            .fetch_all("SELECT 1", Vec::new())
            .await
            .map_err(unreachable)?;
        Ok(())
    }
}

pub(crate) fn unreachable(error: Error) -> Error {
    error.context("could not connect to the database")
}

#[cfg(test)]
#[cfg(feature = "sqlite")]
mod tests {
    use std::path::PathBuf;
    use std::{fs, process};

    use super::*;

    fn config_with(name: &str, database_toml: &str) -> Config {
        let base: PathBuf =
            std::env::temp_dir().join(format!("sabai-orm-{name}-{}", process::id()));
        fs::create_dir_all(base.join("config")).unwrap();
        fs::write(base.join("config/database.toml"), database_toml).unwrap();
        Config::load(&base).unwrap()
    }

    #[test]
    fn an_sqlite_memory_database_answers_a_ping() {
        let config = config_with(
            "ping",
            "default = \"sqlite\"\n[connections.sqlite]\ndriver = \"sqlite\"\ndatabase = \":memory:\"\n",
        );
        let database = Database::from_config(&config).unwrap();
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(database.ping()).unwrap();
    }

    #[test]
    fn sqlite_runs_the_schema_builders_sql() {
        let config = config_with(
            "schema",
            "default = \"sqlite\"\n[connections.sqlite]\ndriver = \"sqlite\"\ndatabase = \":memory:\"\n",
        );
        let database = Database::from_config(&config).unwrap();
        let mut schema = crate::Schema::default();
        schema.create("posts", |table| {
            table.id();
            table.string("title");
            table.timestamps();
        });
        schema.drop_if_exists("posts");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        for sql in schema.to_sql(crate::Driver::Sqlite) {
            runtime.block_on(database.statement(&sql)).unwrap();
        }
    }

    #[cfg(feature = "mysql")]
    #[test]
    fn a_numeric_password_from_the_env_is_read_as_text() {
        let config = config_with(
            "numeric",
            "default = \"mysql\"\n[connections.mysql]\ndriver = \"mysql\"\nhost = \"127.0.0.1\"\nport = 3306\ndatabase = \"blog\"\nusername = \"sabai\"\npassword = 12345\n",
        );

        assert!(Database::from_config(&config).is_ok());
    }

    #[test]
    fn a_default_without_a_connection_says_which_one_is_missing() {
        let config = config_with("missing", "default = \"mysql\"\n[connections]\n");

        let error = Database::from_config(&config).err().unwrap().to_string();

        assert!(
            error.contains("`default` is `mysql`, but there is no `[connections.mysql]`"),
            "{error}"
        );
    }

    #[cfg(not(feature = "mysql"))]
    #[test]
    fn a_driver_built_without_its_feature_fails_when_used_and_says_how_to_add_it() {
        let config = config_with(
            "no-mysql",
            "default = \"mysql\"\n[connections.mysql]\ndriver = \"mysql\"\nhost = \"127.0.0.1\"\nport = 3306\ndatabase = \"blog\"\nusername = \"sabai\"\npassword = \"secret\"\n",
        );

        let database = Database::from_config(&config).unwrap();
        let error = database.pool().err().unwrap().to_string();

        assert!(
            error.contains(
                "add the `mysql` feature: sabai = { ..., features = [\"orm\", \"mysql\"] }"
            ),
            "{error}"
        );
    }
}
