use std::future::Future;
use std::hash::{BuildHasher, RandomState};
use std::sync::OnceLock;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use http::header::{HeaderName, HeaderValue};
use sabai_core::log::{self, Instrument};
use sabai_core::{Request, Response};

use crate::{Middleware, Next};

// A `&str` for reading: `HeaderName::as_str` would compile a match over every standard header.
const REQUEST_ID: &str = "x-request-id";

/// Gives every request an ID and one log line, like `php artisan serve` prints:
/// `request{id=3f9a1c0b7e2d4a65}: GET /posts 200 4.2ms`. Logs written while handling the
/// request carry the same ID, and the response sends it back as `X-Request-Id`.
pub struct TraceRequests;

impl Middleware for TraceRequests {
    // Boxed here so the body compiles once in Sabai, not into every app.
    fn handle(&self, request: Request, next: Next) -> impl Future<Output = Response> + Send {
        Box::pin(trace(request, next))
    }
}

async fn trace(request: Request, next: Next) -> Response {
    let id = match request.header(REQUEST_ID) {
        Some(id) if is_safe_to_log(id) => id.to_owned(),
        _ => new_id(),
    };
    let span = log::info_span!("request", %id);
    let line = format!("{} {}", request.method(), request.path());
    let started = Instant::now();
    let mut response = next.run(request).instrument(span.clone()).await;
    let status = response.status().as_u16();
    span.in_scope(|| log::info!("{line} {status} {:.1?}", started.elapsed()));
    if let Ok(id) = HeaderValue::from_str(&id) {
        let name = HeaderName::from_static(REQUEST_ID);
        response.headers_mut().insert(name, id);
    }
    response
}

// An ID from a proxy is kept so logs line up across services, but only a short plain one:
// anything else could forge log lines.
fn is_safe_to_log(id: &str) -> bool {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '-');
    !id.is_empty() && id.len() <= 64 && id.chars().all(allowed)
}

// A counter hashed with a key chosen once per process: unique here, unguessable elsewhere,
// with no extra dependency. IDs only tell requests apart; they are not secrets.
fn new_id() -> String {
    static COUNT: AtomicU64 = AtomicU64::new(0);
    static KEY: OnceLock<RandomState> = OnceLock::new();
    let count = COUNT.fetch_add(1, Ordering::Relaxed);
    format!("{:016x}", KEY.get_or_init(RandomState::new).hash_one(count))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_short_plain_ids_are_kept() {
        assert!(is_safe_to_log("01J9XKZ-req.7_a"));
        assert!(!is_safe_to_log(""));
        assert!(!is_safe_to_log(&"a".repeat(65)));
        assert!(!is_safe_to_log("abc\nINFO forged line"));
        assert!(!is_safe_to_log("a b"));
    }

    #[test]
    fn new_ids_are_sixteen_hex_digits_and_differ() {
        let (first, second) = (new_id(), new_id());

        assert_eq!(first.len(), 16);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()), "{first}");
        assert_ne!(first, second);
    }
}
