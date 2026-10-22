#[cfg(feature = "argon2")]
mod argon;
// Most apps hash with bcrypt, like Laravel, so Argon2's crates compile only for apps that ask.
#[cfg(not(feature = "argon2"))]
#[path = "hash/argon_disabled.rs"]
mod argon;
mod bcrypt;
mod settings;

use std::fmt;
use std::sync::OnceLock;
use std::thread;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, Result, log};
use tokio::sync::Semaphore;

use settings::Settings;

/// Password hashing like Laravel's `Hash` facade, set by `config/hashing.toml` with Laravel's
/// defaults: bcrypt with 12 rounds, or Argon2id.
///
/// Hashing is slow on purpose, so `make` and `check` run on a blocking thread and are awaited:
/// on a server thread they would stall every request sharing it. At most one hash per CPU core
/// runs at once; a flood of logins waits in line instead of taking memory and CPU from the rest.
pub struct Hash;

impl Hash {
    /// Hashes `password` with a random salt, like `Hash::make($password)`: `$2y$12$...` with
    /// bcrypt, the format PHP's `password_hash` writes.
    pub fn make(password: &str) -> BoxFuture<'static, Result<String>> {
        let password = password.to_owned();
        Box::pin(async move {
            let settings = Settings::current()?;
            on_blocking_thread(move || settings.make(&password)).await?
        })
    }

    /// Whether `password` matches `hashed`, like `Hash::check($password, $hashed)`. A malformed
    /// hash never matches, and neither does another driver's while its `verify` is on.
    pub fn check(password: &str, hashed: &str) -> BoxFuture<'static, bool> {
        let (password, hashed) = (password.to_owned(), hashed.to_owned());
        Box::pin(async move {
            let accepted = Settings::current().map(|settings| settings.accepts(&hashed));
            match accepted {
                Ok(true) => on_blocking_thread(move || settings::matches(&password, &hashed))
                    .await
                    .unwrap_or(false),
                Ok(false) => false,
                Err(error) => {
                    log::error!("{error}");
                    false
                }
            }
        })
    }

    /// Whether `hashed` was made with another driver or other settings than today's, like
    /// `Hash::needsRehash`: after a successful login, hash the password again and save it.
    pub fn needs_rehash(hashed: &str) -> bool {
        Settings::current().is_ok_and(|settings| settings.needs_rehash(hashed))
    }
}

// More hashes at once than cores would not finish sooner, only hold more memory.
async fn on_blocking_thread<T: Send + 'static>(
    job: impl FnOnce() -> T + Send + 'static,
) -> Result<T> {
    static PERMITS: OnceLock<Semaphore> = OnceLock::new();
    let cores = thread::available_parallelism().map_or(1, usize::from);
    let permits = PERMITS.get_or_init(|| Semaphore::new(cores));
    let _permit = permits.acquire().await.map_err(Error::internal)?;
    tokio::task::spawn_blocking(job)
        .await
        .map_err(Error::internal)
}

#[derive(Debug)]
struct HashError(String);

impl fmt::Display for HashError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "could not hash the password: {}", self.0)
    }
}

impl std::error::Error for HashError {}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

    use sabai_core::block_on;

    use super::*;

    // From PHP's manual for `password_verify`: a hash PHP itself made.
    const PHP_HASH: &str = "$2y$10$.vGA1O9wmRjrwAVXD98HNOgsNpDczlqm3Jq7KnEd1rVAGv3Fykk1a";

    fn make(password: &str) -> Result<String> {
        block_on(Hash::make(password)).unwrap()
    }

    fn check(password: &str, hashed: &str) -> bool {
        block_on(Hash::check(password, hashed)).unwrap()
    }

    #[test]
    fn by_default_passwords_hash_like_laravel_with_bcrypt_and_12_rounds() {
        let hashed = make("secret").unwrap();

        assert!(hashed.starts_with("$2y$12$"), "{hashed}");
        assert!(check("secret", &hashed));
        assert!(!check("Secret", &hashed));
        assert!(!Hash::needs_rehash(&hashed));
    }

    #[test]
    fn a_hash_php_made_checks_and_its_10_rounds_are_upgraded() {
        assert!(check("rasmuslerdorf", PHP_HASH));
        assert!(!check("rasmus", PHP_HASH));
        assert!(Hash::needs_rehash(PHP_HASH));
    }

    #[test]
    fn the_same_password_hashes_differently_each_time() {
        assert_ne!(make("secret").unwrap(), make("secret").unwrap());
    }

    #[test]
    fn a_password_longer_than_bcrypt_reads_is_refused_not_cut_short() {
        let error = make(&"a".repeat(73)).unwrap_err();

        assert_eq!(error.status(), 422);
        assert_eq!(
            error.to_string(),
            "The password must not be longer than 72 bytes."
        );
    }

    #[test]
    fn a_malformed_hash_never_matches_and_needs_a_rehash() {
        assert!(!check("secret", "not a hash"));
        assert!(!check("secret", ""));
        assert!(Hash::needs_rehash("not a hash"));
    }

    #[test]
    fn no_more_hashes_run_at_once_than_there_are_cores() {
        let cores = thread::available_parallelism().map_or(1, usize::from);
        let running = Arc::new(AtomicUsize::new(0));
        let most = Arc::new(AtomicUsize::new(0));

        block_on(async {
            let jobs: Vec<_> = (0..cores * 3)
                .map(|_| {
                    let (running, most) = (Arc::clone(&running), Arc::clone(&most));
                    tokio::spawn(on_blocking_thread(move || {
                        let now = running.fetch_add(1, Ordering::SeqCst) + 1;
                        most.fetch_max(now, Ordering::SeqCst);
                        thread::sleep(Duration::from_millis(20));
                        running.fetch_sub(1, Ordering::SeqCst);
                    }))
                })
                .collect();
            for job in jobs {
                job.await.unwrap().unwrap();
            }
        })
        .unwrap();

        let most = most.load(Ordering::SeqCst);
        assert!(
            (1..=cores).contains(&most),
            "{most} ran at once on {cores} cores"
        );
    }
}
