use sabai::{Error, Result, StatusCode};

fn find_post(id: u32) -> Result<&'static str> {
    if id != 1 {
        return Err(Error::not_found());
    }
    Ok("Hello, Sabai")
}

#[test]
fn handlers_return_sabai_errors_through_the_facade() {
    assert_eq!(find_post(1).unwrap(), "Hello, Sabai");
    assert_eq!(find_post(2).unwrap_err().status(), StatusCode::NOT_FOUND);
}

#[test]
fn a_handler_returning_sabai_result_responds_with_json_on_error() {
    use sabai::IntoResponse;

    let response = find_post(2).into_response();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.body().as_bytes(), br#"{"message":"Not Found"}"#);
}
