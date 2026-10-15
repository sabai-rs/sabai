//! The Sabai blog: the example app that uses every framework feature as it lands.
//! `main.rs` serves it; `tests/feature` boots it the same way.

pub mod app;
pub mod routes;

use std::sync::Arc;

use sabai::orm::DatabaseServiceProvider;
use sabai::{App, Config, Result};

use crate::app::providers::AppServiceProvider;

pub use routes::all as routes;

/// Loads `config/` and `.env` from the blog's folder, so it works from any working directory.
pub fn config() -> Result<Config> {
    Config::load(env!("CARGO_MANIFEST_DIR"))
}

/// Boots the blog with its providers, like Laravel's `bootstrap/app.php`.
pub fn app(config: Config) -> Result<Arc<App>> {
    App::builder(config)
        .provider(DatabaseServiceProvider)
        .provider(AppServiceProvider)
        .boot()
}
