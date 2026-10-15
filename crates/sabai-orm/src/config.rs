use std::collections::HashMap;

use percent_encoding::{NON_ALPHANUMERIC, utf8_percent_encode};
use sabai_core::{Config, Error, Result, StatusCode};
use serde::{Deserialize, Deserializer};

/// `config/database.toml`, shaped like Laravel's `config/database.php`.
#[derive(Deserialize)]
struct DatabaseConfig {
    default: String,
    connections: HashMap<String, Connection>,
}

#[derive(Deserialize)]
#[serde(tag = "driver", rename_all = "lowercase")]
enum Connection {
    Sqlite {
        #[serde(deserialize_with = "text")]
        database: String,
    },
    Mysql {
        #[serde(deserialize_with = "text")]
        host: String,
        port: u16,
        #[serde(deserialize_with = "text")]
        database: String,
        #[serde(deserialize_with = "text")]
        username: String,
        #[serde(deserialize_with = "text")]
        password: String,
    },
}

/// Where the default connection points, as sqlx wants it.
pub(crate) struct Target {
    pub(crate) url: String,
    pub(crate) in_memory: bool,
}

pub(crate) fn default_target(config: &Config) -> Result<Target> {
    let config: DatabaseConfig = config.get("database")?;
    let connection = config.connections.get(&config.default).ok_or_else(|| {
        Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(format!(
            "config/database.toml: `default` is `{0}`, but there is no `[connections.{0}]`",
            config.default
        ))
    })?;
    Ok(target(connection))
}

fn target(connection: &Connection) -> Target {
    match connection {
        Connection::Sqlite { database } if database == ":memory:" => Target {
            url: "sqlite::memory:".to_owned(),
            in_memory: true,
        },
        Connection::Sqlite { database } => Target {
            url: format!("sqlite://{database}?mode=rwc"),
            in_memory: false,
        },
        Connection::Mysql {
            host,
            port,
            database,
            username,
            password,
        } => Target {
            url: format!(
                "mysql://{}:{}@{host}:{port}/{}",
                encode(username),
                encode(password),
                encode(database)
            ),
            in_memory: false,
        },
    }
}

fn encode(part: &str) -> String {
    utf8_percent_encode(part, NON_ALPHANUMERIC).to_string()
}

// `${DB_PASSWORD}` set to `12345` arrives as a number, like any env value that looks like one.
fn text<'de, D: Deserializer<'de>>(deserializer: D) -> Result<String, D::Error> {
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Text {
        String(String),
        Integer(i64),
        Float(f64),
    }
    Ok(match Text::deserialize(deserializer)? {
        Text::String(text) => text,
        Text::Integer(number) => number.to_string(),
        Text::Float(number) => number.to_string(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mysql(password: &str) -> Connection {
        Connection::Mysql {
            host: "127.0.0.1".to_owned(),
            port: 3306,
            database: "blog".to_owned(),
            username: "sabai".to_owned(),
            password: password.to_owned(),
        }
    }

    #[test]
    fn a_mysql_password_is_percent_encoded_so_it_cannot_break_the_url() {
        let target = target(&mysql("p@ss/w:rd"));

        assert_eq!(
            target.url,
            "mysql://sabai:p%40ss%2Fw%3Ard@127.0.0.1:3306/blog"
        );
    }

    #[test]
    fn sqlite_memory_is_marked_so_the_pool_keeps_one_connection() {
        let memory = target(&Connection::Sqlite {
            database: ":memory:".to_owned(),
        });
        let file = target(&Connection::Sqlite {
            database: "database/database.sqlite".to_owned(),
        });

        assert!(memory.in_memory);
        assert_eq!(file.url, "sqlite://database/database.sqlite?mode=rwc");
        assert!(!file.in_memory);
    }
}
