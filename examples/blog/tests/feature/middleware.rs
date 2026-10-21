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

#[test]
fn api_answers_are_never_stored_but_pages_may_be() {
    app()
        .get_json("/api/health")
        .assert_header("cache-control", "no-store");
    app().get_json("/").assert_header_missing("cache-control");
}

#[test]
fn every_response_carries_its_request_id_from_the_load_balancer() {
    app()
        .with_header("x-request-id", "lb-7f3a")
        .get_json("/api/health")
        .assert_header("x-request-id", "lb-7f3a");
}
