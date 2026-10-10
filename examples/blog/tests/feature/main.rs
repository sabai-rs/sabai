//! Feature tests, like Laravel's `tests/Feature`: one file per controller action. They are
//! modules of one test binary, because every file directly under `tests/` compiles on its own.

mod api;

use sabai_testing::TestApp;

/// A freshly booted blog, like Laravel's `TestCase`: every test starts with no posts.
fn app() -> TestApp {
    let config = blog::config().expect("the config loads");
    TestApp::new(blog::app(config).expect("the blog boots"), blog::routes())
}
