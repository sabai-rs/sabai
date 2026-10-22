use sabai_core::{App, Config, Error, Result, StatusCode, log};

use super::{argon, bcrypt};

/// `config/hashing.toml`, with the keys and defaults of Laravel's `config/hashing.php`.
#[derive(Clone, Copy)]
pub(super) struct Settings {
    driver: Driver,
    verify: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Driver {
    Bcrypt {
        rounds: u32,
        limit: usize,
    },
    Argon2id {
        memory: u32,
        time: u32,
        threads: u32,
    },
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Algorithm {
    Bcrypt,
    Argon2id,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            driver: Driver::Bcrypt {
                rounds: 12,
                limit: bcrypt::DEFAULT_LIMIT,
            },
            verify: true,
        }
    }
}

impl Settings {
    /// The current app's settings, or Laravel's defaults outside an app.
    pub(super) fn current() -> Result<Self> {
        match App::current() {
            Ok(app) => Self::from_config(app.config()),
            Err(_) => Ok(Self::default()),
        }
    }

    pub(super) fn from_config(config: &Config) -> Result<Self> {
        let driver: String = config.get_or("hashing.driver", "bcrypt".to_owned())?;
        let (driver, verify_key) = match driver.as_str() {
            "bcrypt" => (
                Driver::Bcrypt {
                    rounds: config.get_or("hashing.bcrypt.rounds", 12)?,
                    limit: config.get_or("hashing.bcrypt.limit", bcrypt::DEFAULT_LIMIT)?,
                },
                "hashing.bcrypt.verify",
            ),
            "argon2id" if cfg!(not(feature = "argon2")) => return Err(argon2_needs_its_feature()),
            "argon2id" => (
                Driver::Argon2id {
                    memory: config.get_or("hashing.argon.memory", 65536)?,
                    time: config.get_or("hashing.argon.time", 4)?,
                    threads: config.get_or("hashing.argon.threads", 1)?,
                },
                "hashing.argon.verify",
            ),
            other => return Err(unknown_driver(other)),
        };
        Ok(Self {
            driver,
            verify: config.get_or(verify_key, true)?,
        })
    }

    pub(super) fn make(self, password: &str) -> Result<String> {
        match self.driver {
            Driver::Bcrypt { rounds, limit } => bcrypt::make(password, rounds, limit),
            Driver::Argon2id {
                memory,
                time,
                threads,
            } => argon::make(password, memory, time, threads),
        }
    }

    /// Whether `hashed` is one this app checks: with `verify` on, like Laravel, only the driver's own.
    pub(super) fn accepts(self, hashed: &str) -> bool {
        let Some(algorithm) = Algorithm::of(hashed) else {
            return false;
        };
        let driver = self.driver.algorithm();
        if self.verify && algorithm != driver {
            log::warn!(
                "a {algorithm:?} password hash was checked while hashing.driver is {driver:?}; \
                 set its `verify` to false in config/hashing.toml while moving users over"
            );
            return false;
        }
        true
    }

    pub(super) fn needs_rehash(self, hashed: &str) -> bool {
        match self.driver {
            Driver::Bcrypt { rounds, .. } => {
                Algorithm::of(hashed) != Some(Algorithm::Bcrypt)
                    || bcrypt::cost(hashed) != Some(rounds)
            }
            Driver::Argon2id {
                memory,
                time,
                threads,
            } => {
                Algorithm::of(hashed) != Some(Algorithm::Argon2id)
                    || argon::costs(hashed) != Some((memory, time, threads))
            }
        }
    }
}

/// Checks `password` against `hashed` with the algorithm `hashed` names.
pub(super) fn matches(password: &str, hashed: &str) -> bool {
    match Algorithm::of(hashed) {
        Some(Algorithm::Bcrypt) => bcrypt::matches(password, hashed),
        Some(Algorithm::Argon2id) => argon::matches(password, hashed),
        None => false,
    }
}

impl Driver {
    fn algorithm(self) -> Algorithm {
        match self {
            Driver::Bcrypt { .. } => Algorithm::Bcrypt,
            Driver::Argon2id { .. } => Algorithm::Argon2id,
        }
    }
}

impl Algorithm {
    fn of(hashed: &str) -> Option<Self> {
        match hashed.get(..4)? {
            "$2y$" | "$2b$" | "$2a$" => Some(Algorithm::Bcrypt),
            _ if hashed.starts_with("$argon2id$") => Some(Algorithm::Argon2id),
            _ => None,
        }
    }
}

pub(super) fn argon2_needs_its_feature() -> Error {
    Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "Argon2id needs the `argon2` feature: sabai = { ..., features = [\"auth\", \"argon2\"] }",
    )
}

fn unknown_driver(driver: &str) -> Error {
    Error::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        format!(
            "`hashing.driver` in config/hashing.toml is `{driver}`; use `bcrypt` or `argon2id`"
        ),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(hashing_toml: &str) -> Result<Settings> {
        let base = std::env::temp_dir().join(format!(
            "sabai-hashing-{}-{}",
            std::process::id(),
            hashing_toml.len()
        ));
        std::fs::create_dir_all(base.join("config")).unwrap();
        std::fs::write(base.join("config/hashing.toml"), hashing_toml).unwrap();
        Settings::from_config(&Config::load(&base).unwrap())
    }

    #[cfg(feature = "argon2")]
    fn argon2id() -> Settings {
        settings("driver = \"argon2id\"\n[argon]\nmemory = 19456\ntime = 2\n").unwrap()
    }

    #[cfg(feature = "argon2")]
    #[test]
    fn laravels_keys_pick_the_driver_and_its_costs() {
        let argon2id = argon2id();

        let hashed = argon2id.make("secret").unwrap();

        assert!(
            hashed.starts_with("$argon2id$v=19$m=19456,t=2,p=1$"),
            "{hashed}"
        );
        assert!(matches("secret", &hashed));
        assert!(!argon2id.needs_rehash(&hashed));
    }

    #[cfg(feature = "argon2")]
    #[test]
    fn with_verify_on_only_the_drivers_own_hashes_are_checked() {
        let argon_hash = argon2id().make("secret").unwrap();
        let bcrypt = Settings::default();
        let moving_over = settings("[bcrypt]\nverify = false\n").unwrap();

        assert!(!bcrypt.accepts(&argon_hash));
        assert!(moving_over.accepts(&argon_hash));
        assert!(bcrypt.needs_rehash(&argon_hash));
    }

    #[cfg(feature = "argon2")]
    #[test]
    fn a_hash_from_another_driver_or_with_other_costs_needs_a_rehash() {
        let bcrypt_hash = Settings::default().make("secret").unwrap();
        let stronger = settings("[bcrypt]\nrounds = 13\n").unwrap();

        assert!(argon2id().needs_rehash(&bcrypt_hash));
        assert!(stronger.needs_rehash(&bcrypt_hash));
    }

    #[test]
    fn an_unknown_driver_says_which_ones_exist() {
        let error = settings("driver = \"argon\"\n").err().unwrap();

        assert_eq!(
            error.to_string(),
            "`hashing.driver` in config/hashing.toml is `argon`; use `bcrypt` or `argon2id`"
        );
    }

    #[cfg(not(feature = "argon2"))]
    #[test]
    fn argon2id_without_its_feature_says_how_to_turn_it_on() {
        let error = settings("driver = \"argon2id\"\n").err().unwrap();

        assert_eq!(
            error.to_string(),
            "Argon2id needs the `argon2` feature: sabai = { ..., features = [\"auth\", \"argon2\"] }"
        );
    }

    #[test]
    fn bcrypt_refuses_more_than_its_limit_and_limit_0_cuts_short_like_laravel() {
        let long = "a".repeat(80);
        let like_laravel = settings("[bcrypt]\nlimit = 0\nrounds = 4\n").unwrap();

        let refused = Settings::default().make(&long).unwrap_err();
        let hashed = like_laravel.make(&long).unwrap();

        assert_eq!(refused.status(), 422);
        assert!(matches(&long, &hashed));
        assert!(matches(&"a".repeat(72), &hashed), "bcrypt reads 72 bytes");
    }
}
