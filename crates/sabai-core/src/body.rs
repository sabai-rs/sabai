use std::error::Error as StdError;
use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use http_body::{Frame, SizeHint};

type BoxError = Box<dyn StdError + Send + Sync + 'static>;

/// The body of every request and response: one concrete type, so handlers never become generic over it.
#[derive(Debug, Default)]
pub struct Body {
    bytes: Option<Bytes>,
}

impl Body {
    /// A body with no content.
    pub fn empty() -> Self {
        Self::default()
    }

    /// The whole body as bytes, before it has been sent.
    pub fn as_bytes(&self) -> &[u8] {
        self.bytes.as_deref().unwrap_or_default()
    }
}

impl From<Bytes> for Body {
    fn from(bytes: Bytes) -> Self {
        Self { bytes: Some(bytes) }
    }
}

impl From<String> for Body {
    fn from(text: String) -> Self {
        Self::from(Bytes::from(text))
    }
}

impl From<&'static str> for Body {
    fn from(text: &'static str) -> Self {
        Self::from(Bytes::from_static(text.as_bytes()))
    }
}

impl http_body::Body for Body {
    type Data = Bytes;
    // Boxed rather than `Infallible` so streaming bodies (M1-41, M1-48) can fail without an API change.
    type Error = BoxError;

    fn poll_frame(
        self: Pin<&mut Self>,
        _cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Bytes>, BoxError>>> {
        Poll::Ready(
            self.get_mut()
                .bytes
                .take()
                .map(|bytes| Ok(Frame::data(bytes))),
        )
    }

    fn is_end_stream(&self) -> bool {
        self.bytes.is_none()
    }

    fn size_hint(&self) -> SizeHint {
        SizeHint::with_exact(self.as_bytes().len() as u64)
    }
}
