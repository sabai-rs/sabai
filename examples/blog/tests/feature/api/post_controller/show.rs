use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "posts.show";

#[test]
fn show_returns_the_post() {
    let app = app();
    app.post_json(
        route("posts.store"),
        json!({"title": "Sabai", "body": "Rust, but sabai"}),
    );

    let response = app.get_json(route(ROUTE).with("id", 1));

    response
        .assert_status(200)
        .assert_json_fragment(json!({"id": 1, "body": "Rust, but sabai"}));
}

#[test]
fn show_returns_404_for_a_post_that_does_not_exist() {
    let response = app().get_json(route(ROUTE).with("id", 99));

    response.assert_status(404);
}
