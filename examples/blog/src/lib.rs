//! The Sabai blog: the example app that uses every framework feature as it lands.
//! `main.rs` serves it; `tests/feature` boots it the same way.

pub mod app;
pub mod database;
pub mod routes;

use std::sync::Arc;

use sabai::http::Router;
use sabai::http::middleware::{ConvertEmptyStringsToNull, TrimStrings};
use sabai::orm::DatabaseServiceProvider;
use sabai::{App, Config, Result};

use crate::app::http::middleware::FrameGuard;
use crate::app::providers::AppServiceProvider;

pub use database::migrations::all as migrations;

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

/// Every route, and the middleware that runs on every request, like `withRouting` and
/// `withMiddleware` in Laravel's `bootstrap/app.php`.
pub fn routes() -> Router {
    let mut route = routes::all();
    route
        .global_middleware(TrimStrings)
        .global_middleware(ConvertEmptyStringsToNull)
        .global_middleware(FrameGuard);
    route
}
