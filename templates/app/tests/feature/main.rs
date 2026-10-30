//! Feature tests, which send requests to the app. They are modules of one test binary,
//! because every file directly under `tests/` compiles on its own.

mod example_test;

use sabai::Config;
use sabai_testing::TestApp;

/// A freshly booted app: `.env.testing` points it at a SQLite
/// database in memory, migrated before each test.
fn app() -> TestApp {
    let config = Config::load_for(env!("CARGO_MANIFEST_DIR"), "testing").expect("the config loads");
    TestApp::new(
        bootstrap::app(config).expect("the app boots"),
        bootstrap::routes(),
    )
    .refresh_database(&bootstrap::migrations())
}
