use sabai_core::DateTime;
use sqlx::error::BoxDynError;
use sqlx::mysql::{MySql, MySqlTypeInfo, MySqlValueRef};
use sqlx::{Decode, Type, TypeInfo};

/// A `timestamp` or `datetime` column, read without the chrono or time crates sqlx decodes
/// dates through.
pub(crate) struct MySqlTimestamp(pub(crate) DateTime);

impl Type<MySql> for MySqlTimestamp {
    fn type_info() -> MySqlTypeInfo {
        <&[u8] as Type<MySql>>::type_info()
    }

    fn compatible(ty: &MySqlTypeInfo) -> bool {
        matches!(ty.name(), "TIMESTAMP" | "DATETIME")
    }
}

impl<'r> Decode<'r, MySql> for MySqlTimestamp {
    fn decode(value: MySqlValueRef<'r>) -> Result<Self, BoxDynError> {
        let bytes = <&[u8] as Decode<MySql>>::decode(value)?;
        if bytes == [0] {
            return Err("the column holds MySQL's zero date `0000-00-00 00:00:00`; \
                        store NULL instead and make the field an `Option`"
                .into());
        }
        let text = match binary_to_text(bytes) {
            Some(text) => text,
            None => std::str::from_utf8(bytes)?.to_owned(),
        };
        let time = text
            .parse()
            .map_err(|error: sabai_core::Error| error.to_string())?;
        Ok(Self(time))
    }
}

// A prepared statement's result sends a length byte, then the year (two bytes, little-endian),
// month and day, then the time only if it is not midnight, then microseconds only if not zero.
// https://dev.mysql.com/doc/dev/mysql-server/latest/page_protocol_binary_resultset.html
fn binary_to_text(bytes: &[u8]) -> Option<String> {
    let (&length, rest) = bytes.split_first()?;
    if usize::from(length) != rest.len() {
        return None;
    }
    let (date, time, micros) = match *rest {
        [y0, y1, month, day] => ([y0, y1, month, day], [0; 3], [0; 4]),
        [y0, y1, month, day, hour, minute, second] => {
            ([y0, y1, month, day], [hour, minute, second], [0; 4])
        }
        [y0, y1, month, day, hour, minute, second, m0, m1, m2, m3] => (
            [y0, y1, month, day],
            [hour, minute, second],
            [m0, m1, m2, m3],
        ),
        _ => return None,
    };
    let [y0, y1, month, day] = date;
    let [hour, minute, second] = time;
    let year = u16::from_le_bytes([y0, y1]);
    let micros = u32::from_le_bytes(micros);
    Some(format!(
        "{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}.{micros:06}"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_full_timestamp_reads_with_its_microseconds() {
        let bytes = [11, 0xEA, 0x07, 10, 23, 10, 5, 9, 0x40, 0xE2, 0x01, 0x00];

        assert_eq!(
            binary_to_text(&bytes).unwrap(),
            "2026-10-23 10:05:09.123456"
        );
    }

    #[test]
    fn midnight_comes_as_the_date_alone() {
        let bytes = [4, 0xEA, 0x07, 10, 23];

        assert_eq!(
            binary_to_text(&bytes).unwrap(),
            "2026-10-23 00:00:00.000000"
        );
    }

    #[test]
    fn a_time_without_microseconds_reads() {
        let bytes = [7, 0xEA, 0x07, 10, 23, 10, 5, 9];

        assert_eq!(
            binary_to_text(&bytes).unwrap(),
            "2026-10-23 10:05:09.000000"
        );
    }

    #[test]
    fn text_is_not_mistaken_for_the_binary_form() {
        assert_eq!(binary_to_text(b"2026-10-23 10:05:09"), None);
    }
}
