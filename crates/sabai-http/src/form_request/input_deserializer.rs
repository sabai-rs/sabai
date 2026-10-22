use serde::de::value::{BorrowedStrDeserializer, Error, MapAccessDeserializer};
use serde::de::{
    self, DeserializeOwned, DeserializeSeed, Deserializer, IntoDeserializer, MapAccess, SeqAccess,
    Visitor,
};
use serde_json::{Map, Value};

/// Builds `T` from input fields, JSON and form alike, through one deserializer so each form
/// request compiles once. Like serde_urlencoded, text becomes a `bool` or a number when the
/// struct asks for one: a form's `draft=true` fills `draft: bool`.
pub(super) fn from_fields<T: DeserializeOwned>(fields: &Map<String, Value>) -> Result<T, Error> {
    T::deserialize(MapAccessDeserializer::new(Fields::new(fields)))
}

struct InputDeserializer<'a>(&'a Value);

struct Fields<'a> {
    fields: serde_json::map::Iter<'a>,
    value: Option<&'a Value>,
}

struct Items<'a>(std::slice::Iter<'a, Value>);

impl<'a> Fields<'a> {
    fn new(fields: &'a Map<String, Value>) -> Self {
        Self {
            fields: fields.iter(),
            value: None,
        }
    }
}

// Asked for a number or a bool, text is parsed first; text that does not parse falls through
// to `deserialize_any`, which reports "invalid type: string, expected ...".
macro_rules! parse_text {
    ($($method:ident => $visit:ident($parsed:ty)),* $(,)?) => {$(
        fn $method<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
            match self.0 {
                Value::String(text) => match text.trim().parse::<$parsed>() {
                    Ok(parsed) => visitor.$visit(parsed),
                    Err(_) => self.deserialize_any(visitor),
                },
                _ => self.deserialize_any(visitor),
            }
        }
    )*};
}

impl<'de> Deserializer<'de> for InputDeserializer<'de> {
    type Error = Error;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Null => visitor.visit_unit(),
            Value::Bool(value) => visitor.visit_bool(*value),
            Value::Number(number) => match (number.as_u64(), number.as_i64()) {
                (Some(value), _) => visitor.visit_u64(value),
                (_, Some(value)) => visitor.visit_i64(value),
                _ => visitor.visit_f64(number.as_f64().unwrap_or(f64::NAN)),
            },
            Value::String(text) => visitor.visit_borrowed_str(text),
            Value::Array(items) => visitor.visit_seq(Items(items.iter())),
            Value::Object(fields) => visitor.visit_map(Fields::new(fields)),
        }
    }

    // Laravel's `boolean` rule takes the same four spellings.
    fn deserialize_bool<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0.as_str().map(str::trim) {
            Some("true" | "1") => visitor.visit_bool(true),
            Some("false" | "0") => visitor.visit_bool(false),
            _ => self.deserialize_any(visitor),
        }
    }

    parse_text! {
        deserialize_i8 => visit_i64(i64),
        deserialize_i16 => visit_i64(i64),
        deserialize_i32 => visit_i64(i64),
        deserialize_i64 => visit_i64(i64),
        deserialize_u8 => visit_u64(u64),
        deserialize_u16 => visit_u64(u64),
        deserialize_u32 => visit_u64(u64),
        deserialize_u64 => visit_u64(u64),
        deserialize_f32 => visit_f64(f64),
        deserialize_f64 => visit_f64(f64),
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Error> {
        match self.0 {
            Value::Null => visitor.visit_none(),
            _ => visitor.visit_some(self),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Error> {
        match self.0 {
            Value::String(text) => visitor.visit_enum(text.as_str().into_deserializer()),
            _ => self.deserialize_any(visitor),
        }
    }

    serde::forward_to_deserialize_any! {
        i128 u128 char str string bytes byte_buf unit unit_struct seq tuple tuple_struct map
        struct identifier ignored_any
    }
}

impl<'de> MapAccess<'de> for Fields<'de> {
    type Error = Error;

    fn next_key_seed<K: DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Error> {
        let Some((name, value)) = self.fields.next() else {
            return Ok(None);
        };
        self.value = Some(value);
        seed.deserialize(BorrowedStrDeserializer::new(name))
            .map(Some)
    }

    fn next_value_seed<S: DeserializeSeed<'de>>(&mut self, seed: S) -> Result<S::Value, Error> {
        let value = self
            .value
            .take()
            .ok_or_else(|| de::Error::custom("a value was asked for before its field"))?;
        seed.deserialize(InputDeserializer(value))
    }
}

impl<'de> SeqAccess<'de> for Items<'de> {
    type Error = Error;

    fn next_element_seed<S: DeserializeSeed<'de>>(
        &mut self,
        seed: S,
    ) -> Result<Option<S::Value>, Error> {
        self.0
            .next()
            .map(|item| seed.deserialize(InputDeserializer(item)))
            .transpose()
    }
}

#[cfg(test)]
mod tests {
    use serde::Deserialize;
    use serde_json::json;

    use super::*;

    #[derive(Debug, Deserialize, PartialEq)]
    #[serde(rename_all = "lowercase")]
    enum Status {
        Draft,
        Published,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Item {
        qty: u32,
    }

    #[derive(Debug, Deserialize, PartialEq)]
    struct Order {
        note: String,
        paid: bool,
        total: f64,
        coupon: Option<String>,
        status: Status,
        items: Vec<Item>,
    }

    fn order(input: serde_json::Value) -> Result<Order, Error> {
        from_fields(input.as_object().unwrap())
    }

    fn expected() -> Order {
        Order {
            note: "42".to_owned(),
            paid: true,
            total: 9.5,
            coupon: None,
            status: Status::Published,
            items: vec![Item { qty: 2 }],
        }
    }

    #[test]
    fn typed_json_fills_the_struct() {
        let input = json!({"note": "42", "paid": true, "total": 9.5, "status": "published", "items": [{"qty": 2}]});

        assert_eq!(order(input).unwrap(), expected());
    }

    #[test]
    fn text_becomes_a_number_or_a_bool_only_where_the_struct_asks_for_one() {
        let input = json!({"note": "42", "paid": "1", "total": " 9.5 ", "coupon": null, "status": "published", "items": [{"qty": "2"}]});

        assert_eq!(order(input).unwrap(), expected());
    }

    #[test]
    fn text_that_is_not_a_number_says_what_was_expected() {
        let input = json!({"note": "", "paid": "yes", "total": 1, "status": "draft", "items": []});

        let error = order(input).unwrap_err().to_string();

        assert!(
            error.contains("invalid type: string \"yes\", expected a boolean"),
            "{error}"
        );
    }
}
