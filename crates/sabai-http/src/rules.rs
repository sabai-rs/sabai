//! Validation rules for a [`FormRequest`](crate::FormRequest), like the array its `rules()` returns
//! in Laravel: `Rules::new().field("title", [required(), string(), max(255)])`.

use sabai_core::{Error, Result};
use serde_json::Value;

/// The rules of each field, checked in the order they were added.
#[derive(Default)]
pub struct Rules {
    fields: Vec<(String, Vec<Rule>)>,
}

/// One rule, made by [`required`], [`sometimes`], [`string`] or [`max`].
pub struct Rule(Check);

// Private, so custom rules (M1-55) can be added without breaking apps.
enum Check {
    Required,
    Sometimes,
    String,
    Max(u64),
}

/// The field must be present and not empty: not missing, `null`, `""`, `[]` or `{}`.
pub fn required() -> Rule {
    Rule(Check::Required)
}

/// Check the field only when it is present; it may be left out.
pub fn sometimes() -> Rule {
    Rule(Check::Sometimes)
}

/// The field must be a string.
pub fn string() -> Rule {
    Rule(Check::String)
}

/// At most `limit` characters for a string, `limit` for a number, `limit` items for a list.
pub fn max(limit: u64) -> Rule {
    Rule(Check::Max(limit))
}

impl Rules {
    /// No rules yet.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds the rules of one field: `.field("title", [required(), string()])`.
    pub fn field(self, name: &str, rules: impl IntoIterator<Item = Rule>) -> Self {
        self.push_field(name, rules.into_iter().collect())
    }

    fn push_field(mut self, name: &str, rules: Vec<Rule>) -> Self {
        self.fields.push((name.to_owned(), rules));
        self
    }

    /// Checks `input`, a JSON object, and fails with a `422` listing every message per field.
    pub fn validate(&self, input: &Value) -> Result<()> {
        let mut messages = Vec::new();
        for (name, rules) in &self.fields {
            check_field(name, input.get(name), rules, &mut messages);
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

fn check_field(
    name: &str,
    value: Option<&Value>,
    rules: &[Rule],
    messages: &mut Vec<(String, String)>,
) {
    let attribute = name.replace('_', " ");
    for Rule(check) in rules {
        if let Check::Required = check {
            if is_blank(value) {
                messages.push((
                    name.to_owned(),
                    format!("The {attribute} field is required."),
                ));
                // Like Laravel, a missing field gets one message, not one per rule.
                return;
            }
            continue;
        }
        // Like Laravel, other rules skip a field that is missing or an empty string.
        let Some(value) = value.filter(|value| !is_blank_text(value)) else {
            continue;
        };
        if let Some(message) = failure(check, &attribute, value) {
            messages.push((name.to_owned(), message));
        }
    }
}

fn failure(check: &Check, attribute: &str, value: &Value) -> Option<String> {
    match (check, value) {
        (Check::String, Value::String(_)) => None,
        (Check::String, _) => Some(format!("The {attribute} field must be a string.")),
        (Check::Max(limit), Value::String(text)) if text.chars().count() as u64 > *limit => Some(
            format!("The {attribute} field must not be greater than {limit} characters."),
        ),
        (Check::Max(limit), Value::Number(number))
            if number.as_f64().is_some_and(|number| number > *limit as f64) =>
        {
            Some(format!(
                "The {attribute} field must not be greater than {limit}."
            ))
        }
        (Check::Max(limit), Value::Array(items)) if items.len() as u64 > *limit => Some(format!(
            "The {attribute} field must not have more than {limit} items."
        )),
        _ => None,
    }
}

fn is_blank(value: Option<&Value>) -> bool {
    match value {
        None | Some(Value::Null) => true,
        Some(Value::Array(items)) => items.is_empty(),
        Some(Value::Object(fields)) => fields.is_empty(),
        Some(value) => is_blank_text(value),
    }
}

fn is_blank_text(value: &Value) -> bool {
    matches!(value, Value::String(text) if text.trim().is_empty())
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
        match login_rules().validate(&input) {
            Ok(()) => Vec::new(),
            Err(error) => error.errors().to_vec(),
        }
    }

    fn field(name: &str, message: &str) -> (String, Vec<String>) {
        (name.to_owned(), vec![message.to_owned()])
    }

    #[test]
    fn valid_input_passes_and_an_optional_field_may_be_left_out() {
        assert!(errors(json!({"username": "sang"})).is_empty());
        assert!(errors(json!({"username": "sang", "device_uuid": "abc"})).is_empty());
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
            errors(json!({"username": "sang", "device_uuid": 7})),
            vec![field(
                "device_uuid",
                "The device uuid field must be a string."
            )]
        );
        assert_eq!(
            errors(json!({"username": "sangsakarin"})),
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

        let thai = rules.validate(&json!({"name": "ไทย", "age": 3, "tags": [1, 2, 3]}));
        let over = rules.validate(&json!({"age": 4, "tags": [1, 2, 3, 4]}));

        assert!(thai.is_ok());
        assert_eq!(
            over.err().unwrap().errors(),
            [
                field("age", "The age field must not be greater than 3."),
                field("tags", "The tags field must not have more than 3 items."),
            ]
        );
    }
}
