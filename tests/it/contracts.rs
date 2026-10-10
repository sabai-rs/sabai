//! Contracts implemented the way a driver author would, to prove they are implementable.

mod cache;
mod mail;
mod storage;

use std::sync::Mutex;
use std::time::{Duration, SystemTime};

use sabai::contracts::Clock;

/// A clock that only moves when told to, like Laravel's `travel()`.
struct FrozenClock(Mutex<SystemTime>);

impl FrozenClock {
    fn at_epoch() -> Self {
        Self(Mutex::new(SystemTime::UNIX_EPOCH))
    }

    fn travel(&self, by: Duration) {
        *self.0.lock().unwrap() += by;
    }
}

impl Clock for FrozenClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().unwrap()
    }
}
