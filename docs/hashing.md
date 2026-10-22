# Hashing

`Hash` stores passwords like Laravel: bcrypt with 12 rounds by default, or Argon2id. Apps from `sabai new` have it already, with `config/hashing.toml`; otherwise turn on the `auth` feature:

```toml
sabai = { version = "...", features = ["orm", "auth"] }
```

Argon2id has its own feature, so apps on bcrypt, like most Laravel apps, do not compile it:

```toml
sabai = { version = "...", features = ["orm", "auth", "argon2"] }
```

Without it, `driver = "argon2id"` or an Argon2id hash stops with an error that names the feature.

## Configuration

`config/hashing.toml` has the keys and defaults of Laravel's `config/hashing.php`, so an app can share its `users` table with a Laravel app:

```toml
driver = "${HASH_DRIVER:-bcrypt}"     # or "argon2id"

[bcrypt]
rounds = "${BCRYPT_ROUNDS:-12}"
verify = true
limit = 72                            # bytes; 0 for no limit

[argon]
memory = 65536                        # KiB
time = 4
threads = 1
verify = true
```

A missing file or key uses these defaults. Check what a Laravel app uses with `php artisan config:show hashing`.

## Hashing Passwords

```rust
use sabai::auth::Hash;

let hashed = Hash::make(&request.password).await?;
```

The result is `$2y$12$...` with bcrypt, the format PHP's `password_hash` writes, so Laravel checks it and Sabai checks Laravel's. Hashing the same password twice gives two different strings.

bcrypt reads only the first 72 bytes of a password, so two long passwords that start the same would hash the same. Instead of cutting a longer one short, `make` answers `422 The password must not be longer than 72 bytes.` A Thai letter is three bytes, so that is 24 Thai letters. Check it in the request first with the [`max_bytes(72)`](/validation) rule, so the error lands on the `password` field.

::: laravel
Laravel has the same `limit` key, but off by default: PHP's `password_hash` cuts a longer password short without saying so. Set `limit = 0` for that behaviour.
:::

## Checking a Password

```rust
if !Hash::check(&request.password, &user.password).await {
    return Err(Error::with_messages(&[("email", "These credentials do not match our records.")]));
}
```

A malformed hash never matches. With `verify` on, like Laravel, neither does another driver's: a bcrypt app refuses an Argon2id hash and logs a warning. Turn `verify` off while moving users from one driver to the other.

## Rehashing

When the driver or its settings change, old hashes still check. After a successful login, hash the password again so it uses today's settings:

```rust
if Hash::needs_rehash(&user.password) {
    user.password = Hash::make(&request.password).await?;
    user.save().await?;
}
```

A Laravel hash made with 10 rounds, Laravel's default before version 11, checks and is upgraded to 12 this way.

::: warning Sharing a database with Laravel
Use the Laravel app's driver and settings. If Sabai saved an Argon2id hash while Laravel uses bcrypt with `verify` on, that user could no longer log in to Laravel.
:::

## Speed

| Driver | One hash, optimized |
| --- | --- |
| bcrypt, 12 rounds | about 260 ms |
| Argon2id, 64 MiB, t=4 | about 120 ms |

At most one hash per CPU core runs at a time. A flood of login requests waits in line instead of filling the server's memory and CPU, and other requests are not slowed down.

::: laravel
`Hash::make`, `Hash::check` and `Hash::needsRehash`. They are `async` in Sabai: hashing is slow on purpose, and Sabai runs it on a separate thread so other requests do not wait. PHP gives every request its own process, so Laravel can block.
:::

Unoptimized, hashing is about ten times slower, so add this to your app's `Cargo.toml` to keep `sabai serve` and tests fast in development:

```toml
[profile.dev.package.blowfish]
opt-level = 3

# With the `argon2` feature:
[profile.dev.package.argon2]
opt-level = 3
```
