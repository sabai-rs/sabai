use serde_json::{Map, Value};

/// Every field `pattern` names in `input`, by its full name, like Laravel: `posts.*.title`
/// gives `posts.0.title`, `posts.1.title`, ... A `*` over anything but a list names nothing.
pub(super) fn matching_fields<'a>(
    pattern: &str,
    input: &'a Map<String, Value>,
) -> Vec<(String, Option<&'a Value>)> {
    let mut found = Vec::new();
    let segments: Vec<&str> = pattern.split('.').collect();
    let Some((first, rest)) = segments.split_first() else {
        return found;
    };
    walk(rest, (*first).to_owned(), input.get(*first), &mut found);
    found
}

fn walk<'a>(
    segments: &[&str],
    name: String,
    value: Option<&'a Value>,
    found: &mut Vec<(String, Option<&'a Value>)>,
) {
    let Some((segment, rest)) = segments.split_first() else {
        found.push((name, value));
        return;
    };
    if *segment == "*" {
        if let Some(Value::Array(items)) = value {
            for (index, item) in items.iter().enumerate() {
                walk(rest, format!("{name}.{index}"), Some(item), found);
            }
        }
        return;
    }
    let child = match value {
        Some(Value::Object(fields)) => fields.get(*segment),
        Some(Value::Array(items)) => segment
            .parse()
            .ok()
            .and_then(|index: usize| items.get(index)),
        _ => None,
    };
    walk(rest, format!("{name}.{segment}"), child, found);
}
