use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use pollster::block_on;
use sabai::contracts::{BoxFuture, CacheStore, Clock};
use sabai::{Bytes, Container, Result};

use super::FrozenClock;

type Entries = HashMap<String, (Bytes, Option<SystemTime>)>;

/// An in-memory cache driver. It asks the clock, never `SystemTime::now()`, whether a value expired,
/// and holds the lock for a whole read-modify-write so `add` and `increment` are atomic.
struct MemoryStore {
    clock: Arc<dyn Clock>,
    entries: Mutex<Entries>,
}

impl MemoryStore {
    fn fresh<'e>(
        &self,
        entries: &'e Entries,
        key: &str,
    ) -> Option<&'e (Bytes, Option<SystemTime>)> {
        let now = self.clock.now();
        entries
            .get(key)
            .filter(|(_, expires)| expires.is_none_or(|at| at > now))
    }

    fn expiry(&self, ttl: Option<Duration>) -> Option<SystemTime> {
        ttl.map(|ttl| self.clock.now() + ttl)
    }
}

impl CacheStore for MemoryStore {
    fn get<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<Option<Bytes>>> {
        Box::pin(async move {
            let entries = self.entries.lock().unwrap();
            Ok(self.fresh(&entries, key).map(|(value, _)| value.clone()))
        })
    }

    fn put<'a>(
        &'a self,
        key: &'a str,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> BoxFuture<'a, Result<()>> {
        Box::pin(async move {
            let expires = self.expiry(ttl);
            self.entries
                .lock()
                .unwrap()
                .insert(key.to_owned(), (value, expires));
            Ok(())
        })
    }

    fn add<'a>(
        &'a self,
        key: &'a str,
        value: Bytes,
        ttl: Option<Duration>,
    ) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move {
            let mut entries = self.entries.lock().unwrap();
            if self.fresh(&entries, key).is_some() {
                return Ok(false);
            }
            entries.insert(key.to_owned(), (value, self.expiry(ttl)));
            Ok(true)
        })
    }

    fn increment<'a>(&'a self, key: &'a str, by: i64) -> BoxFuture<'a, Result<i64>> {
        Box::pin(async move {
            let mut entries = self.entries.lock().unwrap();
            let (current, expires) = match self.fresh(&entries, key) {
                Some((value, expires)) => (std::str::from_utf8(value)?.parse::<i64>()?, *expires),
                None => (0, None),
            };
            let next = current + by;
            entries.insert(key.to_owned(), (Bytes::from(next.to_string()), expires));
            Ok(next)
        })
    }

    fn forget<'a>(&'a self, key: &'a str) -> BoxFuture<'a, Result<bool>> {
        Box::pin(async move { Ok(self.entries.lock().unwrap().remove(key).is_some()) })
    }

    fn flush(&self) -> BoxFuture<'_, Result<()>> {
        Box::pin(async move {
            self.entries.lock().unwrap().clear();
            Ok(())
        })
    }
}

fn store_with_frozen_time() -> (Arc<dyn CacheStore>, Arc<FrozenClock>) {
    let clock = Arc::new(FrozenClock::at_epoch());
    let store = MemoryStore {
        clock: clock.clone(),
        entries: Mutex::default(),
    };
    let mut container = Container::default();
    container.bind::<dyn CacheStore>(Arc::new(store));
    (container.get::<dyn CacheStore>().unwrap(), clock)
}

const MINUTE: Option<Duration> = Some(Duration::from_secs(60));

#[test]
fn cached_values_expire_when_the_clock_says_so() {
    let (store, clock) = store_with_frozen_time();

    block_on(store.put("posts.count", Bytes::from("42"), MINUTE)).unwrap();
    let before = block_on(store.get("posts.count")).unwrap();
    clock.travel(Duration::from_secs(61));
    let after = block_on(store.get("posts.count")).unwrap();

    assert_eq!(before, Some(Bytes::from("42")));
    assert_eq!(after, None);
}

#[test]
fn add_only_stores_when_the_key_is_missing_or_expired() {
    let (store, clock) = store_with_frozen_time();

    let first = block_on(store.add("lock", Bytes::from("a"), MINUTE)).unwrap();
    let second = block_on(store.add("lock", Bytes::from("b"), MINUTE)).unwrap();
    clock.travel(Duration::from_secs(61));
    let after_expiry = block_on(store.add("lock", Bytes::from("c"), MINUTE)).unwrap();

    assert_eq!((first, second, after_expiry), (true, false, true));
    assert_eq!(block_on(store.get("lock")).unwrap(), Some(Bytes::from("c")));
}

#[test]
fn increment_starts_at_zero_and_stores_decimal_text() {
    let (store, _clock) = store_with_frozen_time();

    let counts = [1, 5, -2].map(|by| block_on(store.increment("views", by)).unwrap());

    assert_eq!(counts, [1, 6, 4]);
    assert_eq!(
        block_on(store.get("views")).unwrap(),
        Some(Bytes::from("4"))
    );
}

#[test]
fn incrementing_text_is_an_error() {
    let (store, _clock) = store_with_frozen_time();
    block_on(store.put("name", Bytes::from("Ann"), None)).unwrap();

    assert!(block_on(store.increment("name", 1)).is_err());
}

#[test]
fn forget_reports_whether_the_key_was_there_and_flush_clears_all() {
    let (store, _clock) = store_with_frozen_time();
    block_on(store.put("a", Bytes::from("1"), None)).unwrap();
    block_on(store.put("b", Bytes::from("2"), None)).unwrap();

    let forgotten = [
        block_on(store.forget("a")).unwrap(),
        block_on(store.forget("a")).unwrap(),
    ];
    block_on(store.flush()).unwrap();

    assert_eq!(forgotten, [true, false]);
    assert_eq!(block_on(store.get("b")).unwrap(), None);
}
