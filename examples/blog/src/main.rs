//! The Sabai blog: the example app that uses every framework feature as it lands.

mod models;
mod requests;
mod routes;

use sabai::log::Logging;
use sabai::{App, Config, Result};

fn main() -> Result<()> {
    // The manifest directory, not ".", so `cargo run -p blog` works from the workspace root.
    let config = Config::load(env!("CARGO_MANIFEST_DIR"))?;
    Logging::new(&config.get_or("logging.level", String::from("info"))?)?.init()?;
    let app = App::builder(config).boot()?;
    sabai::http::serve(app, routes::all())
}
