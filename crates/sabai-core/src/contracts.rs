//! Traits that drivers, fakes and packages implement, like Laravel's `Illuminate\Contracts`.
//! Code depends on these, never on a concrete driver, so tests and config can swap the driver.

mod cache;
mod clock;
mod mail;
mod storage;

use std::future::Future;
use std::pin::Pin;

pub use cache::CacheStore;
pub use clock::{Clock, SystemClock};
pub use mail::{Address, MailTransport, Message, MessageBuilder};
pub use storage::{FileMetadata, StorageDriver};

// `async fn` in traits is not `dyn`-compatible yet; this is what `async-trait` expands to, without `syn`.
/// A boxed `Send` future, returned by async contract methods so the traits work as `Arc<dyn Trait>`.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
