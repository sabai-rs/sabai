use std::future::Future;

use sabai_core::{Request, Response};
use serde_json::Value;

use super::transforms_request::transform_input;
use crate::{Middleware, Next};

/// Turns every empty input string into `null`, like Laravel's `ConvertEmptyStringsToNull`,
/// so a blank optional field arrives as `None`.
pub struct ConvertEmptyStringsToNull;

impl Middleware for ConvertEmptyStringsToNull {
    fn handle(&self, request: Request, next: Next) -> impl Future<Output = Response> + Send {
        transform_input(request, next, empty_to_null)
    }
}

fn empty_to_null(_name: &str, value: &mut Value) {
    if value.as_str() == Some("") {
        *value = Value::Null;
    }
}
