use sabai_core::Response;
use serde_json::Value;

/// A response with Laravel's assertions; each returns `&Self`, so they chain.
pub struct TestResponse {
    response: Response,
}

impl TestResponse {
    pub(crate) fn new(response: Response) -> Self {
        Self { response }
    }

    /// Fails unless the status is `expected`, showing the body so the reason is visible.
    pub fn assert_status(&self, expected: u16) -> &Self {
        let actual = self.response.status().as_u16();
        assert!(
            actual == expected,
            "Expected response status code [{expected}] but received {actual}.\n{}",
            self.text()
        );
        self
    }

    /// Fails unless the status is `200`.
    pub fn assert_ok(&self) -> &Self {
        self.assert_status(200)
    }

    /// Fails unless the status is `201`.
    pub fn assert_created(&self) -> &Self {
        self.assert_status(201)
    }

    /// Fails unless the status is `404`.
    pub fn assert_not_found(&self) -> &Self {
        self.assert_status(404)
    }

    /// Fails unless one object in the JSON has every key and value of `fragment`, at any depth.
    pub fn assert_json_fragment(&self, fragment: Value) -> &Self {
        let json = self.json();
        assert!(
            contains_fragment(&json, &fragment),
            "Unable to find JSON fragment\n{fragment}\nwithin\n{json}"
        );
        self
    }

    /// Fails unless the header `name` is `expected`, like Laravel's `assertHeader`.
    pub fn assert_header(&self, name: &str, expected: &str) -> &Self {
        let actual = self.response.headers().get(name);
        assert!(
            actual.is_some_and(|value| value == expected),
            "Expected header [{name}] to be [{expected}] but found {actual:?}."
        );
        self
    }

    /// Fails if the response has the header `name`, like Laravel's `assertHeaderMissing`.
    pub fn assert_header_missing(&self, name: &str) -> &Self {
        let actual = self.response.headers().get(name);
        assert!(
            actual.is_none(),
            "Unexpected header [{name}] is present on response: {actual:?}."
        );
        self
    }

    /// The body parsed as JSON.
    pub fn json(&self) -> Value {
        serde_json::from_slice(self.response.body().as_bytes())
            .unwrap_or_else(|error| panic!("The response is not JSON ({error}):\n{}", self.text()))
    }

    /// The body as text, for messages and for asserting on it directly.
    pub fn text(&self) -> String {
        String::from_utf8_lossy(self.response.body().as_bytes()).into_owned()
    }
}

fn contains_fragment(json: &Value, fragment: &Value) -> bool {
    let Value::Object(wanted) = fragment else {
        return json == fragment;
    };
    match json {
        Value::Object(object) => {
            wanted
                .iter()
                .all(|(key, value)| object.get(key) == Some(value))
                || object
                    .values()
                    .any(|child| contains_fragment(child, fragment))
        }
        Value::Array(items) => items.iter().any(|item| contains_fragment(item, fragment)),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn a_fragment_is_found_in_one_object_at_any_depth() {
        let json = json!({"data": [{"id": 1, "title": "a"}, {"id": 2, "title": "b"}]});

        assert!(contains_fragment(&json, &json!({"id": 2, "title": "b"})));
        assert!(!contains_fragment(&json, &json!({"id": 1, "title": "b"})));
    }
}
