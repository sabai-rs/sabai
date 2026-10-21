use std::any::Any;
use std::error::Error as StdError;
use std::fmt;
use std::future::Future;
use std::panic::{self, AssertUnwindSafe};
use std::pin::Pin;
use std::task::{Context, Poll};

use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, IntoResponse, Response};

/// Runs `future`, turning a panic into a logged `500`, like Laravel's handler for an uncaught
/// exception; without it the client gets an empty reply.
pub(crate) fn catch_panic(future: BoxFuture<'static, Response>) -> BoxFuture<'static, Response> {
    Box::pin(CatchPanic(future))
}

struct CatchPanic(BoxFuture<'static, Response>);

impl Future for CatchPanic {
    type Output = Response;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Response> {
        // Unwind safe in practice: after a panic the future is dropped, never polled again.
        match panic::catch_unwind(AssertUnwindSafe(|| self.0.as_mut().poll(cx))) {
            Ok(poll) => poll,
            Err(payload) => Poll::Ready(Error::internal(Panicked::from(payload)).into_response()),
        }
    }
}

#[derive(Debug)]
struct Panicked(String);

impl From<Box<dyn Any + Send>> for Panicked {
    fn from(payload: Box<dyn Any + Send>) -> Self {
        let message = match payload.downcast::<String>() {
            Ok(message) => *message,
            Err(payload) => match payload.downcast::<&str>() {
                Ok(message) => (*message).to_owned(),
                Err(_) => "a panic without a message".to_owned(),
            },
        };
        Self(message)
    }
}

impl fmt::Display for Panicked {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "the request panicked: {}", self.0)
    }
}

impl StdError for Panicked {}
