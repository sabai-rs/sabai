use bootstrap::app::http::middleware::FrameGuard;
use sabai::header::{HeaderValue, X_FRAME_OPTIONS};
use sabai::http::Router;
use sabai::{Config, IntoResponse, Response};
use sabai_testing::TestApp;

use crate::app;

#[test]
fn every_response_forbids_other_sites_from_framing_the_blog() {
    app()
        .get_json("/api/health")
        .assert_header("x-frame-options", "SAMEORIGIN");
    app()
        .get_json("/missing")
        .assert_not_found()
        .assert_header("x-frame-options", "SAMEORIGIN");
}

async fn never_framed() -> Response {
    let mut response = "Never framed".into_response();
    let deny = HeaderValue::from_static("DENY");
    response.headers_mut().insert(X_FRAME_OPTIONS, deny);
    response
}

#[test]
fn a_route_that_sets_its_own_frame_options_keeps_them() {
    let mut route = Router::new();
    route.get("/never-framed", never_framed);
    route.global_middleware(FrameGuard);
    let config = Config::load_for(env!("CARGO_MANIFEST_DIR"), "testing").expect("the config loads");

    TestApp::new(bootstrap::app(config).expect("the blog boots"), route)
        .get_json("/never-framed")
        .assert_header("x-frame-options", "DENY");
}

#[test]
fn api_answers_are_never_stored_but_pages_may_be() {
    app()
        .get_json("/api/health")
        .assert_header("cache-control", "no-store");
    app().get_json("/").assert_header_missing("cache-control");
}
