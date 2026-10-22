//! Validation rules for a [`FormRequest`](crate::FormRequest), like the array its `rules()` returns
//! in Laravel: `Rules::new().field("title", [required(), string(), max(255)])`.

mod catalog;
mod checks;
mod email;
mod fields;

use sabai_core::{Error, Result};
use serde_json::{Map, Value};

use checks::{Check, check_field};
use fields::matching_fields;

pub use catalog::{
    array, confirmed, email, lowercase, max, max_bytes, min, required, sometimes, string,
};

/// The rules of each field, checked in the order they were added.
#[derive(Default)]
pub struct Rules {
    fields: Vec<(String, Vec<Rule>)>,
}

/// One rule, made by [`required`], [`string`], [`email`], [`min`], [`confirmed`] and the others here.
pub struct Rule(Check);

impl Rules {
    /// No rules yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the rules of one field: `.field("title", [required(), string()])`. A name reaches into
    /// objects with `.` and into every item of a list with `*`: `posts.*.title`.
    pub fn field(self, name: &str, rules: impl IntoIterator<Item = Rule>) -> Self {
        self.push_field(name, rules.into_iter().collect())
    }

    fn push_field(mut self, name: &str, rules: Vec<Rule>) -> Self {
        self.fields.push((name.to_owned(), rules));
        self
    }

    /// Checks `input`, such as `request.all()`, and fails with a `422` listing every message per field.
    pub fn validate(&self, input: &Map<String, Value>) -> Result<()> {
        let mut messages = Vec::new();
        for (pattern, rules) in &self.fields {
            for (name, value) in matching_fields(pattern, input) {
                check_field(&name, value, rules, input, &mut messages);
            }
        }
        if messages.is_empty() {
            return Ok(());
        }
        let messages: Vec<_> = messages
            .iter()
            .map(|(field, message)| (field.as_str(), message.as_str()))
            .collect();
        Err(Error::with_messages(&messages))
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn login_rules() -> Rules {
        Rules::new()
            .field("username", [required(), string(), max(10)])
            .field("device_uuid", [sometimes(), string()])
    }

    fn errors(input: Value) -> Vec<(String, Vec<String>)> {
        match login_rules().validate(input.as_object().unwrap()) {
            Ok(()) => Vec::new(),
            Err(error) => error.errors().to_vec(),
        }
    }

    fn field(name: &str, message: &str) -> (String, Vec<String>) {
        (name.to_owned(), vec![message.to_owned()])
    }

    #[test]
    fn valid_input_passes_and_an_optional_field_may_be_left_out() {
        assert!(errors(json!({"username": "test"})).is_empty());
        assert!(errors(json!({"username": "test", "device_uuid": "abc"})).is_empty());
    }

    #[test]
    fn a_missing_null_or_blank_required_field_gets_only_the_required_message() {
        let required = vec![field("username", "The username field is required.")];

        assert_eq!(errors(json!({})), required);
        assert_eq!(errors(json!({"username": null})), required);
        assert_eq!(errors(json!({"username": "   "})), required);
    }

    #[test]
    fn a_wrong_type_or_a_long_value_says_what_is_wrong_with_spaces_in_the_name() {
        assert_eq!(
            errors(json!({"username": "test", "device_uuid": 7})),
            vec![field(
                "device_uuid",
                "The device uuid field must be a string."
            )]
        );
        assert_eq!(
            errors(json!({"username": "testuser123"})),
            vec![field(
                "username",
                "The username field must not be greater than 10 characters."
            )]
        );
    }

    #[test]
    fn max_counts_characters_not_bytes_and_compares_numbers_and_lists() {
        let rules = Rules::new()
            .field("name", [max(3)])
            .field("age", [max(3)])
            .field("tags", [max(3)]);

        let thai = json!({"name": "ไทย", "age": 3, "tags": [1, 2, 3]});
        let over = json!({"age": 4, "tags": [1, 2, 3, 4]});
        let thai = rules.validate(thai.as_object().unwrap());
        let over = rules.validate(over.as_object().unwrap());

        assert!(thai.is_ok());
        assert_eq!(
            over.err().unwrap().errors(),
            [
                field("age", "The age field must not be greater than 3."),
                field("tags", "The tags field must not have more than 3 items."),
            ]
        );
    }

    fn import_errors(input: Value) -> Vec<(String, Vec<String>)> {
        let rules = Rules::new()
            .field("posts", [required(), array(), max(2)])
            .field("posts.*.title", [required(), string(), max(5)])
            .field("author.name", [required()]);
        match rules.validate(input.as_object().unwrap()) {
            Ok(()) => Vec::new(),
            Err(error) => error.errors().to_vec(),
        }
    }

    #[test]
    fn a_star_checks_every_item_and_names_the_one_that_failed() {
        let input = json!({
            "posts": [{"title": "Sabai"}, {"title": "Too long"}],
            "author": {"name": "Test User"},
        });

        assert_eq!(
            import_errors(input),
            vec![field(
                "posts.1.title",
                "The posts.1.title field must not be greater than 5 characters."
            )]
        );
    }

    #[test]
    fn a_missing_nested_field_is_required_by_its_full_name() {
        let input = json!({"posts": [{"body": "no title"}], "author": {}});

        assert_eq!(
            import_errors(input),
            vec![
                field("posts.0.title", "The posts.0.title field is required."),
                field("author.name", "The author.name field is required."),
            ]
        );
    }

    #[test]
    fn a_star_over_something_that_is_not_a_list_checks_nothing_and_array_says_so() {
        let input = json!({"posts": "no", "author": {"name": "Test User"}});

        assert_eq!(
            import_errors(input),
            vec![field("posts", "The posts field must be an array.")]
        );
    }

    // Laravel Breeze's `RegisteredUserController@store`, with `Password::defaults()`'s minimum.
    fn registration_errors(input: Value) -> Vec<(String, Vec<String>)> {
        let rules = Rules::new()
            .field("name", [required(), string(), max(255)])
            .field(
                "email",
                [required(), string(), lowercase(), email(), max(255)],
            )
            .field("password", [required(), confirmed(), min(8)]);
        match rules.validate(input.as_object().unwrap()) {
            Ok(()) => Vec::new(),
            Err(error) => error.errors().to_vec(),
        }
    }

    #[test]
    fn a_valid_registration_passes() {
        let input = json!({
            "name": "Test User",
            "email": "test@example.com",
            "password": "correct horse",
            "password_confirmation": "correct horse",
        });

        assert!(registration_errors(input).is_empty());
    }

    #[test]
    fn a_bad_registration_gets_a_message_for_each_broken_rule() {
        let input = json!({
            "name": "Test User",
            "email": "Test@Example",
            "password": "short",
            "password_confirmation": "shorter",
        });

        assert_eq!(
            registration_errors(input),
            vec![
                field("email", "The email field must be lowercase."),
                (
                    "password".to_owned(),
                    vec![
                        "The password field confirmation does not match.".to_owned(),
                        "The password field must be at least 8 characters.".to_owned(),
                    ]
                ),
            ]
        );
    }

    #[test]
    fn a_missing_confirmation_does_not_match() {
        let input =
            json!({"name": "Test User", "email": "test@example.com", "password": "correct horse"});

        assert_eq!(
            registration_errors(input),
            vec![field(
                "password",
                "The password field confirmation does not match."
            )]
        );
    }

    #[test]
    fn a_nested_field_is_confirmed_by_its_own_sibling() {
        let rules = Rules::new().field("users.*.password", [confirmed()]);
        let input = json!({"users": [
            {"password": "a", "password_confirmation": "a"},
            {"password": "b", "password_confirmation": "c"},
        ]});

        let error = rules.validate(input.as_object().unwrap()).unwrap_err();

        assert_eq!(
            error.errors(),
            [field(
                "users.1.password",
                "The users.1.password field confirmation does not match."
            )]
        );
    }

    #[test]
    fn min_counts_characters_and_compares_numbers_and_lists() {
        let rules = Rules::new()
            .field("name", [min(3)])
            .field("age", [min(18)])
            .field("tags", [min(2)]);
        let input = json!({"name": "ไท", "age": 17, "tags": [1]});

        let error = rules.validate(input.as_object().unwrap()).unwrap_err();

        assert_eq!(
            error.errors(),
            [
                field("name", "The name field must be at least 3 characters."),
                field("age", "The age field must be at least 18."),
                field("tags", "The tags field must have at least 2 items."),
            ]
        );
    }

    #[test]
    fn max_bytes_counts_bytes_so_a_thai_password_fits_in_fewer_letters() {
        let rules = Rules::new().field("password", [max_bytes(72)]);
        let fits = json!({"password": "ก".repeat(24)});
        let too_long = json!({"password": "ก".repeat(25)});

        let error = rules.validate(too_long.as_object().unwrap()).unwrap_err();

        assert!(rules.validate(fits.as_object().unwrap()).is_ok());
        assert_eq!(
            error.errors(),
            [field(
                "password",
                "The password field must not be greater than 72 bytes."
            )]
        );
    }
}
