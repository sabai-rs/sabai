use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "posts.import";

#[test]
fn import_creates_every_post_in_order() {
    let app = app();

    let response = app.post_json(
        route(ROUTE),
        json!({"posts": [
            {"title": "First", "body": "a"},
            {"title": "Second", "body": "b"},
        ]}),
    );

    response
        .assert_status(201)
        .assert_json_fragment(json!({"id": 1, "title": "First"}))
        .assert_json_fragment(json!({"id": 2, "title": "Second"}));
}

#[test]
fn import_names_the_post_whose_field_is_wrong() {
    let response = app().post_json(
        route(ROUTE),
        json!({"posts": [
            {"title": "Fine", "body": "a"},
            {"title": "  ", "body": "b"},
        ]}),
    );

    response.assert_status(422).assert_json_fragment(json!({
        "posts.1.title": ["The posts.1.title field is required."],
    }));
}

#[test]
fn import_needs_a_list_of_posts() {
    let response = app().post_json(route(ROUTE), json!({"posts": "First"}));

    response.assert_status(422).assert_json_fragment(json!({
        "posts": ["The posts field must be an array."],
    }));
}
