//! The app's code. `main.rs` runs its commands; tests boot it the same way.

pub mod app;
pub mod routes;

use std::sync::Arc;

use sabai::{App, Config, Result};

use crate::app::providers::AppServiceProvider;

pub use routes::all as routes;

/// Loads `config/` and `.env` from the app's folder, so it works from any working directory.
pub fn config() -> Result<Config> {
    Config::load(env!("CARGO_MANIFEST_DIR"))
}

/// Boots the app with its providers, like Laravel's `bootstrap/app.php`.
pub fn app(config: Config) -> Result<Arc<App>> {
    App::builder(config).provider(AppServiceProvider).boot()
}
