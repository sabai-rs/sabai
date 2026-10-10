mod env;
mod error;

use std::fs;
use std::path::Path;

use serde::de::DeserializeOwned;
use serde_json::{Map, Value};

use crate::{Error, Result};
use env::{Env, interpolate};
use error::ConfigError;

/// All app config, loaded from `config/*.toml`: `app.toml` becomes the `app` section, and so on.
#[derive(Debug, Default)]
pub struct Config {
    root: Map<String, Value>,
}

impl Config {
    /// Loads `base/config/*.toml`, one section per file, filling `${NAME}` from the env and `base/.env`.
    pub fn load(base: impl AsRef<Path>) -> Result<Self> {
        Self::load_base(base.as_ref())
    }

    fn load_base(base: &Path) -> Result<Self> {
        let env = Env::load(base)?;
        Self::load_dir(&base.join("config"), &env)
    }

    /// Reads a section or a dotted key, like Laravel's `config('app.name')`, into any deserializable type.
    pub fn get<T: DeserializeOwned>(&self, key: &str) -> Result<T> {
        let value = self.value(key)?;
        T::deserialize(value).map_err(|source| invalid(key, source))
    }

    fn load_dir(dir: &Path, env: &Env) -> Result<Self> {
        let read_dir = |source| ConfigError::ReadDir {
            dir: dir.to_owned(),
            source,
        };
        let mut config = Self::default();
        for entry in fs::read_dir(dir).map_err(read_dir)? {
            let path = entry.map_err(read_dir)?.path();
            if path
                .extension()
                .is_some_and(|extension| extension == "toml")
            {
                config.load_file(&path, env)?;
            }
        }
        Ok(config)
    }

    fn load_file(&mut self, path: &Path, env: &Env) -> Result<()> {
        let text = fs::read_to_string(path).map_err(|source| ConfigError::ReadFile {
            path: path.to_owned(),
            source,
        })?;
        Ok(self.insert_section(path, &text, env)?)
    }

    fn insert_section(
        &mut self,
        path: &Path,
        toml_text: &str,
        env: &Env,
    ) -> Result<(), ConfigError> {
        let parse_error = |source| ConfigError::Parse {
            path: path.to_owned(),
            source,
        };
        let mut section: Value = toml::from_str(toml_text).map_err(parse_error)?;
        let name = path.file_stem().unwrap_or_default().to_string_lossy();
        interpolate(&mut section, &name, env)?;
        self.root.insert(name.into_owned(), section);
        Ok(())
    }

    fn value(&self, key: &str) -> Result<&Value> {
        let mut segments = key.split('.');
        segments
            .next()
            .and_then(|section| self.root.get(section))
            .and_then(|section| segments.try_fold(section, |value, segment| value.get(segment)))
            .ok_or_else(|| missing(key))
    }
}

fn missing(key: &str) -> Error {
    ConfigError::Missing {
        key: key.to_owned(),
    }
    .into()
}

fn invalid(key: &str, source: serde_json::Error) -> Error {
    ConfigError::Invalid {
        key: key.to_owned(),
        source,
    }
    .into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(section: &str, toml_text: &str) -> Config {
        let mut config = Config::default();
        let path = Path::new("config").join(format!("{section}.toml"));
        config
            .insert_section(&path, toml_text, &Env::default())
            .unwrap();
        config
    }

    #[test]
    fn dotted_keys_reach_nested_values() {
        let config = config("database", "[connections.pg]\nport = 5432\n");

        assert_eq!(
            config.get::<u16>("database.connections.pg.port").unwrap(),
            5432
        );
    }

    #[test]
    fn a_missing_key_says_which_file_to_add_it_to() {
        let config = config("app", "name = \"Blog\"\n");

        let error = config.get::<String>("mail.host").unwrap_err();

        assert_eq!(
            error.to_string(),
            "Server Error: config key `mail.host` is not set; add it to `config/mail.toml`"
        );
    }

    #[test]
    fn a_wrong_type_names_the_key() {
        let config = config("app", "debug = \"yes\"\n");

        let error = config.get::<bool>("app.debug").unwrap_err();

        assert!(
            error
                .to_string()
                .contains("config key `app.debug` does not match")
        );
    }

    #[test]
    fn invalid_toml_points_at_the_line() {
        let error = Config::default()
            .insert_section(
                Path::new("config/app.toml"),
                "port = \"80\n",
                &Env::default(),
            )
            .unwrap_err();

        assert!(error.to_string().contains("line 1"), "{error}");
    }
}
