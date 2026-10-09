use sabai::{IntoResponse, Response, StatusCode};

fn index() -> impl IntoResponse {
    "All posts"
}

fn store() -> impl IntoResponse {
    (StatusCode::CREATED, "Post saved")
}

#[test]
fn handlers_return_anything_that_turns_into_a_response() {
    let responses: [Response; 2] = [index().into_response(), store().into_response()];

    assert_eq!(responses[0].status(), StatusCode::OK);
    assert_eq!(responses[1].status(), StatusCode::CREATED);
    assert_eq!(responses[1].body().as_bytes(), b"Post saved");
}
