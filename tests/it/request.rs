use sabai::{Body, Error, IntoResponse, Method, Request, Result, StatusCode};

fn show_token(request: &Request) -> Result<String> {
    let token = request
        .header("authorization")
        .ok_or_else(Error::unauthorized)?;
    Ok(format!(
        "{} {} with {token}",
        request.method(),
        request.path()
    ))
}

fn request(authorization: Option<&str>) -> Request {
    let mut builder = http::Request::builder().method(Method::GET).uri("/me");
    if let Some(value) = authorization {
        builder = builder.header("Authorization", value);
    }
    Request::from(builder.body(Body::empty()).unwrap())
}

#[test]
fn handlers_read_the_request_through_the_facade() {
    let response = show_token(&request(Some("Bearer abc"))).into_response();

    assert_eq!(response.body().as_bytes(), b"GET /me with Bearer abc");
}

#[test]
fn a_missing_header_becomes_a_401() {
    let response = show_token(&request(None)).into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
