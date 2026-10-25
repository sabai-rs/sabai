use std::time::SystemTime;

use sabai_core::contracts::Clock;
use sabai_core::{App, DateTime, Result};

/// A field `save()` can fill with the current time: `DateTime` or `Option<DateTime>`.
pub trait Timestamp {
    /// Sets the field to `time`.
    fn set(&mut self, time: DateTime);
}

impl Timestamp for DateTime {
    fn set(&mut self, time: DateTime) {
        *self = time;
    }
}

impl Timestamp for Option<DateTime> {
    fn set(&mut self, time: DateTime) {
        *self = Some(time);
    }
}

/// A model's `created_at` and `updated_at` fields, which `#[derive(Model)]` hands to `save()`.
pub struct Timestamps<'a> {
    /// Set when the row is inserted.
    pub created_at: &'a mut dyn Timestamp,
    /// Set on every save.
    pub updated_at: &'a mut dyn Timestamp,
}

pub(crate) fn touch(timestamps: Option<Timestamps<'_>>, inserting: bool) -> Result<()> {
    let Some(timestamps) = timestamps else {
        return Ok(());
    };
    // Whole seconds, like Laravel, so the saved model matches what a `timestamp` column keeps.
    let now = DateTime::try_from(now())?.start_of_second();
    if inserting {
        timestamps.created_at.set(now);
    }
    timestamps.updated_at.set(now);
    Ok(())
}

// An app without a bound `Clock` reads the system's, like Carbon without `setTestNow`.
fn now() -> SystemTime {
    App::current()
        .and_then(|app| app.get::<dyn Clock>())
        .map_or_else(|_| SystemTime::now(), |clock| clock.now())
}
