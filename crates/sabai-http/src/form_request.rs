use std::future::ready;

use http::header::CONTENT_TYPE;
use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, Method, Request, Result, StatusCode};
use serde::de::DeserializeOwned;
use serde_json::error::Category;

use crate::FromRequest;

/// The input of one action as a struct, like Laravel's `StorePostRequest`: a handler that takes
/// `request: StorePostRequest` receives it filled from the query string (`GET`) or the JSON or form body.
///
/// ```ignore
/// #[derive(Deserialize)]
/// pub struct StorePostRequest {
///     pub title: String,
/// }
///
/// impl FormRequest for StorePostRequest {}
/// ```
pub trait FormRequest: DeserializeOwned + Send + 'static {}

impl<T: FormRequest> FromRequest for T {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(parse_input(request)))
    }
}

fn parse_input<T: DeserializeOwned>(request: &Request) -> Result<T> {
    match input_of(request)? {
        Input::Json(bytes) => serde_json::from_slice(bytes).map_err(json_error),
        Input::Form(bytes) => serde_urlencoded::from_bytes(bytes).map_err(invalid_input),
    }
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

    #[derive(Debug, Deserialize, PartialEq)]
    struct StorePostRequest {
        title: String,
        draft: bool,
    }

    impl FormRequest for StorePostRequest {}

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
}
