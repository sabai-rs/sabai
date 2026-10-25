use bootstrap::app::models::User;
use sabai::orm::Model;
use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "login";

// From PHP's manual for `password_verify`: a hash PHP made, for "rasmuslerdorf", with 10 rounds.
const PHP_HASH: &str = "$2y$10$.vGA1O9wmRjrwAVXD98HNOgsNpDczlqm3Jq7KnEd1rVAGv3Fykk1a";

#[test]
fn login_signs_in_with_the_right_password_and_upgrades_an_old_hash() {
    let app = app();
    let mut user = User {
        name: "Test User".to_owned(),
        email: "test@example.com".to_owned(),
        password: PHP_HASH.to_owned(),
        ..Default::default()
    };
    app.run(user.save()).expect("the user saves");

    let response = app.post_json(
        route(ROUTE),
        json!({"email": "test@example.com", "password": "rasmuslerdorf"}),
    );

    response
        .assert_ok()
        .assert_json_fragment(json!({"email": "test@example.com"}));
    let user = app.run(User::find_or_fail(1)).expect("the user exists");
    assert!(user.password.starts_with("$2y$04$"), "{}", user.password);
}

#[test]
fn login_refuses_a_wrong_password() {
    let app = app();
    let mut user = User {
        name: "Test User".to_owned(),
        email: "test@example.com".to_owned(),
        password: PHP_HASH.to_owned(),
        ..Default::default()
    };
    app.run(user.save()).expect("the user saves");

    let response = app.post_json(
        route(ROUTE),
        json!({"email": "test@example.com", "password": "wrong"}),
    );

    response.assert_status(422).assert_json_fragment(json!({
        "email": ["These credentials do not match our records."],
    }));
}

#[test]
fn login_refuses_an_unknown_email_with_the_same_message() {
    let response = app().post_json(
        route(ROUTE),
        json!({"email": "nobody@example.com", "password": "rasmuslerdorf"}),
    );

    response.assert_status(422).assert_json_fragment(json!({
        "email": ["These credentials do not match our records."],
    }));
}
