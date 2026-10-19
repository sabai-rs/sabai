use crate::app;

#[test]
fn the_application_returns_a_successful_response() {
    app().get_json("/").assert_status(200);
}
