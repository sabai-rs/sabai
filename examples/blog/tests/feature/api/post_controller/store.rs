use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "posts.store";

#[test]
fn store_creates_a_post() {
    let response = app().post_json(
        route(ROUTE),
        json!({"title": "Sabai", "body": "Rust, but sabai"}),
    );

    response
        .assert_status(201)
        .assert_json_fragment(json!({"id": 1, "title": "Sabai"}));
}

#[test]
fn store_rejects_a_post_without_a_title() {
    let response = app().post_json(route(ROUTE), json!({"body": "Rust, but sabai"}));

    response.assert_status(422);
}
