use sabai_core::contracts::BoxFuture;
use sabai_core::{App, Result};
use sabai_orm::Model;
use tokio::sync::OnceCell;

use crate::Hash;

/// A model people sign in as, like Laravel's `Authenticatable` on the `User` model.
pub trait Authenticatable: Model + Sync {
    /// The column people sign in with.
    const LOGIN: &'static str = "email";

    /// The stored password hash.
    fn password(&self) -> &str;

    /// Replaces the stored hash, when it is upgraded on login.
    fn set_password(&mut self, hashed: String);

    /// The user whose login and password match, or `None`, without signing anyone in, like a Laravel
    /// user provider's `retrieveByCredentials` and `validateCredentials` together. A hash made with
    /// older settings is upgraded and saved, like `rehash_on_login`. Guards build on this (M3-04).
    fn validate_credentials(
        login: &str,
        password: &str,
    ) -> BoxFuture<'static, Result<Option<Self>>> {
        let found = Self::where_eq(Self::LOGIN, login).first();
        let password = password.to_owned();
        Box::pin(async move {
            let Some(mut user) = found.await? else {
                verify(password, None).await?;
                return Ok(None);
            };
            match verify(password, Some(user.password().to_owned())).await? {
                Verdict::Wrong => Ok(None),
                Verdict::Right => Ok(Some(user)),
                Verdict::RightWithNewHash(hashed) => {
                    user.set_password(hashed);
                    user.save().await?;
                    Ok(Some(user))
                }
            }
        })
    }
}

enum Verdict {
    Wrong,
    Right,
    RightWithNewHash(String),
}

// Not generic, so the hashing compiles once rather than for every model that signs in.
fn verify(password: String, stored: Option<String>) -> BoxFuture<'static, Result<Verdict>> {
    Box::pin(async move {
        let Some(stored) = stored else {
            spend_a_check(&password).await?;
            return Ok(Verdict::Wrong);
        };
        if !Hash::check(&password, &stored).await {
            return Ok(Verdict::Wrong);
        }
        if Hash::needs_rehash(&stored) && rehash_on_login()? {
            return Ok(Verdict::RightWithNewHash(Hash::make(&password).await?));
        }
        Ok(Verdict::Right)
    })
}

// Checking a password takes a hash's time, so an unknown login does the same work: otherwise how
// fast the answer comes tells an attacker which emails have accounts.
async fn spend_a_check(password: &str) -> Result<()> {
    static UNUSED_HASH: OnceCell<String> = OnceCell::const_new();
    let hashed = UNUSED_HASH
        .get_or_try_init(|| Hash::make("an account that does not exist"))
        .await?;
    Hash::check(password, hashed).await;
    Ok(())
}

fn rehash_on_login() -> Result<bool> {
    match App::current() {
        Ok(app) => app.config().get_or("hashing.rehash_on_login", true),
        Err(_) => Ok(true),
    }
}
