use bootstrap::app::models::User;
use sabai::auth::Hash;
use sabai::orm::Model;
use sabai_testing::{json, route};

use crate::app;

const ROUTE: &str = "register";

#[test]
fn store_creates_a_user_with_a_hashed_password_it_never_shows() {
    let app = app();

    let response = app.post_json(
        route(ROUTE),
        json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "correct horse",
            "password_confirmation": "correct horse",
        }),
    );

    response
        .assert_created()
        .assert_json_fragment(json!({"id": 1, "email": "test@example.com"}));
    assert!(
        response.json().get("password").is_none(),
        "{}",
        response.text()
    );
    let user = app.run(User::find_or_fail(1)).expect("the user exists");
    assert!(user.password.starts_with("$2y$04$"), "{}", user.password);
    assert!(app.run(Hash::check("correct horse", &user.password)));
}

#[test]
fn store_refuses_an_email_that_is_already_taken() {
    let app = app();
    let account = json!({
        "name": "Test User",
        "email": "test@example.com",
        "password": "correct horse",
        "password_confirmation": "correct horse",
    });
    app.post_json(route(ROUTE), &account).assert_created();

    let response = app.post_json(route(ROUTE), &account);

    response.assert_status(422).assert_json_fragment(json!({
        "email": ["The email has already been taken."],
    }));
}

#[test]
fn store_explains_a_weak_or_unconfirmed_password() {
    let response = app().post_json(
        route(ROUTE),
        json!({"name": "Test User", "email": "test@example.com", "password": "short"}),
    );

    response.assert_status(422).assert_json_fragment(json!({
        "password": [
            "The password field confirmation does not match.",
            "The password field must be at least 8 characters.",
        ],
    }));
}

#[test]
fn store_refuses_a_password_longer_than_bcrypt_reads_on_its_field() {
    let password = "ก".repeat(25);

    let response = app().post_json(
        route(ROUTE),
        json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": password,
            "password_confirmation": password,
        }),
    );

    response.assert_status(422).assert_json_fragment(json!({
        "password": ["The password field must not be greater than 72 bytes."],
    }));
}
