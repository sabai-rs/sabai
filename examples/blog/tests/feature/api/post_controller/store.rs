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
fn store_requires_a_title_and_a_body() {
    let response = app().post_json(route(ROUTE), json!({}));

    response.assert_status(422).assert_json_fragment(json!({
        "title": ["The title field is required."],
        "body": ["The body field is required."],
    }));
}

#[test]
fn store_rejects_a_title_longer_than_255_characters() {
    let title = "a".repeat(256);

    let response = app().post_json(route(ROUTE), json!({"title": title, "body": "b"}));

    response.assert_status(422).assert_json_fragment(json!({
        "title": ["The title field must not be greater than 255 characters."],
    }));
}

#[test]
fn store_rejects_a_body_that_is_not_a_string() {
    let response = app().post_json(route(ROUTE), json!({"title": "Sabai", "body": 7}));

    response
        .assert_status(422)
        .assert_json_fragment(json!({"body": ["The body field must be a string."]}));
}
