use std::collections::HashMap;
use std::path::{Path, PathBuf};

use serde_json::{Number, Value};

use super::error::ConfigError;

/// The variables that `${NAME}` placeholders in config files can read.
#[derive(Debug, Default)]
pub(crate) struct Env {
    vars: HashMap<String, String>,
}

impl Env {
    /// The process environment, plus `base/.env` (or `base/.env.{APP_ENV}`) for names it does not set.
    pub(crate) fn load(base: &Path) -> Result<Self, ConfigError> {
        Self::from_process().with_file_from(base)
    }

    fn with_file_from(mut self, base: &Path) -> Result<Self, ConfigError> {
        let path = env_file(base, self.get("APP_ENV"));
        for (name, value) in read_env_file(&path)? {
            self.vars.entry(name).or_insert(value);
        }
        Ok(self)
    }

    /// A snapshot of the process environment; variables that are not valid UTF-8 are skipped.
    pub(crate) fn from_process() -> Self {
        let vars = std::env::vars_os()
            .filter_map(|(name, value)| Some((name.into_string().ok()?, value.into_string().ok()?)))
            .collect();
        Self { vars }
    }

    fn get(&self, name: &str) -> Option<&str> {
        self.vars.get(name).map(String::as_str)
    }
}

fn env_file(base: &Path, app_env: Option<&str>) -> PathBuf {
    app_env
        .map(|app_env| base.join(format!(".env.{app_env}")))
        .filter(|path| path.exists())
        .unwrap_or_else(|| base.join(".env"))
}

fn read_env_file(path: &Path) -> Result<Vec<(String, String)>, ConfigError> {
    let env_file_error = |source| ConfigError::EnvFile {
        path: path.to_owned(),
        source,
    };
    match dotenvy::from_path_iter(path) {
        Ok(lines) => lines.collect::<Result<_, _>>().map_err(env_file_error),
        Err(error) if error.not_found() => Ok(Vec::new()),
        Err(error) => Err(env_file_error(error)),
    }
}

/// Replaces `${NAME}` and `${NAME:-default}` in every string under `value`; `key` names it in errors.
pub(crate) fn interpolate(value: &mut Value, key: &str, env: &Env) -> Result<(), ConfigError> {
    match value {
        Value::String(text) => {
            if let Some(resolved) = resolve(text, key, env)? {
                *value = resolved;
            }
        }
        Value::Array(items) => {
            for (index, item) in items.iter_mut().enumerate() {
                interpolate(item, &format!("{key}.{index}"), env)?;
            }
        }
        Value::Object(fields) => {
            for (name, field) in fields.iter_mut() {
                interpolate(field, &format!("{key}.{name}"), env)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn resolve(text: &str, key: &str, env: &Env) -> Result<Option<Value>, ConfigError> {
    if !text.contains("${") {
        return Ok(None);
    }
    if let Some(placeholder) = whole_placeholder(text) {
        return Ok(Some(typed(lookup(placeholder, key, env)?)));
    }
    Ok(Some(Value::String(expand(text, key, env)?)))
}

fn whole_placeholder(text: &str) -> Option<&str> {
    let inner = text.strip_prefix("${")?.strip_suffix('}')?;
    (!inner.contains(['$', '}'])).then_some(inner)
}

fn expand(text: &str, key: &str, env: &Env) -> Result<String, ConfigError> {
    let unclosed = || ConfigError::UnclosedPlaceholder {
        key: key.to_owned(),
    };
    let mut output = String::with_capacity(text.len());
    let mut rest = text;
    while let Some(start) = rest.find("${") {
        output.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let end = after.find('}').ok_or_else(unclosed)?;
        output.push_str(lookup(&after[..end], key, env)?);
        rest = &after[end + 1..];
    }
    output.push_str(rest);
    Ok(output)
}

fn lookup<'a>(placeholder: &'a str, key: &str, env: &'a Env) -> Result<&'a str, ConfigError> {
    let (name, default) = match placeholder.split_once(":-") {
        Some((name, default)) => (name, Some(default)),
        None => (placeholder, None),
    };
    env.get(name)
        .or(default)
        .ok_or_else(|| ConfigError::MissingEnv {
            key: key.to_owned(),
            name: name.to_owned(),
        })
}

// Like Laravel's env(): "true", "false" and "null" get their types. Numbers follow JSON
// rules, so a value with a leading zero such as a zip code "01234" stays a string.
fn typed(text: &str) -> Value {
    match text {
        "true" => Value::Bool(true),
        "false" => Value::Bool(false),
        "null" => Value::Null,
        _ => text
            .parse::<Number>()
            .map_or_else(|_| text.into(), Value::Number),
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    fn env(vars: &[(&str, &str)]) -> Env {
        let vars = vars
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()));
        Env {
            vars: vars.collect(),
        }
    }

    fn fixtures(dir: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/it/fixtures")
            .join(dir)
    }

    #[test]
    fn os_env_wins_over_the_env_file() {
        let loaded = env(&[("SABAI_TEST_APP_NAME", "From OS")])
            .with_file_from(&fixtures(""))
            .unwrap();

        assert_eq!(loaded.get("SABAI_TEST_APP_NAME"), Some("From OS"));
        assert_eq!(loaded.get("SABAI_TEST_DB"), Some("database.sqlite"));
    }

    #[test]
    fn app_env_picks_its_own_file_and_falls_back_to_env() {
        let testing = env(&[("APP_ENV", "testing")])
            .with_file_from(&fixtures(""))
            .unwrap();
        let staging = env(&[("APP_ENV", "staging")])
            .with_file_from(&fixtures(""))
            .unwrap();

        assert_eq!(testing.get("SABAI_TEST_DB"), Some(":memory:"));
        assert_eq!(staging.get("SABAI_TEST_DB"), Some("database.sqlite"));
    }

    #[test]
    fn a_missing_env_file_is_fine() {
        let loaded = Env::default()
            .with_file_from(&fixtures("does-not-exist"))
            .unwrap();

        assert_eq!(loaded.get("SABAI_TEST_DB"), None);
    }

    #[test]
    fn a_broken_env_file_names_the_file() {
        let error = Env::default()
            .with_file_from(&fixtures("broken-env"))
            .unwrap_err();

        assert!(error.to_string().contains("broken-env/.env"), "{error}");
    }

    fn interpolated(mut value: Value, env: &Env) -> Result<Value, ConfigError> {
        interpolate(&mut value, "app", env)?;
        Ok(value)
    }

    #[test]
    fn a_whole_placeholder_takes_the_type_of_its_value() {
        let env = env(&[("APP_DEBUG", "true"), ("DB_PORT", "5432"), ("ZIP", "01234")]);
        let value = json!({ "debug": "${APP_DEBUG}", "port": "${DB_PORT}", "zip": "${ZIP}" });

        let value = interpolated(value, &env).unwrap();

        assert_eq!(
            value,
            json!({ "debug": true, "port": 5432, "zip": "01234" })
        );
    }

    #[test]
    fn defaults_apply_only_when_the_variable_is_missing() {
        let env = env(&[("APP_NAME", "Blog")]);
        let value = json!(["${APP_NAME:-Sabai}", "${APP_ENV:-local}"]);

        assert_eq!(interpolated(value, &env).unwrap(), json!(["Blog", "local"]));
    }

    #[test]
    fn placeholders_inside_text_stay_text() {
        let env = env(&[("PORT", "3000")]);
        let value = json!("http://${HOST:-localhost}:${PORT}/api");

        assert_eq!(
            interpolated(value, &env).unwrap(),
            json!("http://localhost:3000/api")
        );
    }

    #[test]
    fn a_missing_variable_without_default_explains_the_fix() {
        let value = json!({ "key": "${APP_KEY}" });

        let error = interpolated(value, &Env::default()).unwrap_err();

        assert_eq!(
            error.to_string(),
            "config key `app.key` needs env var `APP_KEY`, which is not set; \
             add `APP_KEY=...` to `.env` or give a default: `${APP_KEY:-value}`"
        );
    }

    #[test]
    fn an_unclosed_placeholder_is_an_error() {
        let error = interpolated(json!("http://${HOST"), &Env::default()).unwrap_err();

        assert!(error.to_string().contains("without a closing `}`"));
    }
}
