# Sabai Roadmap: Laravel-Parity To-Do

Oct 8, 2026 · @SCZ4

## How to use this list

Build in milestone order: each one ends with a feature a demo app can actually use, and each later milestone depends only on earlier ones. Built from scratch: no code carries over from oxalin, and every item is judged on developer experience and compile time first.

Every to-do has a ticket ID: `M0-01` to `M7-36` for milestones, plus `DX-`, `QA-`, `DOC-` and `REL-`. Use it in branch names and commits, e.g. branch `m1-03-named-routes`, commit `M1-03: named routes and url_for`.

| Milestone | Crate | Depends on | Done when |
| --- | --- | --- | --- |
| M0 Foundation | sabai-core, sabai-macros | none | `sabai new` app boots with config and logging |
| M1 HTTP & controllers | sabai-http | core | A resource controller serves JSON with validation |
| M2 ORM & migrations | sabai-orm | core, macros | `Post::find(1)` works and app + package migrations run |
| M3 Auth | sabai-auth | http, orm | Register, log in, hit a protected route |
| M4 Queue | sabai-queue | core, orm | A job dispatched from a controller runs on a worker |
| M5 Console kernel | sabai-cli | all | `sabai make:*`, `migrate`, `queue:work`, `schedule:run` |
| M6 Package ecosystem | sabai (public API) | all | `sabai-sanctum` and `sabai-media` ship using public APIs only |
| M7 Storage, cache, mail | sabai-storage, -cache, -mail, -notify | core | Upload an avatar to S3 and email a link to it |

Compile-time rules that apply to every item below:

- Derive macros emit metadata (consts, field lists) only; logic lives in runtime crates.
- Every public generic API is a one-line shim over a non-generic inner function.
- Pluggable parts (middleware, guards, queue drivers, DB drivers) are `Arc<dyn Trait>`, not nested generics.
- `sabai-macros` never depends on runtime crates; it emits `::sabai::...` paths.
- Integration tests live in one binary: `tests/it/main.rs`.

## Folder structure

Two trees: the framework repo, split into crates by external dependency so apps only compile what they use, and the app that `sabai new` generates, laid out like Laravel.

Framework repo (`sabai-rs/sabai`)

```text
sabai/
├── Cargo.toml                 # [workspace] + [workspace.dependencies]
├── .cargo/config.toml         # fast linker, dev profiles
├── crates/
│   ├── sabai/                 # facade: re-exports + features (orm, auth, queue, s3, full)
│   ├── sabai-core/            # Error, Request/Response, config, container,
│   │                          # contracts: Clock, Cache, Filesystem, Mailer
│   ├── sabai-macros/          # derives emit metadata only; no runtime deps
│   ├── sabai-http/            # router, handlers, extractors, middleware, validation, sessions
│   ├── sabai-orm/             # query builder, models, relations, schema builder, migrator
│   ├── sabai-auth/            # guards, hashing, gates, policies
│   ├── sabai-queue/           # jobs, drivers, worker
│   ├── sabai-cache/           # memory + Redis drivers, locks, rate limiter
│   ├── sabai-storage/         # Storage facade, local disk, uploads
│   ├── sabai-storage-s3/      # S3 driver: keeps the AWS SDK out of apps that skip it
│   ├── sabai-mail/            # mailables, SMTP / SES
│   ├── sabai-notify/          # notifications, channels
│   ├── sabai-cli/             # `sabai` binary: kernel, make:*, migrate, queue:work
│   └── sabai-testing/         # test client, fakes, DB assertions, factory helpers
├── packages/                  # first-party packages, public API only
│   ├── sabai-sanctum/
│   └── sabai-media/
├── templates/                 # `sabai new` skeletons: api, web, minimal
├── examples/todo-app/
├── docs/                      # getsabai.dev source
└── tests/it/main.rs           # one integration-test binary
```

Generated app (`sabai new blog`)

```text
blog/
├── Cargo.toml                 # workspace: app + database
├── .env
├── config/                    # app, database, auth, queue, cache, storage, mail (.toml)
├── src/
│   ├── main.rs                # boots App, registers providers
│   ├── routes/
│   │   ├── web.rs
│   │   └── api.rs
│   ├── http/
│   │   ├── controllers/
│   │   ├── middleware/
│   │   ├── requests/          # validated request structs
│   │   └── resources/         # API resources: model -> JSON
│   ├── models/
│   ├── policies/
│   ├── jobs/
│   ├── mail/
│   ├── notifications/
│   ├── events/
│   ├── listeners/
│   ├── console/
│   │   ├── commands/
│   │   └── schedule.rs
│   ├── database/
│   │   ├── factories/         # need models, so they live in the app crate
│   │   └── seeders/
│   └── providers/
├── database/                  # separate crate: migrations only, no models,
│   ├── Cargo.toml             # so editing a migration never rebuilds handlers
│   └── migrations/
├── resources/views/           # templates (web template only)
├── storage/
│   ├── app/public/            # public disk; `storage:link` -> public/storage
│   ├── framework/             # file cache, sessions
│   └── logs/
├── public/                    # static assets
└── tests/
    └── it/
        ├── main.rs            # one test binary, one link
        ├── feature/
        └── unit/
```

When an app grows, split `src/` into a `domain` crate (models, business logic) and a `web` crate (routes, handlers), so editing a handler never recompiles the domain.

## M0: Workspace & foundation

Goal: a workspace that compiles fast and an app skeleton every later milestone plugs into.

Workspace

- [x] **M0-01** New repo `sabai-rs/sabai`, fresh workspace (no code from oxalin)
- [x] **M0-02** Split into `crates/sabai-*` with `[workspace.dependencies]`
- [x] **M0-03** `sabai` facade crate with features: `orm`, `auth`, `queue`, `full`
- [x] **M0-04** `.cargo/config.toml`: fast linker (mold/lld), `debug = "line-tables-only"`, `build-override opt-level = 3`
- [ ] **M0-05** CI: `cargo build --timings` and `cargo llvm-lines` on PRs, fail when over the compile budgets below
  - [x] **M0-05a** CI reports: fmt, clippy, tests (default and all features), MSRV check, `--timings` artifact, `llvm-lines` summary
  - [ ] **M0-05b** Fail CI when over the compile budgets (needs the walking-skeleton app to measure)

Core (sabai-core)

- [x] **M0-06** `Error` type with HTTP status mapping
- [ ] **M0-07** `Request`, `Response`, `Body`, `IntoResponse`
- [ ] **M0-08** Config: layered `.env` + `config/*.toml`, typed access `config::<AppConfig>()`
- [ ] **M0-09** Logging via `tracing`, request-id span per request
- [ ] **M0-10** App state / service container: type-map of `Arc<dyn Any + Send + Sync>`
- [ ] **M0-11** Service providers: `trait Provider { fn register(&self, app: &mut App); fn boot(&self, app: &App); }`
- [ ] **M0-12** Events: `dispatch(Event)` with sync listeners (queued listeners come in M4)

* [ ] **M0-13** Contracts from day one: `Clock`, `Cache`, `Filesystem`, `Mailer` traits, so fakes, the scheduler and packages plug in

Macros (sabai-macros)

- [ ] **M0-14** Crate skeleton with minimal `syn` features
- [ ] **M0-15** `proc-macro-crate` lookup so macros work via facade or sub-crates

## M1: HTTP layer & controllers

Goal: Laravel-style routes, controllers and validation built fresh on hyper + matchit.

Router

- [ ] **M1-01** `get/post/put/patch/delete`, path params, `{*rest}` wildcard
- [ ] **M1-02** Groups with prefix, group middleware, `merge`
- [ ] **M1-03** Named routes and `url_for` with percent-encoding
- [ ] **M1-04** Per-route and per-group body limits
- [ ] **M1-05** Auto `HEAD`, `405` with `Allow` header, panic catching
- [ ] **M1-06** Route table for `route:list`
- [ ] **M1-07** Route-level middleware: `.get(...).middleware(x)`
- [ ] **M1-08** Fallback route (custom 404)
- [ ] **M1-09** Route model binding: `{post}` resolves to `Post` (after M2)

Router: compile-time rules

- [ ] **M1-10** `group`/`merge` run the closure, then call a non-generic `mount()`
- [ ] **M1-11** `route` boxes the handler, then calls a non-generic `push_route()`
- [ ] **M1-12** Middleware stored as `Arc<dyn Middleware>`, never nested generic layers
- [ ] **M1-13** `impl Handler` macro capped at 8 extractor args
- [ ] **M1-14** Extractor rejections turned into a `Response` by one non-generic helper

Controllers

- [ ] **M1-15** Handlers as methods: `.get("/posts", PostController::index)`
- [ ] **M1-16** `Router::resource("posts", PostController)` maps the 7 REST actions
- [ ] **M1-17** `api_resource` (no `create`/`edit`)
- [ ] **M1-18** Controller state via `State<T>` extractor

Middleware

- [ ] **M1-29** `Middleware` trait: code before and after `next.run(req)`, plus terminable middleware that runs after the response is sent
- [ ] **M1-30** `from_fn(fn)` for writing middleware as a plain async function
- [ ] **M1-31** Middleware aliases and groups (`web`, `api`) with a priority order, like Laravel's HTTP kernel
- [ ] **M1-32** Parameterised middleware: `throttle(60).per_minute()`, `auth("api")`, `can("update", "post")`
- [ ] **M1-33** Request ID and access log (tracing span per request)
- [ ] **M1-34** Timeout and compression (gzip, br)
- [ ] **M1-35** Security headers: HSTS, CSP, X-Frame-Options, Referrer-Policy
- [ ] **M1-36** Static files from `public/` with cache headers
- [ ] **M1-37** Signed-URL check middleware
- [ ] **M1-38** Trim strings and turn empty strings into `None`

Request & response

- [ ] **M1-19** Extractors: `Json<T>`, `Form<T>`, `Query<T>`, `Path<T>`, `Header`, `State<T>`
- [ ] **M1-20** `#[derive(FromRequest)]` for grouping extractors
- [ ] **M1-21** Responses: `json()`, `redirect()`, `view()` hook, `download()`
- [ ] **M1-22** Cookies (signed + encrypted)
- [ ] **M1-23** Sessions: driver trait, cookie and Redis drivers
- [ ] **M1-24** CSRF middleware for form routes
- [ ] **M1-25** CORS, trust-proxy, rate-limit middleware
- [ ] **M1-39** Request helpers: `input("name")` (query + body), `only`, `except`, `has`, `bearer_token()`, `ip()` behind trust-proxy, `wants_json()`, `route_is("posts.*")`
- [ ] **M1-40** `Extension<T>` for per-request data set by middleware
- [ ] **M1-41** Raw body (`Bytes`) and streaming request body
- [ ] **M1-42** Method spoofing (`_method=PUT`) for HTML forms
- [ ] **M1-43** Flash data and old input for redirect-back flows
- [ ] **M1-44** Locale from `Accept-Language`
- [ ] **M1-45** `IntoResponse` for `(StatusCode, headers, body)` tuples and `Result<T, E>`
- [ ] **M1-46** Status helpers: `created()` with `Location`, `no_content()`, `accepted()`
- [ ] **M1-47** Redirects: `back()`, `to_route("posts.show", params)`, `.with("status", "Saved")`
- [ ] **M1-48** Streaming responses, Server-Sent Events, `file()` inline with range requests
- [ ] **M1-49** Central exception handler: every error becomes one response shape (`{message, errors}`), debug page in dev only, no internals in production

Validation

- [ ] **M1-26** `#[derive(Validate)]` emitting rule metadata only
- [ ] **M1-27** Rules: `required`, `email`, `min`, `max`, `in`, `confirmed`, `unique` (after M2)
- [ ] **M1-28** Form-request style: `Valid<Json<T>>` returns `422` with field errors
- [ ] **M1-50** Fixed error shape: `{"message": ..., "errors": {"email": ["..."]}}`
- [ ] **M1-51** Nested and array fields: `items.*.qty`
- [ ] **M1-52** Conditional rules: `nullable`, `sometimes`, `required_if`, `required_with`, `bail`
- [ ] **M1-53** More rules: `regex`, `url`, `uuid`, `date`, `before`/`after`, `numeric`, `integer`, `between`, `same`, `exists`, `enum`
- [ ] **M1-54** File rules: `file`, `image`, `mimes`, max size (with M7-04)
- [ ] **M1-55** Custom rules: `impl Rule` (sync) and `impl AsyncRule` for DB checks
- [ ] **M1-56** Custom messages and attribute names, Thai and English (with M7-23)
- [ ] **M1-57** `authorize()` on request structs, like Laravel FormRequest: 403 before validation runs
- [ ] **M1-58** Web forms: failed validation redirects back with errors and old input; APIs get 422
- [ ] **M1-59** `Validator::make(data, rules)` for dynamic rules without a derive

HTTP caching

- [ ] **M1-60** `Cache-Control` helpers: `.cache_for(duration)`, `.no_store()`, `.private()`
- [ ] **M1-61** ETag / Last-Modified with automatic `304 Not Modified`
- [ ] **M1-62** Response cache middleware: stores whole GET responses in the cache store, keyed by route, query and user, with tags for invalidation
- [ ] **M1-63** Correct `Vary` headers (Accept, Accept-Language, Authorization)

## M2: ORM (Eloquent-style)

Goal: `Post::query().where("published", true).with("author").paginate(20)` against Postgres, MySQL and SQLite.

Connection

- [ ] **M2-01** Pool on top of `sqlx` (Postgres first, then MySQL, SQLite)
- [ ] **M2-02** Multiple named connections from config
- [ ] **M2-03** Transactions: `db.transaction(|tx| async { ... })`

Query builder

- [ ] **M2-04** `select`, `where`, `or_where`, `where_in`, `where_null`, `order_by`, `limit`, `offset`
- [ ] **M2-05** `join`, `group_by`, `having`, raw expressions with bindings
- [ ] **M2-06** `insert`, `update`, `delete`, `upsert`
- [ ] **M2-07** Aggregates: `count`, `sum`, `max`, `exists`
- [ ] **M2-08** Builder compiled by a non-generic `Grammar` per driver

Models

- [ ] **M2-09** `#[derive(Model)]` emits `TABLE`, `COLUMNS`, `values()`, `from_row()` only
- [ ] **M2-10** `find`, `find_or_fail` (404), `all`, `first`, `create`, `save`, `delete`
- [ ] **M2-11** Timestamps (`created_at`, `updated_at`) on by default
- [ ] **M2-12** Soft deletes via `#[model(soft_delete)]`
- [ ] **M2-13** Mass-assignment guard: `#[model(fillable(...))]`
- [ ] **M2-14** Casts: JSON, enum, `chrono` dates
- [ ] **M2-15** Hidden fields when serialized (e.g. `password`)

Relations

- [ ] **M2-16** `has_one`, `has_many`, `belongs_to`
- [ ] **M2-17** `belongs_to_many` with pivot table
- [ ] **M2-18** Eager loading `with("author")` without N+1

Migrations, schema & data

- [ ] **M2-19** Migrations as Rust files with `up`/`down`; raw `.sql` migrations also accepted
- [ ] **M2-20** Timestamped names, run in order across the app and every package
- [ ] **M2-21** `migrations` table with batch numbers, so rollback undoes the last batch
- [ ] **M2-22** Packages register migrations from their provider; `vendor:publish --tag=migrations` copies them into the app
- [ ] **M2-23** Schema builder: create/alter/drop table, indexes, foreign keys, `timestamps()`, `soft_deletes()`, `morphs()`
- [ ] **M2-24** Commands: `migrate`, `migrate:rollback --step`, `migrate:reset`, `migrate:fresh`, `migrate:status`, `--pretend` (print SQL only)
- [ ] **M2-25** One transaction per migration where the driver supports transactional DDL
- [ ] **M2-26** Advisory lock so two deploys never migrate at once
- [ ] **M2-27** `schema:dump` to squash old migrations into one SQL file
- [ ] **M2-28** Migrations live in the app's own `database` crate, so editing them never rebuilds handlers
- [ ] **M2-29** Seeders and factories (`Post::factory().count(10).create()`)
- [ ] **M2-30** Pagination: `paginate(n)` and `cursor_paginate(n)` with JSON shape

## M3: Auth

Goal: session login for web routes and token auth for API routes, both behind one `Auth` extractor.

Core

- [ ] **M3-01** `trait Authenticatable` (id, password hash) implemented by the `User` model
- [ ] **M3-02** `trait UserProvider` with an ORM-backed default
- [ ] **M3-03** Password hashing with Argon2id, `Hash::make` / `Hash::check`, rehash on login
- [ ] **M3-04** Guards as `Arc<dyn Guard>`: `session` and `token`, picked per route group
- [ ] **M3-05** Extractors: `Auth<User>` (required, else 401/redirect) and `OptionalAuth<User>`
- [ ] **M3-06** Middleware: `auth`, `auth("api")`, `guest`

Web (session) flow

- [ ] **M3-07** Login, logout, "remember me" cookie
- [ ] **M3-08** Session regeneration on login (fixation defence)
- [ ] **M3-09** Login throttling: lock after N failures per email + IP
- [ ] **M3-10** Password reset: signed, expiring token by email
- [ ] **M3-11** Email verification with signed URLs

API (token) flow

- [ ] **M3-12** Personal access tokens, stored hashed, shipped as the `sabai-sanctum` package (see M6)
- [ ] **M3-13** Token abilities / scopes: `token.can("posts:write")`
- [ ] **M3-14** Token expiry and revoke

Authorization

- [ ] **M3-15** Gates: `Gate::define("edit-post", |user, post| ...)`
- [ ] **M3-16** Policies per model: `PostPolicy::update(&user, &post)`
- [ ] **M3-17** `authorize!` helper returning 403

Scaffolding

- [ ] **M3-18** `sabai make:auth` generates controllers, routes, migrations

## M4: Queue

Goal: `SendWelcomeEmail { user_id }.dispatch()` from a controller, run by `sabai queue:work` with retries.

Jobs

- [ ] **M4-01** `trait Job: Serialize + DeserializeOwned` with `async fn handle(&self, ctx)`
- [ ] **M4-02** `#[derive(Job)]` registers a name for deserialization (metadata only)
- [ ] **M4-03** Options: `queue`, `delay`, `tries`, `backoff`, `timeout`
- [ ] **M4-04** Dispatch: `dispatch()`, `dispatch_after(dur)`, `dispatch_sync()`

Drivers (`Arc<dyn QueueDriver>`)

- [ ] **M4-05** `sync` (runs inline, for tests)
- [ ] **M4-06** `database` (uses ORM, `SELECT ... FOR UPDATE SKIP LOCKED`)
- [ ] **M4-07** `redis` (lists + sorted set for delayed jobs)
- [ ] **M4-08** SQS (optional, later)

Worker

- [ ] **M4-09** `queue:work` with concurrency, queue priority list, graceful shutdown on SIGTERM
- [ ] **M4-10** Retries with exponential backoff, then `failed_jobs` table
- [ ] **M4-11** `queue:failed`, `queue:retry {id}`, `queue:flush`
- [ ] **M4-12** Job middleware: rate limit, without-overlapping

Beyond basics

- [ ] **M4-13** Chains: run B after A succeeds
- [ ] **M4-14** Batches with progress and `then`/`catch` callbacks
- [ ] **M4-15** Queued event listeners (from M0 events)
- [ ] **M4-16** Unique jobs (lock by key)
- [ ] **M4-17** Metrics: jobs processed, failed, wait time (tracing / Prometheus)

## M5: Console kernel & commands (Artisan-style)

Goal: one `sabai` binary for generators and framework tasks, plus user-defined commands and a scheduler.

Kernel

- [ ] **M5-01** `sabai-cli` on `clap`, installed as `cargo install sabai-cli`
- [ ] **M5-02** Two modes: global (`sabai new`) and in-app (delegates to the app's own binary so app commands see app code)
- [ ] **M5-03** `trait Command` with `signature`, `description`, `async fn handle(&self, io)`
- [ ] **M5-04** `#[derive(Command)]` maps struct fields to args/options
- [ ] **M5-05** Console IO helpers: `info`, `warn`, `error`, `table`, `confirm`, `ask`, progress bar
- [ ] **M5-06** Commands registered in a `ConsoleKernel` like the HTTP router

Built-in commands

- [ ] **M5-07** `route:list`
- [ ] **M5-08** `new <app>`, `serve`, `about`, `env`
- [ ] **M5-09** `make:controller`, `make:model -m`, `make:migration`, `make:job`, `make:command`, `make:policy`, `make:middleware`
- [ ] **M5-10** `migrate`, `migrate:rollback`, `migrate:fresh --seed`, `db:seed`
- [ ] **M5-11** `queue:work`, `queue:failed`, `queue:retry`
- [ ] **M5-12** `key:generate`, `config:show`

Scheduler

- [ ] **M5-13** `schedule.command("reports:send").daily_at("08:00")`, `.every_five_minutes()`, cron syntax
- [ ] **M5-14** `schedule:run` (one cron entry runs it every minute) and `schedule:work` (long-running)
- [ ] **M5-15** `without_overlapping()` and `on_one_server()` via cache lock
- [ ] **M5-16** Schedule jobs and closures, not only commands

## M6: Package ecosystem (Sanctum, MediaLibrary-style libs)

Goal: anyone can publish a crate like `sabai-sanctum` or `sabai-medialibrary` that installs with `cargo add` plus one line, using only public APIs. The extension points below are designed in from M0, not bolted on at the end.

Package API

- [ ] **M6-01** `ServiceProvider` (`register` / `boot`) is the single entry point for every package
- [ ] **M6-02** One-line install: `App::new().provider(SanctumProvider)`; optional auto-register behind a feature, since it costs compile time
- [ ] **M6-03** Providers can add routes, middleware, config defaults, migrations, commands, listeners, jobs, guards and extractors
- [ ] **M6-04** `vendor:publish --provider=<name> --tag=config|migrations|views` copies package files into the app
- [ ] **M6-05** Config merge: package defaults, overridden by the app's `config/<package>.toml`

Model extension (what a MediaLibrary needs)

- [ ] **M6-06** Model traits from other crates: `impl HasMedia for Post` gives `post.add_media(file)`
- [ ] **M6-07** Polymorphic relations: `morph_to`, `morph_many` (media, comments, tags all need them)
- [ ] **M6-08** Model events / observers: `creating`, `saved`, `deleted` hooks packages can subscribe to
- [ ] **M6-09** Filesystem layer: `Storage::disk("s3")` (built in M7)

Auth extension (what a Sanctum needs)

- [ ] **M6-10** Custom guards registered by name: `auth.extend("sanctum", ...)`
- [ ] **M6-11** `FromRequest` public and stable, so packages ship their own extractors
- [ ] **M6-12** Package commands show up in `sabai list`

Stability & tooling

- [ ] **M6-13** `sabai::contracts` module holds every trait packages implement, covered by semver
- [ ] **M6-14** `sabai make:package <name>` scaffolds provider, config, migration and a test
- [ ] **M6-15** `sabai-testing` boots a test app with just one provider, for package tests
- [ ] **M6-16** Dogfood: ship `sabai-sanctum` and `sabai-media` as separate crates, public APIs only
- [ ] **M6-17** Package directory page on getsabai.dev

## M7: Storage, cache, mail & app services

Goal: the services every real app needs, each a contract in `sabai-core` with a fake for tests, so packages and test helpers plug in without changes.

Storage (sabai-storage)

- [ ] **M7-01** `Storage::disk("local" | "public" | "s3")` configured in `config/storage.toml`
- [ ] **M7-02** `put`, `get`, `exists`, `delete`, `copy`, `move`, `size`, `last_modified`, list files and directories
- [ ] **M7-03** Streaming reads and writes for large files, never loaded fully into memory
- [ ] **M7-04** `UploadedFile` extractor (multipart) with `store("avatars")` and size / MIME validation
- [ ] **M7-05** Public URLs and expiring signed URLs (S3 presigned, local via signed route)
- [ ] **M7-06** Per-file visibility: public or private
- [ ] **M7-07** `storage:link` command: public disk to `public/storage`
- [ ] **M7-08** S3-compatible driver (AWS, Cloudflare R2, MinIO) in the separate `sabai-storage-s3` crate
- [ ] **M7-09** `Storage::fake("s3")` with `assert_exists` / `assert_missing`

Cache (sabai-cache)

- [ ] **M7-10** `get`, `put`, `remember`, `forget`, TTL, tags (Redis)
- [ ] **M7-11** Drivers: memory, Redis, database
- [ ] **M7-12** Atomic locks, used by the scheduler and unique jobs
- [ ] **M7-13** Rate limiter on top of cache (login throttle, API limits)
- [ ] **M7-29** `add`, `pull`, `increment` / `decrement`, `many` / `put_many`, `remember_forever`
- [ ] **M7-30** `flexible` (stale-while-revalidate): serve the stale value, refresh in the background
- [ ] **M7-31** Multiple stores with key prefixes: `Cache::store("redis")`, plus file and null drivers
- [ ] **M7-32** Per-request memo cache, so one request never hits Redis twice for the same key
- [ ] **M7-33** Typed values via serde over one non-generic byte-store API (generic shim only for (de)serialize)
- [ ] **M7-34** Commands: `cache:clear`, `cache:forget <key>`
- [ ] **M7-35** `Cache::fake()` and cache events (hit, miss, write) for tests and the debug dashboard
- [ ] **M7-36** ORM query cache: `Post::query().remember(60)` with tag invalidation on save

Mail & notifications (sabai-mail, sabai-notify)

- [ ] **M7-14** Mailables with templates, attachments from Storage, queued sending
- [ ] **M7-15** Drivers: SMTP, SES, log (dev)
- [ ] **M7-16** Mail preview route in dev
- [ ] **M7-17** Notifications: one object, many channels (mail, database, Slack, LINE)
- [ ] **M7-18** Database notifications table with read / unread

App services

- [ ] **M7-19** API resources: `PostResource` turns models into JSON, with collections and pagination meta
- [ ] **M7-20** Views: choose engine (Askama compile-time vs MiniJinja runtime), measure compile cost first
- [ ] **M7-21** HTTP client on `reqwest` with `Http::fake()`
- [ ] **M7-22** Encryption (`Crypt`) and expiring signed URLs (`URL::signed_route`)
- [ ] **M7-23** Localization: translation files, validation messages in Thai and English

Later

- [ ] **M7-24** Broadcasting / WebSockets
- [ ] **M7-25** Maintenance mode: `sabai down` / `sabai up`
- [ ] **M7-26** Health check endpoint
- [ ] **M7-27** Debug dashboard (Telescope-style): requests, queries, jobs
- [ ] **M7-28** Multiple log channels

## DX & compile-speed targets

Two numbers gate every release: rerun after editing a handler in under 2 s, and a clean build of a new app in under 30 s.

| Metric | Target | Checked by |
| --- | --- | --- |
| Dev rebuild after editing one handler | < 2 s | `cargo build --timings` in CI |
| Clean dev build of `sabai new` app | < 30 s | CI, 4-core runner |
| Clean release build of `sabai new` app | < 90 s | CI, 4-core runner |
| LLVM lines added per route | < 50 | `cargo llvm-lines` diff on PR |
| `sabai new` to first response | < 60 s | Manual check each release |

Developer experience

- [ ] **DX-01** Macro errors point at the user's code with a span and a fix hint
- [ ] **DX-02** `#[sabai::debug_handler]` explains why a function is not a valid handler
- [ ] **DX-03** `sabai dev`: watch, rebuild, restart on the same port
- [ ] **DX-04** Dev error page: request, matched route, stack trace, SQL queries run
- [ ] **DX-05** `sabai new` templates: `api`, `web`, `minimal`
- [ ] **DX-06** One import: `use sabai::prelude::*;`
- [ ] **DX-07** Works with zero config: SQLite, sync queue, cookie sessions
- [ ] **DX-08** Every docs example compiles in CI (doctests)

## Cross-cutting: testing, docs, release

Testing (sabai-testing)

- [ ] **QA-01** In-process test client: `app.get("/posts").await.assert_ok().assert_json(...)`
- [ ] **QA-02** `acting_as(&user)` for authenticated tests
- [ ] **QA-03** `RefreshDatabase`: migrate once per run, one rolled-back transaction per test
- [ ] **QA-04** DB assertions: `assert_database_has`, `assert_database_missing`, `assert_soft_deleted`, `assert_model_exists`
- [ ] **QA-05** Factories: states (`.published()`), sequences, relations (`has`, `for`), `make()` without saving
- [ ] **QA-06** Fakes with assertions: Queue, Mail, Event, Notification, Storage, Http
- [ ] **QA-07** Time travel: `travel_to(date)`, `freeze_time()` (built on the `Clock` contract)
- [ ] **QA-08** Parallel tests with one database per worker
- [ ] **QA-09** `sabai test` wrapper over `cargo nextest`

Docs & example

- [ ] **DOC-01** `examples/todo-app` exercising every milestone
- [ ] **DOC-02** getsabai.dev docs site: install, routing, controllers, ORM, auth, queue, CLI
- [ ] **DOC-03** "Coming from Laravel" page mapping each Laravel concept to Sabai

Release

- [ ] **REL-01** Reserve names: crates `sabai`, `sabai-*`; GitHub org `sabai-rs`; domain `getsabai.dev`
- [ ] **REL-02** Publish `0.1.0` after M1 (routing + controllers usable on their own)
- [ ] **REL-03** Changelog and semver policy; MSRV pinned in CI
