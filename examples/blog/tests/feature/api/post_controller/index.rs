use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "posts.index";

#[test]
fn index_returns_every_post() {
    let app = app();
    app.post_json(route("posts.store"), json!({"title": "First", "body": "a"}));
    app.post_json(
        route("posts.store"),
        json!({"title": "Second", "body": "b"}),
    );

    let response = app.get_json(route(ROUTE));

    response
        .assert_status(200)
        .assert_json_fragment(json!({"id": 1, "title": "First"}))
        .assert_json_fragment(json!({"id": 2, "title": "Second"}));
}

#[test]
fn index_returns_an_empty_list_when_there_are_no_posts() {
    let response = app().get_json(route(ROUTE));

    response.assert_status(200);
    assert_eq!(response.json(), json!([]));
}
