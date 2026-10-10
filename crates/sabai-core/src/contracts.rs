//! Traits that drivers, fakes and packages implement, like Laravel's `Illuminate\Contracts`.
//! Code depends on these, never on a concrete driver, so tests and config can swap the driver.

mod cache;
mod clock;

use std::future::Future;
use std::pin::Pin;

pub use cache::CacheStore;
pub use clock::{Clock, SystemClock};

// Native `async fn` in traits cannot be used as `dyn Trait` yet, and drivers are picked
// from config at runtime, so async methods return this instead. It is what `async-trait`
// generates, written by hand to keep `syn` out of core.
/// A boxed `Send` future, returned by async contract methods so the traits work as `Arc<dyn Trait>`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
