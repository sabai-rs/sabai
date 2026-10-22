use sabai_core::contracts::BoxFuture;
use sabai_core::{Request, Response};
use serde_json::Value;

use crate::Next;

/// Changes one string in the input; it gets the name of the field that holds the string.
pub(super) type Transform = fn(&str, &mut Value);

/// Runs `transform` on every string in the input, nested ones included, then goes on,
/// like Laravel's `TransformsRequest`. Input that cannot be read is left for the handler to reject.
// A `fn` pointer and a future boxed here, so the body compiles once, not per middleware or app.
pub(super) fn transform_input(
    request: Request,
    next: Next,
    transform: Transform,
) -> BoxFuture<'static, Response> {
    Box::pin(transform_then_run(request, next, transform))
}

async fn transform_then_run(request: Request, next: Next, transform: Transform) -> Response {
    let mut fields = match request.all() {
        Ok(fields) if !fields.is_empty() => fields.clone(),
        _ => return next.run(request).await,
    };
    for (name, value) in &mut fields {
        transform_value(name, value, transform);
    }
    next.run(request.with_input(fields)).await
}

// Nested strings get the name of their nearest field, so `user.password` is still a `password`.
fn transform_value(name: &str, value: &mut Value, transform: Transform) {
    match value {
        Value::String(_) => transform(name, value),
        Value::Array(items) => {
            for item in items {
                transform_value(name, item, transform);
            }
        }
        Value::Object(fields) => {
            for (name, value) in fields {
                transform_value(name, value, transform);
            }
        }
        _ => {}
    }
}
