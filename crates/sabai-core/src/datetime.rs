use std::fmt;
use std::str::FromStr;
use std::time::SystemTime;

use jiff::Timestamp;
use jiff::civil;
use jiff::tz::Offset;
use serde::de::{self, Deserialize, Deserializer};
use serde::ser::{Serialize, Serializer};

use crate::{Error, Result, StatusCode};

const DATABASE_FORMAT: &str = "%Y-%m-%d %H:%M:%S";
const JSON_FORMAT: &str = "%Y-%m-%dT%H:%M:%S%.6fZ";

/// A moment in time, kept in UTC: what a `timestamp` column holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DateTime(Timestamp);

impl DateTime {
    /// The current time, read from the system clock.
    pub fn now() -> Self {
        Self(Timestamp::now())
    }

    /// The time in JSON's form, with microseconds: `2026-10-23T10:00:00.000000Z`.
    pub fn to_json(&self) -> String {
        self.0.strftime(JSON_FORMAT).to_string()
    }
}

/// Reads a `Clock`'s time; fails only past the year 9999.
impl TryFrom<SystemTime> for DateTime {
    type Error = Error;

    fn try_from(time: SystemTime) -> Result<Self> {
        Ok(Self(Timestamp::try_from(time)?))
    }
}

/// Prints the form a database stores: `2026-10-23 10:00:00`.
impl fmt::Display for DateTime {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.strftime(DATABASE_FORMAT))
    }
}

/// Reads a time with an offset (`2026-10-23T10:00:00Z`), or the database form, taken as UTC.
impl FromStr for DateTime {
    type Err = Error;

    fn from_str(input: &str) -> Result<Self> {
        if let Ok(timestamp) = input.parse::<Timestamp>() {
            return Ok(Self(timestamp));
        }
        let civil: civil::DateTime = input.parse().map_err(|_| not_a_date_time(input))?;
        Ok(Self(Offset::UTC.to_timestamp(civil)?))
    }
}

impl Serialize for DateTime {
    fn serialize<S: Serializer>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_json())
    }
}

impl<'de> Deserialize<'de> for DateTime {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> std::result::Result<Self, D::Error> {
        let input = String::deserialize(deserializer)?;
        input.parse().map_err(de::Error::custom)
    }
}

fn not_a_date_time(input: &str) -> Error {
    Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("`{input}` is not a date and time like `2026-10-23 10:00:00`"),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prints_the_form_a_database_stores() {
        let time: DateTime = "2026-10-23T10:05:09.123456Z".parse().unwrap();

        assert_eq!(time.to_string(), "2026-10-23 10:05:09");
    }

    #[test]
    fn reads_the_database_form_as_utc() {
        let time: DateTime = "2026-10-23 10:05:09".parse().unwrap();

        assert_eq!(time.to_json(), "2026-10-23T10:05:09.000000Z");
    }

    #[test]
    fn a_time_with_an_offset_becomes_utc() {
        let time: DateTime = "2026-10-23T17:05:09+07:00".parse().unwrap();

        assert_eq!(time.to_string(), "2026-10-23 10:05:09");
    }

    #[test]
    fn json_has_microseconds_and_a_z() {
        let time: DateTime = "2026-10-23T10:05:09.5Z".parse().unwrap();

        let json = serde_json::to_string(&time).unwrap();

        assert_eq!(json, r#""2026-10-23T10:05:09.500000Z""#);
    }

    #[test]
    fn json_reads_back_to_the_same_time() {
        let time: DateTime = "2026-10-23T10:05:09.123456Z".parse().unwrap();

        let read: DateTime = serde_json::from_str(&serde_json::to_string(&time).unwrap()).unwrap();

        assert_eq!(read, time);
    }

    #[test]
    fn json_already_read_into_a_value_reads_too() {
        let value = serde_json::json!("2026-10-23T10:05:09Z");

        let time: DateTime = serde_json::from_value(value).unwrap();

        assert_eq!(time.to_string(), "2026-10-23 10:05:09");
    }

    #[test]
    fn a_system_time_converts() {
        let time = SystemTime::UNIX_EPOCH + std::time::Duration::from_secs(86_400);

        let time = DateTime::try_from(time).unwrap();

        assert_eq!(time.to_string(), "1970-01-02 00:00:00");
    }

    #[test]
    fn text_that_is_not_a_time_says_what_it_expects() {
        let error = "yesterday".parse::<DateTime>().err().unwrap().to_string();

        assert!(
            error.contains("`yesterday` is not a date and time like `2026-10-23 10:00:00`"),
            "{error}"
        );
    }
}
