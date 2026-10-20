use http::header::CONTENT_TYPE;
use http::{Method, StatusCode};
use serde_json::{Map, Value};

use crate::{Error, Request};

pub(super) type Fields = Map<String, Value>;

/// Why the input could not be read; kept with the request so every reader gets the same error.
#[derive(Clone, Debug)]
pub(super) struct InputError {
    status: StatusCode,
    message: String,
}

impl From<&InputError> for Error {
    fn from(error: &InputError) -> Self {
        Error::new(error.status, error.message.clone())
    }
}

/// The query string, merged with the body unless the request is a `GET` or `HEAD`;
/// the body wins when both have a field, like Laravel.
pub(super) fn read(request: &Request) -> Result<Fields, InputError> {
    let query = request.query_string().unwrap_or_default();
    let mut fields = form_fields(query.as_bytes())?;
    if request.method() != Method::GET && request.method() != Method::HEAD {
        fields.extend(body_fields(request)?);
    }
    Ok(fields)
}

/// The media type without parameters: `application/json; charset=utf-8` gives `application/json`.
pub(super) fn content_type(request: &Request) -> &str {
    let content_type = request.header(CONTENT_TYPE.as_str()).unwrap_or_default();
    content_type.split(';').next().unwrap_or_default().trim()
}

// A cross-site form can post `text/plain` without a preflight, so only a JSON type is parsed as JSON.
fn body_fields(request: &Request) -> Result<Fields, InputError> {
    let body = request.body().as_bytes();
    if request.is_json() {
        return json_fields(body);
    }
    match content_type(request) {
        "application/x-www-form-urlencoded" => form_fields(body),
        "" if body.is_empty() => Ok(Fields::new()),
        _ => Err(InputError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: "Send the body as JSON (Content-Type: application/json) \
                      or as a form (Content-Type: application/x-www-form-urlencoded)."
                .to_owned(),
        }),
    }
}

fn json_fields(body: &[u8]) -> Result<Fields, InputError> {
    match serde_json::from_slice(body) {
        Ok(Value::Object(fields)) => Ok(fields),
        Ok(_) => Err(InputError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: r#"The JSON body must be an object, like {"title": "Sabai"}."#.to_owned(),
        }),
        Err(error) => Err(InputError {
            status: StatusCode::BAD_REQUEST,
            message: format!("The JSON body is malformed: {error}."),
        }),
    }
}

fn form_fields(bytes: &[u8]) -> Result<Fields, InputError> {
    let pairs: Vec<(String, String)> =
        serde_urlencoded::from_bytes(bytes).map_err(|error| InputError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            message: format!("The input is invalid: {error}."),
        })?;
    let fields = pairs
        .into_iter()
        .map(|(name, value)| (name, Value::String(value)));
    Ok(fields.collect())
}
