use std::future::Future;

use sabai_core::{Request, Response};
use serde_json::Value;

use super::transforms_request::transform_input;
use crate::{Middleware, Next};

// Spaces around a password are part of it.
const NOT_TRIMMED: [&str; 3] = ["current_password", "password", "password_confirmation"];

/// Trims spaces around every input string except passwords, like Laravel's `TrimStrings`.
pub struct TrimStrings;

impl Middleware for TrimStrings {
    fn handle(&self, request: Request, next: Next) -> impl Future<Output = Response> + Send {
        transform_input(request, next, trim)
    }
}

fn trim(name: &str, value: &mut Value) {
    if NOT_TRIMMED.contains(&name) {
        return;
    }
    if let Value::String(text) = value {
        let trimmed = text.trim();
        if trimmed.len() != text.len() {
            *text = trimmed.to_owned();
        }
    }
}
