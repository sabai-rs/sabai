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

#[test]
fn handlers_read_the_request_through_the_facade() {
    let request = http::Request::builder()
        .uri("/me")
        .header("Authorization", "Bearer abc")
        .body(Body::empty())
        .unwrap();

    let response = show_token(&Request::from(request)).into_response();

    assert_eq!(response.body().as_bytes(), b"GET /me with Bearer abc");
}

#[test]
fn a_missing_header_becomes_a_401() {
    let request = Request::create(Method::GET, "/me").unwrap();

    let response = show_token(&request).into_response();

    assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
}
