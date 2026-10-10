use std::future::ready;

use http::header::CONTENT_TYPE;
use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, Method, Request, Result, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::error::Category;
use serde_json::{Map, Value};

use crate::{FromRequest, Rules};

/// The input of one action as a struct, like Laravel's `StorePostRequest`: a handler that takes
/// `request: StorePostRequest` receives it filled from the query string (`GET`) or the JSON or
/// form body, after its [`rules`](FormRequest::rules) pass.
///
/// ```ignore
/// #[derive(Deserialize)]
/// pub struct StorePostRequest {
///     pub title: String,
/// }
///
/// impl FormRequest for StorePostRequest {
///     fn rules() -> Rules {
///         Rules::new().field("title", [required(), string(), max(255)])
///     }
/// }
/// ```
pub trait FormRequest: DeserializeOwned + Send + 'static {
    /// The rules the raw input must pass before it becomes `Self`; a failure is a `422` per field.
    fn rules() -> Rules {
        Rules::new()
    }
}

impl<T: FormRequest> FromRequest for T {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(parse_input(request, &T::rules())))
    }
}

fn parse_input<T: DeserializeOwned>(request: &Request, rules: &Rules) -> Result<T> {
    match validated_input(request, rules)? {
        Validated::Json(value) => T::deserialize(value).map_err(invalid_input),
        // serde_urlencoded reads `draft=true` into a `bool`; a `Value` of strings would not.
        Validated::Form(bytes) => serde_urlencoded::from_bytes(bytes).map_err(invalid_input),
    }
}

enum Validated<'a> {
    Json(Value),
    Form(&'a [u8]),
}

// Not generic: parsing and validating compile once, however many request structs an app has.
fn validated_input<'a>(request: &'a Request, rules: &Rules) -> Result<Validated<'a>> {
    match input_of(request)? {
        Input::Json(bytes) => {
            let value = serde_json::from_slice(bytes).map_err(json_error)?;
            rules.validate(&value)?;
            Ok(Validated::Json(value))
        }
        Input::Form(bytes) => {
            rules.validate(&form_value(bytes)?)?;
            Ok(Validated::Form(bytes))
        }
    }
}

fn form_value(bytes: &[u8]) -> Result<Value> {
    let pairs: Vec<(String, String)> =
        serde_urlencoded::from_bytes(bytes).map_err(invalid_input)?;
    let fields: Map<String, Value> = pairs
        .into_iter()
        .map(|(name, value)| (name, Value::String(value)))
        .collect();
    Ok(Value::Object(fields))
}

enum Input<'a> {
    Json(&'a [u8]),
    Form(&'a [u8]),
}

// Like Laravel's `isJson()`, the Content-Type picks the parser, so a JSON body sent as
// `text/plain` (which a cross-site form can do without a preflight) is never read.
fn input_of(request: &Request) -> Result<Input<'_>> {
    if request.method() == Method::GET || request.method() == Method::HEAD {
        let query = request.query_string().unwrap_or_default();
        return Ok(Input::Form(query.as_bytes()));
    }
    let body = request.body().as_bytes();
    let content_type = request.header(CONTENT_TYPE.as_str()).unwrap_or_default();
    let essence = content_type.split(';').next().unwrap_or_default().trim();
    match essence {
        _ if essence.ends_with("/json") || essence.ends_with("+json") => Ok(Input::Json(body)),
        "application/x-www-form-urlencoded" => Ok(Input::Form(body)),
        "" if body.is_empty() => Ok(Input::Form(body)),
        _ => Err(Error::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "Send the body as JSON (Content-Type: application/json) \
             or as a form (Content-Type: application/x-www-form-urlencoded).",
        )),
    }
}

fn json_error(error: serde_json::Error) -> Error {
    match error.classify() {
        Category::Syntax | Category::Eof => Error::new(
            StatusCode::BAD_REQUEST,
            format!("The JSON body is malformed: {error}."),
        ),
        Category::Data | Category::Io => invalid_input(error),
    }
}

fn invalid_input(error: impl std::fmt::Display) -> Error {
    Error::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        format!("The input is invalid: {error}."),
    )
}

#[cfg(test)]
mod tests {
    use pollster::block_on;
    use sabai_core::Body;
    use serde::Deserialize;

    use super::*;
    use crate::rules::{max, required, string};

    #[derive(Debug, Deserialize, PartialEq)]
    struct StorePostRequest {
        title: String,
        draft: bool,
    }

    impl FormRequest for StorePostRequest {
        fn rules() -> Rules {
            Rules::new().field("title", [required(), string(), max(10)])
        }
    }

    fn send(
        method: Method,
        uri: &str,
        content_type: &str,
        body: &'static str,
    ) -> Result<StorePostRequest> {
        let request = http::Request::builder()
            .method(method)
            .uri(uri)
            .header(CONTENT_TYPE, content_type)
            .body(Body::from(body))
            .unwrap();
        block_on(StorePostRequest::from_request(&Request::from(request)))
    }

    fn expected() -> StorePostRequest {
        StorePostRequest {
            title: "Sabai".to_owned(),
            draft: true,
        }
    }

    #[test]
    fn a_json_body_fills_the_request() {
        let body = r#"{"title":"Sabai","draft":true}"#;

        let request = send(
            Method::POST,
            "/posts",
            "application/json; charset=utf-8",
            body,
        );

        assert_eq!(request.unwrap(), expected());
    }

    #[test]
    fn a_form_body_and_a_get_query_string_fill_the_request() {
        let form = "application/x-www-form-urlencoded";

        let posted = send(Method::POST, "/posts", form, "title=Sabai&draft=true");
        let queried = send(Method::GET, "/posts?title=Sabai&draft=true", "", "");

        assert_eq!(posted.unwrap(), expected());
        assert_eq!(queried.unwrap(), expected());
    }

    #[test]
    fn broken_json_is_a_400_and_wrong_input_is_a_422() {
        let status = |body| {
            send(Method::POST, "/posts", "application/json", body)
                .err()
                .unwrap()
                .status()
        };

        assert_eq!(status(r#"{"title":"#), StatusCode::BAD_REQUEST);
        assert_eq!(
            status(r#"{"title":"Sabai"}"#),
            StatusCode::UNPROCESSABLE_ENTITY
        );
        assert_eq!(
            status(r#"{"title":1,"draft":true}"#),
            StatusCode::UNPROCESSABLE_ENTITY
        );
    }

    #[test]
    fn json_sent_as_plain_text_is_never_parsed() {
        let body = r#"{"title":"Sabai","draft":true}"#;

        let error = send(Method::POST, "/posts", "text/plain", body)
            .err()
            .unwrap();

        assert_eq!(error.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert!(
            error.to_string().contains("Content-Type: application/json"),
            "{error}"
        );
    }

    #[test]
    fn rules_run_on_json_and_form_input_before_the_struct_is_built() {
        let form = "application/x-www-form-urlencoded";

        let missing = send(
            Method::POST,
            "/posts",
            "application/json",
            r#"{"draft":true}"#,
        );
        let too_long = send(
            Method::POST,
            "/posts",
            form,
            "title=Sabai+framework&draft=true",
        );

        let missing = missing.err().unwrap();
        assert_eq!(missing.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(missing.message(), "The title field is required.");
        assert_eq!(
            too_long.err().unwrap().message(),
            "The title field must not be greater than 10 characters."
        );
    }
}
