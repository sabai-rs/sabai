use std::time::Duration;

use bytes::Bytes;

use super::BoxFuture;
use crate::Result;

/// A cache driver such as memory or Redis, like Laravel's `Cache\Store`. It stores bytes; the
/// app-facing `Cache` with `remember`, `pull` and typed values is built on top of these methods.
///
/// `add` and `increment` must be atomic in the driver (Redis `SET NX`, `INCR`): built from
/// `get` and `put`, two requests at once could lose a write.
pub trait CacheStore: Send + Sync {
    /// The value stored under `key`, or `None` when it is missing or expired.
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<Bytes>>>;

    /// Stores `value` under `key`; a `ttl` of `None` keeps it until it is forgotten.
    fn put<'a>(
        &'a self,
        key: &'a str,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> BoxFuture<'a, Result<()>>;

    /// Stores `value` only if `key` is missing or expired, atomically; `true` when it was stored.
    fn add<'a>(
        &'a self,
        key: &'a str,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> BoxFuture<'a, Result<bool>>;

    /// Adds `by` (negative to decrement) atomically and returns the new value. A missing key
    /// starts at 0; the number is stored as decimal text and keeps any existing TTL.
    fn increment<'a>(&'a self, key: &'a str, by: i64) -> BoxFuture<'a, Result<i64>>;

    /// Removes `key`; `true` when it was there.
    fn forget<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<bool>>;

    /// Removes every key in this store.
    fn flush(&self) -> BoxFuture<'_, Result<()>>;
}
