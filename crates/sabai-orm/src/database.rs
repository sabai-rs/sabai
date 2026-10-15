use std::str::FromStr;
use std::sync::{Once, OnceLock};
use std::time::Duration;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Config, Error, Result};
use sqlx::AnyPool;
use sqlx::any::{AnyConnectOptions, AnyPoolOptions, install_default_drivers};

use crate::Driver;
use crate::config::default_target;

const ACQUIRE_TIMEOUT: Duration = Duration::from_secs(5);

/// The app's database, like Laravel's `DB`: a pool for the default connection in
/// `config/database.toml`. Handlers take it as `db: Service<Database>`.
pub struct Database {
    options: AnyConnectOptions,
    in_memory: bool,
    driver: Driver,
    pool: OnceLock<AnyPool>,
}

impl Database {
    #[cfg(test)]
    pub(crate) fn in_memory() -> Self {
        Self {
            options: AnyConnectOptions::from_str("sqlite::memory:").unwrap(),
            in_memory: true,
            driver: Driver::Sqlite,
            pool: OnceLock::new(),
        }
    }

    /// The default connection from config, checked now; nothing connects until the first query.
    pub fn from_config(config: &Config) -> Result<Self> {
        let target = default_target(config)?;
        let options = AnyConnectOptions::from_str(&target.url).map_err(|error| {
            Error::from(error).context("config/database.toml has an invalid connection")
        })?;
        Ok(Self {
            options,
            in_memory: target.in_memory,
            driver: target.driver,
            pool: OnceLock::new(),
        })
    }

    /// The SQL dialect of the default connection.
    pub fn driver(&self) -> Driver {
        self.driver
    }

    /// The sqlx pool, for queries the query builder cannot write yet.
    // sqlx needs the tokio runtime, which starts after boot.
    pub fn pool(&self) -> &AnyPool {
        self.pool.get_or_init(|| self.new_pool())
    }

    /// Runs `SELECT 1`, so a health check can tell that the database answers.
    pub fn ping(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(self.select_one())
    }

    async fn select_one(&self) -> Result<()> {
        sqlx::query("SELECT 1")
            .execute(self.pool())
            .await
            .map_err(unreachable)?;
        Ok(())
    }

    fn new_pool(&self) -> AnyPool {
        static DRIVERS: Once = Once::new();
        DRIVERS.call_once(install_default_drivers);
        let options = match self.in_memory {
            // Every `:memory:` connection is a separate database.
            true => AnyPoolOptions::new()
                .max_connections(1)
                .idle_timeout(None)
                .max_lifetime(None),
            false => AnyPoolOptions::new().acquire_timeout(ACQUIRE_TIMEOUT),
        };
        options.connect_lazy_with(self.options.clone())
    }
}

pub(crate) fn unreachable(error: sqlx::Error) -> Error {
    Error::from(error).context("could not connect to the database")
}

#[cfg(test)]
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
            runtime
                .block_on(sqlx::query(&sql).execute(database.pool()))
                .unwrap();
        }
    }

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
}
