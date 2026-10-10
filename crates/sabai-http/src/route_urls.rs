use std::collections::HashMap;

use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};
use sabai_core::{Error, Result, StatusCode};

// Everything but RFC 3986's unreserved characters, so a value can never add a `/`, `?` or `#`.
const COMPONENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

// A `{*rest}` wildcard spans segments, so its `/` stays.
const WILDCARD: &AsciiSet = &COMPONENT.remove(b'/');

/// Path templates of the named routes, such as `posts.show` -> `/posts/{id}`.
#[derive(Default)]
pub(crate) struct RouteUrls {
    paths: HashMap<String, String>,
}

impl RouteUrls {
    /// Remembers `path` under `name`; `false` if the name is taken.
    pub(crate) fn insert(&mut self, name: &str, path: &str) -> bool {
        if self.paths.contains_key(name) {
            return false;
        }
        self.paths.insert(name.to_owned(), path.to_owned());
        true
    }

    /// The URL of the route named `name`. Params the path does not use become the query string.
    pub(crate) fn route(&self, name: &str, params: &[(&str, String)]) -> Result<String> {
        let path = self.paths.get(name).ok_or_else(|| no_route_named(name))?;
        let mut used = vec![false; params.len()];
        let mut url = String::new();
        let mut rest = path.as_str();
        while let Some((before, param, after)) = next_param(rest) {
            url.push_str(before);
            let (key, set) = match param.strip_prefix('*') {
                Some(key) => (key, WILDCARD),
                None => (param, COMPONENT),
            };
            let index = params
                .iter()
                .position(|(given, _)| *given == key)
                .ok_or_else(|| missing_param(name, path, key))?;
            used[index] = true;
            url.extend(utf8_percent_encode(&params[index].1, set));
            rest = after;
        }
        url.push_str(rest);
        let mut query = params.iter().zip(used).filter(|(_, used)| !used);
        if let Some(((key, value), _)) = query.next() {
            push_pair(&mut url, '?', key, value);
            query.for_each(|((key, value), _)| push_pair(&mut url, '&', key, value));
        }
        Ok(url)
    }
}

fn next_param(path: &str) -> Option<(&str, &str, &str)> {
    let (before, rest) = path.split_once('{')?;
    let (param, after) = rest.split_once('}')?;
    Some((before, param, after))
}

fn push_pair(url: &mut String, separator: char, key: &str, value: &str) {
    url.push(separator);
    url.extend(utf8_percent_encode(key, COMPONENT));
    url.push('=');
    url.extend(utf8_percent_encode(value, COMPONENT));
}

fn no_route_named(name: &str) -> Error {
    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(format!(
        "no route is named `{name}`; name one with `route.get(path, handler).name(\"{name}\")`"
    ))
}

fn missing_param(name: &str, path: &str, key: &str) -> Error {
    Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(format!(
        "route `{name}` ({path}) needs `{key}`: `.route(\"{name}\", &[(\"{key}\", ...)])`"
    ))
}
