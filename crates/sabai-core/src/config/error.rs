use std::error::Error as StdError;
use std::fmt;
use std::io;
use std::path::PathBuf;

/// Why loading or reading config failed, worded so the message says how to fix it.
#[derive(Debug)]
pub(crate) enum ConfigError {
    ReadDir {
        dir: PathBuf,
        source: io::Error,
    },
    ReadFile {
        path: PathBuf,
        source: io::Error,
    },
    Parse {
        path: PathBuf,
        source: toml::de::Error,
    },
    Missing {
        key: String,
    },
    Invalid {
        key: String,
        source: serde_json::Error,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ReadDir { dir, source } => {
                write!(
                    f,
                    "could not read the config directory `{}`: {source}",
                    dir.display()
                )
            }
            Self::ReadFile { path, source } => {
                write!(f, "could not read `{}`: {source}", path.display())
            }
            Self::Parse { path, source } => {
                write!(f, "`{}` is not valid TOML\n{source}", path.display())
            }
            Self::Missing { key } => {
                let file = key.split('.').next().unwrap_or(key);
                write!(
                    f,
                    "config key `{key}` is not set; add it to `config/{file}.toml`"
                )
            }
            Self::Invalid { key, source } => {
                write!(
                    f,
                    "config key `{key}` does not match the expected type: {source}"
                )
            }
        }
    }
}

impl StdError for ConfigError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::ReadDir { source, .. } | Self::ReadFile { source, .. } => Some(source),
            Self::Parse { source, .. } => Some(source),
            Self::Invalid { source, .. } => Some(source),
            Self::Missing { .. } => None,
        }
    }
}
