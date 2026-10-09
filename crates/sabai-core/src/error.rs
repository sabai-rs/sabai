use std::borrow::Cow;
use std::error::Error as StdError;
use std::fmt;

use http::StatusCode;

type BoxedSource = Box<dyn StdError + Send + Sync + 'static>;

/// The error every Sabai handler, extractor and service returns; it knows its HTTP status.
#[derive(Debug)]
pub struct Error {
    status: StatusCode,
    message: Cow<'static, str>,
    source: Option<BoxedSource>,
}

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
        }
    }

    /// The HTTP status this error responds with.
    pub fn status(&self) -> StatusCode {
        self.status
    }

    /// The client-facing message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// The underlying cause, if any.
    pub fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.source.as_deref().map(|source| source as _)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)?;
        match &self.source {
            Some(source) => write!(f, ": {source}"),
            None => Ok(()),
        }
    }
}

// `Error` deliberately does not implement `std::error::Error`: that impl would
// overlap with this blanket `From` (coherence), and the blanket `From` is what
// lets `?` turn any error into a 500, like an uncaught exception in Laravel.
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
        assert_eq!(
            error.to_string(),
            "Server Error: invalid digit found in string"
        );
    }
}
