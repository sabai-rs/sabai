use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "stats.index";

#[test]
fn index_counts_every_read_of_a_post() {
    let app = app();
    app.post_json(route("posts.store"), json!({"title": "Sabai", "body": "b"}));
    app.get_json(route("posts.show").with("id", 1));
    app.get_json(route("posts.show").with("id", 1));

    let response = app.get_json(route(ROUTE));

    response
        .assert_status(200)
        .assert_json_fragment(json!({"post_views": 2}));
}

#[test]
fn a_post_that_does_not_exist_is_not_counted() {
    let app = app();
    app.get_json(route("posts.show").with("id", 99));

    let response = app.get_json(route(ROUTE));

    response.assert_json_fragment(json!({"post_views": 0}));
}
