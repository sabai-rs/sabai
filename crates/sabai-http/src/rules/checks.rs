use serde_json::{Map, Value};

use super::Rule;
use super::email::is_email;
use super::fields::matching_fields;

pub(super) enum Check {
    Required,
    Sometimes,
    String,
    Array,
    Email,
    Lowercase,
    Confirmed,
    Min(u64),
    Max(u64),
    MaxBytes(u64),
}

pub(super) fn check_field(
    name: &str,
    value: Option<&Value>,
    rules: &[Rule],
    input: &Map<String, Value>,
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
                return;
            }
            continue;
        }
        let Some(value) = value.filter(|value| !is_blank_text(value)) else {
            continue;
        };
        let message = match check {
            Check::Confirmed => unconfirmed(name, value, input)
                .then(|| format!("The {attribute} field confirmation does not match.")),
            check => failure(check, &attribute, value),
        };
        if let Some(message) = message {
            messages.push((name.to_owned(), message));
        }
    }
}

fn failure(check: &Check, attribute: &str, value: &Value) -> Option<String> {
    match (check, value) {
        (Check::String, Value::String(_)) => None,
        (Check::String, _) => Some(format!("The {attribute} field must be a string.")),
        (Check::Array, Value::Array(_)) => None,
        (Check::Array, _) => Some(format!("The {attribute} field must be an array.")),
        (Check::Email, Value::String(text)) if is_email(text) => None,
        (Check::Email, _) => Some(format!(
            "The {attribute} field must be a valid email address."
        )),
        (Check::Lowercase, Value::String(text)) if *text != text.to_lowercase() => {
            Some(format!("The {attribute} field must be lowercase."))
        }
        (Check::Min(limit), Value::String(text)) if (text.chars().count() as u64) < *limit => Some(
            format!("The {attribute} field must be at least {limit} characters."),
        ),
        (Check::Min(limit), Value::Number(number))
            if number.as_f64().is_some_and(|number| number < *limit as f64) =>
        {
            Some(format!("The {attribute} field must be at least {limit}."))
        }
        (Check::Min(limit), Value::Array(items)) if (items.len() as u64) < *limit => Some(format!(
            "The {attribute} field must have at least {limit} items."
        )),
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
        (Check::MaxBytes(limit), Value::String(text)) if text.len() as u64 > *limit => Some(
            format!("The {attribute} field must not be greater than {limit} bytes."),
        ),
        (Check::Max(limit), Value::Array(items)) if items.len() as u64 > *limit => Some(format!(
            "The {attribute} field must not have more than {limit} items."
        )),
        _ => None,
    }
}

// `password` is confirmed by `password_confirmation`, and `users.0.password` by
// `users.0.password_confirmation`.
fn unconfirmed(name: &str, value: &Value, input: &Map<String, Value>) -> bool {
    let confirmation = matching_fields(&format!("{name}_confirmation"), input);
    confirmation.first().and_then(|(_, value)| *value) != Some(value)
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
