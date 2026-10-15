//! Feature tests, like Laravel's `tests/Feature`: one file per controller action. They are
//! modules of one test binary, because every file directly under `tests/` compiles on its own.

mod api;

use sabai::Config;
use sabai_testing::TestApp;

/// A freshly booted blog, like Laravel's `TestCase`: every test starts with no posts, and
/// `.env.testing` points it at an empty SQLite database in memory.
fn app() -> TestApp {
    let config = Config::load_for(env!("CARGO_MANIFEST_DIR"), "testing").expect("the config loads");
    TestApp::new(blog::app(config).expect("the blog boots"), blog::routes())
}
