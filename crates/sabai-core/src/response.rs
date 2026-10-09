use http::StatusCode;
use http::header::{CONTENT_TYPE, HeaderValue};

use crate::Body;

/// An HTTP response with a Sabai [`Body`].
pub type Response = http::Response<Body>;

/// Anything a handler can return: strings, status codes, `(StatusCode, T)`, `Result<T, E>` and `Response` itself.
pub trait IntoResponse {
    /// Turns `self` into a [`Response`].
    fn into_response(self) -> Response;
}

impl IntoResponse for Response {
    fn into_response(self) -> Response {
        self
    }
}

impl IntoResponse for Body {
    fn into_response(self) -> Response {
        Response::new(self)
    }
}

impl IntoResponse for () {
    fn into_response(self) -> Response {
        Body::empty().into_response()
    }
}

impl IntoResponse for StatusCode {
    fn into_response(self) -> Response {
        with_status(().into_response(), self)
    }
}

impl IntoResponse for &'static str {
    fn into_response(self) -> Response {
        plain_text(Body::from(self))
    }
}

impl IntoResponse for String {
    fn into_response(self) -> Response {
        plain_text(Body::from(self))
    }
}

impl<T: IntoResponse> IntoResponse for (StatusCode, T) {
    fn into_response(self) -> Response {
        with_status(self.1.into_response(), self.0)
    }
}

impl<T: IntoResponse, E: IntoResponse> IntoResponse for Result<T, E> {
    fn into_response(self) -> Response {
        match self {
            Ok(value) => value.into_response(),
            Err(error) => error.into_response(),
        }
    }
}

fn plain_text(body: Body) -> Response {
    let mut response = Response::new(body);
    let text = HeaderValue::from_static("text/plain; charset=utf-8");
    response.headers_mut().insert(CONTENT_TYPE, text);
    response
}

fn with_status(mut response: Response, status: StatusCode) -> Response {
    *response.status_mut() = status;
    response
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strings_respond_200_with_plain_text() {
        let response = "Hello, Sabai".into_response();

        assert_eq!(response.status(), StatusCode::OK);
        assert_eq!(
            response.headers()[CONTENT_TYPE],
            "text/plain; charset=utf-8"
        );
        assert_eq!(response.body().as_bytes(), b"Hello, Sabai");
    }

    #[test]
    fn a_status_code_alone_responds_with_an_empty_body() {
        let response = StatusCode::NO_CONTENT.into_response();

        assert_eq!(response.status(), StatusCode::NO_CONTENT);
        assert!(response.body().as_bytes().is_empty());
    }

    #[test]
    fn a_status_tuple_overrides_the_status_and_keeps_the_body() {
        let response = (StatusCode::CREATED, String::from("saved")).into_response();

        assert_eq!(response.status(), StatusCode::CREATED);
        assert_eq!(response.body().as_bytes(), b"saved");
    }

    #[test]
    fn a_result_responds_with_whichever_side_it_holds() {
        let failed: Result<&str, StatusCode> = Err(StatusCode::NOT_FOUND);

        assert_eq!(failed.into_response().status(), StatusCode::NOT_FOUND);
    }
}
