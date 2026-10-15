use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;

use http::StatusCode;
use serde::ser::{Serialize, SerializeMap, Serializer};

use crate::response::{json, with_status};
use crate::{IntoResponse, Response};

type BoxedSource = Box<dyn StdError + Send + Sync + 'static>;

/// The error every Sabai handler, handler argument and service returns; it knows its HTTP status.
pub struct Error {
    status: StatusCode,
    message: Cow<'static, str>,
    source: Option<BoxedSource>,
    context: Vec<Cow<'static, str>>,
    errors: FieldErrors,
}

/// Messages per field, in the order they were added, like Laravel's error bag.
#[derive(Debug, Default)]
struct FieldErrors(Vec<(String, Vec<String>)>);

/// `Result` with [`Error`] as the default error type.
pub type Result<T, E = Error> = std::result::Result<T, E>;

impl Error {
    /// An error with a status and a message that is safe to show to the client.
    pub fn new(status: StatusCode, message: impl Into<Cow<'static, str>>) -> Self {
        Self::from_parts(status, message.into(), None)
    }

    /// An error with a status and its standard reason phrase, like Laravel's `abort(404)`.
    pub fn from_status(status: StatusCode) -> Self {
        let reason = status.canonical_reason().unwrap_or("Error");
        Self::from_parts(status, Cow::Borrowed(reason), None)
    }

    /// `404 Not Found`.
    pub fn not_found() -> Self {
        Self::from_status(StatusCode::NOT_FOUND)
    }

    /// `401 Unauthorized`: the request has no valid credentials.
    pub fn unauthorized() -> Self {
        Self::from_status(StatusCode::UNAUTHORIZED)
    }

    /// `403 Forbidden`: the user is known but not allowed.
    pub fn forbidden() -> Self {
        Self::from_status(StatusCode::FORBIDDEN)
    }

    /// `422` with messages per field, like Laravel's `ValidationException::withMessages([...])`:
    /// `Error::with_messages(&[("email", "These credentials do not match our records.")])`.
    /// A field may appear more than once to give it several messages.
    pub fn with_messages(messages: &[(&str, &str)]) -> Self {
        let mut errors = FieldErrors::default();
        for (field, message) in messages {
            errors.push(field, message);
        }
        let message = errors.summary();
        Self {
            errors,
            ..Self::from_parts(StatusCode::UNPROCESSABLE_ENTITY, message.into(), None)
        }
    }

    /// `500 Server Error` wrapping the underlying cause, which is logged but never shown to the client.
    pub fn internal(source: impl StdError + Send + Sync + 'static) -> Self {
        Self::internal_boxed(Box::new(source))
    }

    fn internal_boxed(source: BoxedSource) -> Self {
        Self::from_parts(
            StatusCode::INTERNAL_SERVER_ERROR,
            Cow::Borrowed("Server Error"),
            Some(source),
        )
    }

    fn from_parts(
        status: StatusCode,
        message: Cow<'static, str>,
        source: Option<BoxedSource>,
    ) -> Self {
        Self {
            status,
            message,
            source,
            context: Vec::new(),
            errors: FieldErrors::default(),
        }
    }

    /// Adds what was being done when this error happened; shown in logs, never to the client.
    pub fn context(self, context: impl Into<Cow<'static, str>>) -> Self {
        self.push_context(context.into())
    }

    fn push_context(mut self, context: Cow<'static, str>) -> Self {
        self.context.push(context);
        self
    }

    /// The HTTP status this error responds with.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The client-facing message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The messages per field of a validation error, in order; empty for other errors.
    pub fn errors(&self) -> &[(String, Vec<String>)] {
        &self.errors.0
    }

    /// The underlying cause, if any.
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_deref().map(|source| source as _)
    }
}

// Display is for logs, so it shows the cause; clients only see `message`.
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for context in self.context.iter().rev() {
            write!(f, "{context}: ")?;
        }
        match &self.source {
            Some(source) => write!(f, "{source}"),
            None => f.write_str(&self.message),
        }
    }
}

// `main` returning `Err` prints `Debug`, so it reads like `Display`; `{:#?}` still shows the fields.
impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !f.alternate() {
            return fmt::Display::fmt(self, f);
        }
        f.debug_struct("Error")
            .field("status", &self.status)
            .field("message", &self.message)
            .field("source", &self.source)
            .field("context", &self.context)
            .field("errors", &self.errors)
            .finish()
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        if self.status.is_server_error() {
            crate::log::error!("{self}");
        }

        let body =
            serde_json::to_string(&self).unwrap_or_else(|_| r#"{"message":"Server Error"}"#.into());
        with_status(json(body), self.status)
    }
}

impl FieldErrors {
    fn push(&mut self, field: &str, message: &str) {
        match self.0.iter_mut().find(|(name, _)| name == field) {
            Some((_, messages)) => messages.push(message.to_owned()),
            None => self.0.push((field.to_owned(), vec![message.to_owned()])),
        }
    }

    // Laravel's top-level message: "The title field is required. (and 2 more errors)".
    fn summary(&self) -> String {
        let mut messages = self.0.iter().flat_map(|(_, messages)| messages);
        let Some(first) = messages.next() else {
            return "The given data was invalid.".to_owned();
        };
        match messages.count() {
            0 => first.clone(),
            1 => format!("{first} (and 1 more error)"),
            more => format!("{first} (and {more} more errors)"),
        }
    }
}

// The client's view: `message`, plus `errors` for validation. By hand, because a
// `serde_json::Map` would sort the fields, and Laravel keeps them in the rules' order.
impl Serialize for Error {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut body = serializer.serialize_map(None)?;
        body.serialize_entry("message", &self.message)?;
        if !self.errors.0.is_empty() {
            body.serialize_entry("errors", &FieldsInOrder(&self.errors.0))?;
        }
        body.end()
    }
}

struct FieldsInOrder<'a>(&'a [(String, Vec<String>)]);

impl Serialize for FieldsInOrder<'_> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (field, messages) in self.0 {
            map.serialize_entry(field, messages)?;
        }
        map.end()
    }
}

// No `impl std::error::Error`: it would overlap this blanket `From`, which lets `?` turn any error into a 500.
impl<E: StdError + Send + Sync + 'static> From<E> for Error {
    fn from(source: E) -> Self {
        Self::internal(source)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn shortcuts_map_to_their_status_and_reason() {
        assert_eq!(Error::not_found().status(), StatusCode::NOT_FOUND);
        assert_eq!(Error::not_found().message(), "Not Found");
        assert_eq!(Error::unauthorized().status(), StatusCode::UNAUTHORIZED);
        assert_eq!(Error::forbidden().status(), StatusCode::FORBIDDEN);
    }

    #[test]
    fn new_keeps_a_custom_message() {
        let error = Error::new(StatusCode::CONFLICT, "Email already taken");

        assert_eq!(error.status(), StatusCode::CONFLICT);
        assert_eq!(error.message(), "Email already taken");
        assert_eq!(error.to_string(), "Email already taken");
        assert!(error.source().is_none());
    }

    #[test]
    fn question_mark_turns_any_error_into_a_500_that_keeps_its_cause() {
        fn parse_id() -> Result<i32> {
            Ok("abc".parse::<i32>()?)
        }

        let error = parse_id().unwrap_err();

        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert_eq!(error.message(), "Server Error");
        assert!(error.source().is_some());
        assert_eq!(error.to_string(), "invalid digit found in string");
    }

    #[test]
    fn with_messages_is_a_422_with_errors_per_field_in_order() {
        let error = Error::with_messages(&[
            ("title", "The title field is required."),
            ("body", "The body field must be a string."),
            ("title", "The title field must be a string."),
        ]);

        let response = error.into_response();

        assert_eq!(response.status(), StatusCode::UNPROCESSABLE_ENTITY);
        assert_eq!(
            response.body().as_bytes(),
            br#"{"message":"The title field is required. (and 2 more errors)","errors":{"title":["The title field is required.","The title field must be a string."],"body":["The body field must be a string."]}}"#
        );
    }

    #[test]
    fn one_message_is_also_the_summary() {
        let error = Error::with_messages(&[("credential", "These credentials do not match.")]);

        assert_eq!(error.message(), "These credentials do not match.");
        assert_eq!(error.errors()[0].0, "credential");
    }

    #[test]
    fn responds_with_its_status_and_a_json_message() {
        let response = Error::not_found().into_response();

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
        assert_eq!(response.headers()["content-type"], "application/json");
        assert_eq!(response.body().as_bytes(), br#"{"message":"Not Found"}"#);
    }

    #[test]
    fn the_response_never_shows_the_internal_cause() {
        let error = Error::from("abc".parse::<i32>().unwrap_err());

        let response = error.into_response();

        assert_eq!(response.body().as_bytes(), br#"{"message":"Server Error"}"#);
    }

    #[test]
    fn messages_are_escaped_as_json() {
        let error = Error::new(StatusCode::CONFLICT, r#"Title "Hello" is taken"#);

        let response = error.into_response();

        assert_eq!(
            response.body().as_bytes(),
            br#"{"message":"Title \"Hello\" is taken"}"#
        );
    }

    #[test]
    fn context_reads_outermost_first_in_logs() {
        let error = Error::from("abc".parse::<i32>().unwrap_err())
            .context("reading the port")
            .context("booting the app");

        assert_eq!(
            error.to_string(),
            "booting the app: reading the port: invalid digit found in string"
        );
    }

    #[test]
    fn debug_reads_like_display_so_main_prints_a_sentence() {
        let error = Error::from("abc".parse::<i32>().unwrap_err()).context("reading the port");

        assert_eq!(
            format!("{error:?}"),
            "reading the port: invalid digit found in string"
        );
        assert!(format!("{error:#?}").starts_with("Error {"));
    }

    #[test]
    fn context_never_reaches_the_client() {
        let response = Error::not_found()
            .context("looking up post 7")
            .into_response();

        assert_eq!(response.body().as_bytes(), br#"{"message":"Not Found"}"#);
    }
}
