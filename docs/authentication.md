# Authentication

Sabai checks who someone is with their email and password. Sessions, tokens and the `auth` middleware are coming next.

## The User Model

A model people sign in as implements `Authenticatable`. Apps from `sabai new` already have it:

```rust
use sabai::auth::Authenticatable;

impl Authenticatable for User {
    fn password(&self) -> &str {
        &self.password
    }

    fn set_password(&mut self, hashed: String) {
        self.password = hashed;
    }
}
```

People sign in with the `email` column. Another one, such as `username`, is one line: `const LOGIN: &'static str = "username";`.

Any model people sign in as can implement it, such as `Staff` or `Client`, each with its own `LOGIN`: `Staff::validate_credentials(...)`.

## Checking a Login

`validate_credentials` finds the user by their login and checks the password against its [hash](/hashing), without signing them in:

```rust
pub async fn login(request: LoginRequest) -> Result<Json<User>> {
    match User::validate_credentials(&request.email, &request.password).await? {
        Some(user) => Ok(Json(user)),
        None => Err(Error::with_messages(&[(
            "email",
            "These credentials do not match our records.",
        )])),
    }
}
```

Answer an unknown email and a wrong password with the same message, so neither says which it was.

::: laravel
`User::validate_credentials` is what a Laravel user provider does with `retrieveByCredentials` and `validateCredentials`: it checks without signing in. `Auth::attempt`, which checks and signs in through a guard, comes with guards. The blog's `POST /api/login` is `AuthController::login`, Breeze's `AuthenticatedSessionController@store` for an API.
:::

## An Unknown Email Takes as Long

Checking a password takes a hash's time, about a quarter of a second with bcrypt. If an unknown email skipped it, the answer would come back almost at once, and anyone could tell which emails have accounts. `validate_credentials` checks the password against an unused hash instead, so every answer takes as long:

| Login | Answer |
| --- | --- |
| Wrong password | about 240 ms |
| Unknown email | about 240 ms |
| Right password | about 240 ms |

::: laravel
Laravel's `EloquentUserProvider` returns at once when the email is unknown.
:::

## Rehashing on Login

When a user signs in with a hash made by older settings, such as 10 bcrypt rounds, `validate_credentials` hashes the password again with today's and saves the user. Turn it off in `config/hashing.toml`:

```toml
rehash_on_login = false
```
