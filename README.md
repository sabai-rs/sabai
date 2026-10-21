<p align="center">
  <picture>
    <source media="(prefers-color-scheme: dark)" srcset="docs/public/logo-dark.svg">
    <img src="docs/public/logo.svg" width="96" alt="Sabai logo">
  </picture>
</p>

<h1 align="center">Sabai</h1>

<p align="center"><strong>Rust, but sabai.</strong></p>

<p align="center">
<a href="https://github.com/sabai-rs/sabai/actions/workflows/ci.yml"><img src="https://github.com/sabai-rs/sabai/actions/workflows/ci.yml/badge.svg" alt="Build Status"></a>
<a href="LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue" alt="License"></a>
</p>

> Sabai is in early development and not yet published to crates.io. APIs change from one commit to the next.

## About Sabai

Sabai is a web application framework for Rust, built to feel like Laravel. If you know Laravel, you already know where things go and what they are called. Sabai keeps the parts that make Laravel enjoyable, and adds what Rust gives for free: a single fast binary, and mistakes caught when the app starts instead of in production.

Three things guide every decision, in this order:

1. **Developer experience.** APIs read like Laravel, and errors say how to fix them.
2. **Compile speed.** Every change is measured for its effect on build time.
3. **Correctness and security.**

What works today:

- [Routing](https://sabai-rs.github.io/sabai/routing) with groups, named routes and resource routes.
- [Middleware](https://sabai-rs.github.io/sabai/middleware), with Laravel's defaults and request IDs in every log line.
- [Controllers](https://sabai-rs.github.io/sabai/controllers) and a [service container](https://sabai-rs.github.io/sabai/container) with constructor and method injection.
- [Form requests](https://sabai-rs.github.io/sabai/requests) and [validation](https://sabai-rs.github.io/sabai/validation) with Laravel's rules, messages and `422` shape.
- [Migrations](https://sabai-rs.github.io/sabai/migrations) that write Laravel's SQL, and [Eloquent-style models](https://sabai-rs.github.io/sabai/eloquent) on MySQL or SQLite.
- [HTTP tests](https://sabai-rs.github.io/sabai/http-tests): `app().get_json(route("posts.show").with("id", 1)).assert_status(200)`.
- The [`sabai` command](https://sabai-rs.github.io/sabai/console), like `laravel new` and `php artisan`.

Coming next: auth, queues, storage, cache and mail. The [roadmap](ROADMAP.md) lists every feature and what is done.

## Getting Started

```bash
cargo install --git https://github.com/sabai-rs/sabai sabai-cli
sabai new my-app
cd my-app
sabai migrate
sabai serve
```

See [Installation](https://sabai-rs.github.io/sabai/installation) for the details.

## Learning Sabai

The [documentation](https://sabai-rs.github.io/sabai/) covers every feature, with the Laravel equivalent beside each one.

The [example blog](examples/blog) uses every feature as it lands, laid out like a Laravel app. It is the best place to see how the pieces fit together.

## Contributing

Thank you for considering contributing to Sabai! The [contribution guide](CONTRIBUTING.md) explains how work is organised.

## Security Vulnerabilities

If you discover a security vulnerability within Sabai, please report it privately through [GitHub security advisories](https://github.com/sabai-rs/sabai/security/advisories/new) instead of a public issue.

## License

Sabai is open-sourced software licensed under the [MIT license](LICENSE).
