# Sabai Roadmap: Laravel-Parity To-Do

Oct 8, 2026 · @SCZ4

## How to use this list

Build in milestone order: each one ends with a feature a demo app can actually use, and each later milestone depends only on earlier ones. Built from scratch: no code carries over from oxalin, and every item is judged on developer experience and compile time first.

Every to-do has a ticket ID: `M0-01` to `M7-36` for milestones, plus `DX-`, `QA-`, `DOC-` and `REL-`. Use it in branch names and commits, e.g. branch `m1-03-named-routes`, commit `feat(M1-03a): named routes and route()`.

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
- A public `async fn` that other crates await returns a `BoxFuture` made in its own crate; otherwise its body, and everything it awaits, compiles again in every app (a plain `async fn ping` put 7,000 lines of sqlx into the blog).
- `sabai-macros` never depends on runtime crates; it emits `::sabai::...` paths.
- Integration tests live in one binary: `tests/it/main.rs`.

## Now: walking skeleton

`CLAUDE.md` asks for an end-to-end slice before deepening any milestone: `sabai new blog` -> `sabai migrate` -> `sabai serve` -> `GET /posts` returns JSON from the database. M1 went deep first; these come next, in this order, and anything else waits unless it blocks one of them.

1. ~~**M1-16 / M1-17**~~ `api_resource` done (M1-17); M1-16's web `create`/`edit` waits for views
2. ~~**M2-01**~~ A MySQL and SQLite pool from config (M2-01a)
3. ~~**M2-19 / M2-21 / M2-23 / M2-24**~~ Migrations with a schema builder and `migrate` (M2-21a, M2-23a, M2-24a)
4. ~~**M2-09 / M2-10**~~ (smallest part) `#[derive(Model)]` metadata and `all`, `find_or_fail`, `create`
5. ~~The blog's `PostRepository` on the database instead of in memory~~, with feature tests on a test database (QA-03, smallest part)
6. ~~**M5-01 / M5-02 / M5-08 / M5-10**~~ (smallest part) `sabai new`, `sabai serve`, `sabai migrate` (M5-01a, M5-06a, M5-08a to M5-08c)
7. ~~**M0-05b**~~ Compile budgets in CI, measured on the skeleton

After the skeleton: middleware (M1-29a, M1-02b, M1-31a, M1-33, done), then array rules (M1-51), and the rest of each milestone in order. `authorize()` (M1-57) waits for `Auth<User>` (M3-05), so the blog can use it for a real ownership check.

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
│   ├── sabai-http/            # router, handlers and their arguments, middleware, validation, sessions
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
├── examples/blog/
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
- [x] **M0-04** Dev profile in `Cargo.toml`: `debug = "line-tables-only"`; no forced linker (ld-prime / rust-lld are already fast). `build-override opt-level = 3` was measured and dropped: +3 s clean build, no gain on rebuilds even with 200 serde derives; re-measured at M0-14 with minimal `syn`: still +1.8 s
- [ ] **M0-05** CI: `cargo build --timings` and `cargo llvm-lines` on PRs, fail when over the compile budgets below
  - [x] **M0-05a** CI reports: fmt, clippy, tests (default and all features), MSRV check, `--timings` artifact, `llvm-lines` summary
  - [x] **M0-05b** Fail CI when a crate's LLVM lines go over its budget in `compile-budgets.toml` (measured plus about 10%). Build time stays a report only: GitHub's runners vary too much to fail on it
  - [x] **M0-05c** CI runs `sabai new` against the commit's own crates, then `sabai migrate`, fmt, clippy and tests in the new app, so the template always compiles. No budget of its own: the blog uses everything the new app does

Core (sabai-core)

- [x] **M0-06** `Error` type with HTTP status mapping
  - [x] **M0-06b** `Error::context("...")` adds what was being done to an error (`provider app::MailProvider failed to register: mail.host is not set`); `AppBuilder::boot` uses it to name the failing provider
  - [x] **M0-06c** Readable errors from `main`: `Debug` prints like `Display` (`{:#?}` keeps the fields); a port already in use says how to fix it
- [x] **M0-07** `Request`, `Response`, `Body`, `IntoResponse`
- [x] **M0-08** Config: layered `.env` + `config/*.toml`, typed access `config.section::<AppConfig>()` via `ConfigSection` (app-wide access through the container in M0-10 and `request.app()?.config()` in handlers since M1-19c, not a global)
- [x] **M0-09** Logging via `tracing`; the subscriber accepts extra layers registered by providers (e.g. Sentry). The request-id span moved to M1-33, where the server exists to use it
- [x] **M0-10** App state / service container: type-map of `Arc<dyn Any + Send + Sync>`
- [x] **M0-11** Service providers: `trait Provider { fn register(&self, app: &mut App) -> Result<()>; fn boot(&self, app: &App) -> Result<()>; }`; `App::builder(config).provider(..).boot()` returns a read-only `Arc<App>`
- [x] **M0-12** Events: `app.dispatch(&event)` with sync listeners (closures or `Listener<E>` structs), registered with `app.listen(..)` in providers; queued listeners come in M4-15

* [x] **M0-13** Contracts from day one in `sabai::contracts`: `Clock`, `CacheStore`, `StorageDriver`, `MailTransport` (the driver layer, like Laravel's `Store`, Flysystem adapters and Symfony transports), so fakes, the scheduler and packages plug in. The app-facing `Cache`, `Filesystem` and `Mailer` with Laravel's method names are built on them in M7

Macros (sabai-macros)

- [x] **M0-14** Crate skeleton with minimal `syn` features (`derive`, `parsing`, `printing`, `proc-macro`); first derive: `ConfigSection` (`MailConfig` reads `config/mail.toml`, or `#[config("mail")]`)
- [x] **M0-15** `proc-macro-crate` lookup so macros work via facade or sub-crates: `::sabai`, a renamed `sabai`, `::sabai_core`, or `crate` inside `sabai-core`
- [x] **M0-16** `sabai-macros` on syn 3, the version `serde_derive` uses, so apps compile syn once instead of twice (~1s of a clean build)
- [x] **M0-17** The blog laid out like a Laravel app: `src/app/http/{controllers,requests}`, `src/app/{models,providers,services,repositories}`, `src/routes/{web,api}.rs`; the framework never reads folders, so apps may arrange modules their own way
- [x] **M0-18** `cargo deny` checks dependencies against RustSec on every pull request and weekly (`.github/workflows/security.yml`), and allows crates.io only. RUSTSEC-2023-0071 (rsa, Marvin Attack) is ignored with its reason: sqlx-mysql only uses the server's public key

## M1: HTTP layer & controllers

Goal: Laravel-style routes, controllers and validation built fresh on hyper + matchit.

Router

- [ ] **M1-01** `get/post/put/patch/delete`, path params, `{*rest}` wildcard
  - [x] **M1-01a** `get`/`post`/`put`/`patch`/`delete`, `{id}` params and the `{*rest}` wildcard, matched by `matchit`
  - [ ] **M1-01d** Graceful shutdown on Ctrl-C / SIGTERM: stop accepting, let in-flight requests finish (measure tokio `macros` / `signal` cost first)
- [ ] **M1-02** Groups with prefix, group middleware, `merge`
  - [x] **M1-02a** Prefix groups like Laravel's `Route::prefix('api')->group(...)`: `route.prefix("/api").group(|route| ...)` or `.group(api::routes)`, nestable; the closure runs on a fresh `Router` and a non-generic `mount_group` adds the prefix
  - [x] **M1-02b** Group and route middleware, like Laravel: `route.prefix("/admin").middleware(Auth).group(...)`, `route.middleware(Auth).group(...)` and `route.get(...).middleware(Auth)`; global middleware is `route.global_middleware(...)` in `lib.rs`, like `withMiddleware` in `bootstrap/app.php`
  - [ ] **M1-02c** `route.merge(router)` for routes a package builds on its own
- [ ] **M1-03** Named routes and `route("posts.show", ...)` URLs with percent-encoding, like Laravel's `route()` helper
  - [x] **M1-03a** `route.get(...).name("posts.show")` and `routes.route("posts.show", &[("id", 7)])`: values percent-encoded, unused params become the query string, a duplicate name stops boot
  - [ ] **M1-03b** A `Url` handler argument, `url.route(...)`, with redirects (`redirect()->route()`, M1-21), where web pages need it; feature tests use `route(...)` through the test client (QA-01)
  - [ ] **M1-03c** Group name prefixes, `prefix("/admin").name("admin.")`, and absolute URLs from `app.url`
- [ ] **M1-04** Per-route and per-group body limits
- [x] **M1-05** Auto `HEAD`, `405` with `Allow` header, panic catching
  - [x] **M1-05a** A panic in a handler or middleware answers a logged `500` instead of an empty reply: caught around the handler, so middleware and the access log still see the `500`, and around the whole request as a safety net
  - [x] **M1-05b** `405` with an `Allow` header and Laravel's message, `HEAD` answered by the `GET` route (hyper drops the body and keeps `Content-Length`), and `OPTIONS` answered `200` with `Allow`, like Laravel
- [x] **M1-06** `sabai route:list` in Laravel's layout: methods colored, `GET|HEAD`, dots to the terminal's width, `name › PostController@index`, sorted by path, `Showing [n] routes`; handler names come from `type_name` at registration
- [x] **M1-07** Route-level middleware: `.get(...).middleware(x)` (M1-02b)
- [ ] **M1-08** Fallback route (custom 404), after views (M1-21): an API's 404 is already Laravel's `{"message":"Not Found"}`, so it is for HTML pages
- [ ] **M1-09** Route model binding: `{post}` resolves to `Post` (after M2)

Router: compile-time rules

- [x] **M1-10** `group` runs the closure on a fresh `Router`, then calls the non-generic `mount_group()` (M1-02a); `merge` (M1-02c) must do the same
- [x] **M1-11** `route.get(...)` boxes the handler, then calls the non-generic `push_route()`
  - [x] **M1-11b** Measured LLVM lines per route and kept the per-handler design: about 180 for a plain route and 450 with arguments. Sharing code per `(Args, R)` shape halves routes that share a shape but doubles routes that each return their own type, which is how Sabai apps are written (`Json<Post>`, `Json<Vec<Tag>>`). Boxing, `Arc::new` and `Router::get` alone cost about 100 per route, so the budget is 200 plain and 500 with arguments
- [x] **M1-12** Middleware stored as `Arc<dyn Middleware>`, never nested generic layers (M1-29a): `Next` walks an `Arc<[Arc<dyn Middleware>]>` by position, so a chain of any length is one copy of code
- [x] **M1-13** `impl Handler` macro capped at 8 arguments
- [x] **M1-14** Rejected handler arguments are a `sabai::Error`, rendered by its one non-generic `into_response`

Controllers

- [x] **M1-15** Handlers as methods: `.get("/posts/{id}", PostController::show)`, with `self: Arc<Self>` (M1-18)
- [ ] **M1-16** `Router::resource("posts", PostController)` maps the 7 REST actions
- [x] **M1-17** `route.api_resource("posts", |posts| { posts.index(...); posts.show(...); })`: Laravel's paths, methods and names (`posts.index` ... `posts.destroy`, `update` on `PUT` and `PATCH`), only the actions declared; `{id}` instead of `{post}` since there is no model binding
- [x] **M1-18** Controllers like Laravel's constructor injection: `FromApp` (`fn new(app) -> Result<Self>`) for controllers, services and repositories, `app.make::<T>()` for concrete types, actions as `async fn store(self: Arc<Self>, ...)`, every controller built once when `serve` starts so a missing binding stops startup
  - [x] **M1-18a** `FromApp` and `app.make::<T>()`; `serve` prepares every route's arguments before listening, so a `Service<T>` that is not bound stops startup and names the route
  - [x] **M1-18b** `Controller` trait and `app.singleton::<T>()`: a controller is made once when `serve` starts and shared by every request
  - [x] **M1-18c** The blog's `PostController` -> `PostService` -> `dyn PostRepository`, with an in-memory repository until the ORM (M2)
  - [x] **M1-18d** Actions without `self` need nothing (`pub async fn index() -> ...` calling `Post::all()`), so `FromApp` and `Controller` are only for constructor injection; the Controllers docs show that first, and a missing `FromApp` says both ways out (`#[diagnostic::on_unimplemented]`)

Middleware

- [ ] **M1-29** `Middleware` trait: code before and after `next.run(req)`, plus terminable middleware that runs after the response is sent
  - [x] **M1-29a** `Middleware` and `Next`; `route.middleware(...)` runs on every route of that router, the top one also on 404 and 405, a group's only on its routes; `assert_header`
  - [ ] **M1-29b** Terminable middleware, run after the response is sent (no current need; add when a feature such as sessions or queued work wants it)
- [ ] **M1-30** `from_fn(fn)` for writing middleware as a plain async function (low priority: `async fn handle` is already short)
- [ ] **M1-31** Middleware groups (`web`, `api`) with a priority order, like Laravel's HTTP kernel. String aliases (`'auth'`) are not needed: in Rust the type is the alias
  - [x] **M1-31a** Default global middleware (`TrimStrings`, `ConvertEmptyStringsToNull`) in every app; `route.global_middleware(|middleware| ...)` with `append` and `remove::<T>()`, like `withMiddleware`; `FrameGuard` ships in `sabai::http::middleware`
  - [ ] **M1-31b** The `web` and `api` groups with their defaults, once there is something to put in `web` (cookies, sessions, CSRF: M1-22 to M1-24)
- [x] **M1-32** Parameterised middleware: a middleware is a struct, so its parameters are fields, `route.middleware(Throttle::per_minute(60))`, with no string parsing like `'throttle:60,1'`. The middleware themselves come with their features: throttle (M7-13), `auth("api")` (M3-06), `can(...)` (M3)
- [x] **M1-33** Request ID and access log (`TraceRequests`, the first default; `TestApp::with_header`): a tracing span per request with `X-Request-Id` (reuse the incoming value only if it is short and `[A-Za-z0-9._-]`, to block log injection; otherwise generate one), echoed in the response
- [ ] **M1-34** Timeout and compression (gzip, br)
  - [x] **M1-34a** Slow clients (slowloris): hyper's 30 s header timeout was silently off without a timer, so `serve` gives it a `TokioTimer`; reading the body is limited to 30 s (`408`). Tests against a real server send half a request. The body size and both times come from `config/app.toml` (`APP_MAX_BODY_KB`, `APP_HEADER_TIMEOUT`, `APP_BODY_TIMEOUT`); `0` stops the app
  - [ ] **M1-34b** A timeout for handlers, and compression
- [ ] **M1-35** Security headers: HSTS, CSP, X-Frame-Options, Referrer-Policy
- [ ] **M1-36** Static files from `public/` with cache headers
- [ ] **M1-37** Signed-URL check middleware
- [x] **M1-38** Trim strings and turn empty strings into `None`
  - [x] **M1-38a** `TrimStrings` and `ConvertEmptyStringsToNull` in `sabai::http::middleware`, editing the input bag at any depth like Laravel's `TransformsRequest`; passwords are never trimmed; `request.with_input(...)`

Request & response

- [x] **M1-19a** Handler arguments (Laravel's method injection): async `FromRequest` over `&Request`, `Param<T>` (a value that does not parse is a 404, like `whereNumber`), `Request` itself, up to 8 per handler
- [x] **M1-19b** `FormRequest`: a request struct is a handler argument, filled from the query string (`GET`) or a JSON or form body picked by Content-Type (malformed JSON 400, invalid input 422); `Json<T>` responses
- [x] **M1-19c** `Service<T>` handler argument (method injection: `clock: Service<dyn Clock>`) and `request.app()`, like Laravel's `app()`
- [ ] **M1-19** `Param<(A, B)>` for routes with several parameters
- [x] **M1-19d** `Request` is one `Arc`, so taking `request: Request` as a handler argument costs a counter increment instead of copying every header (222ns to 21ns with 8 headers), in any argument position; `with_route_params` and `with_app` change it in place with `Arc::make_mut` before any clone exists
- [x] **M1-19e** Measured, not kept: JSON through a non-generic `serde_json::Value` cut the blog from 12,924 to 10,198 LLVM lines (~330 per extra request struct) but did not change rebuild time, doubled JSON parse time (317ns to 627ns) and let duplicate keys through; every input through `Value` cut 30% but broke typed form fields (`draft=true` into `bool`). Revisit with M1-28
- [ ] **M1-19f** Hot-path trims, each kept only if a benchmark shows it: route parameter names as `Arc<str>` shared from the endpoint (one allocation per parameter instead of two), a cheap hasher for the `TypeId` maps in `Container` and `singleton`, and `Error` as a `Box` around its fields so every `Result` is one pointer wide, and `FormRequest::rules()` built once in `prepare` instead of on every request
- [ ] **M1-20** `#[derive(FromRequest)]` for grouping handler arguments
- [ ] **M1-21** Responses: `json()`, `redirect()`, `view()` hook, `download()`
- [ ] **M1-22** Cookies (signed + encrypted)
- [ ] **M1-23** Sessions: driver trait, cookie and Redis drivers
- [ ] **M1-24** CSRF middleware for form routes
- [ ] **M1-25** CORS, trust-proxy, rate-limit middleware
- [ ] **M1-39** Request helpers: `input("name")` (query + body), `only`, `except`, `has`, `bearer_token()`, `ip()` behind trust-proxy, `wants_json()`, `route_is("posts.*")`
  - [x] **M1-39a** The input bag: the query string merged with the JSON or form body, parsed once on first use; `input`, `all`, `is_json`; form requests and `Rules::validate` read it
- [ ] **M1-40** `Extension<T>` for per-request data set by middleware
- [ ] **M1-41** Raw body (`Bytes`) and streaming request body
- [ ] **M1-42** Method spoofing (`_method=PUT`) for HTML forms
- [ ] **M1-43** Flash data and old input for redirect-back flows
- [ ] **M1-44** Locale from `Accept-Language`
- [ ] **M1-45** `IntoResponse` for `(StatusCode, headers, body)` tuples; `(StatusCode, T)` and `Result<T, E>` are done
- [ ] **M1-46** Status helpers: `created()` with `Location`, `no_content()`, `accepted()`
- [ ] **M1-47** Redirects: `back()`, `to_route("posts.show", params)`, `.with("status", "Saved")`
- [ ] **M1-48** Streaming responses, Server-Sent Events, `file()` inline with range requests
- [ ] **M1-49** Central exception handler: every error becomes one response shape (`{message, errors}`) by default, replaceable by the app via a registered error renderer (`Arc<dyn ErrorRenderer>`); before rendering, errors (5xx by default) go to every registered `Arc<dyn ErrorReporter>`, like Laravel's report/render split; debug page in dev only, no internals in production; server errors are already logged with their cause and context (M2-01a)
  - [ ] **M1-49a** Errors remember where they were made (`#[track_caller]`, a `&'static Location`, no stack walk), and logs print `at src/app/...rs:18`
  - [ ] **M1-49b** With `APP_DEBUG=true`, a `500` answers like Laravel: `{"message", "exception", "file", "line", "trace"}` with the context chain; with it off, only `{"message": "Server Error"}`
  - [ ] **M1-49c** Optional full backtrace when `APP_DEBUG=true` and `RUST_BACKTRACE=1`, tokio frames filtered out
  - [ ] **M1-49d** Logs drop the client message (`: Internal Server Error`) when an error already has context

Validation

- [x] **M1-26** Dropped: rules are typed functions returned by `fn rules()` (M1-28b), so no derive is needed
- [ ] **M1-27** More rules: `email`, `min`, `in`, `array`, `confirmed`, `unique` (after M2); `required`, `sometimes`, `string` and `max` are done (M1-28b)
- [x] **M1-28** `FormRequest` validation returns `422` with field errors, like Laravel; a wrong type (`title` not a string, `draft=yes` for a `bool`) also lands under its field instead of one serde sentence. Revisit M1-19e here: one lenient deserializer over `Value` that reads form strings into `bool` and numbers like Laravel's `boolean` and `integer` rules, which also cut ~30% of LLVM lines
  - [x] **M1-28a** `Error::with_messages(&[("email", "...")])`: a `422` with `{"message": "... (and 1 more error)", "errors": {"email": ["..."]}}`, fields in the order given, like Laravel's `ValidationException::withMessages`
  - [x] **M1-28b** Typed rules in `sabai::http::rules`: `Rules::new().field("title", [required(), string(), max(255)])` and `rules.validate(&input)`; `required` (stops the field, like Laravel), `sometimes`, `string`, `max` (characters, number or items), Laravel's English messages with `device_uuid` read as "device uuid"
  - [x] **M1-28c** `FormRequest::rules()` runs on the raw input (JSON as sent, form fields as strings) before the struct is built, so a missing or wrong-typed field is a field error, not a serde sentence; parsing and validating are not generic, only the final deserialize is; the blog's `StorePostRequest` has rules and feature tests
  - [x] **M1-28d** One deserializer per form request, not two: JSON and form input both go through sabai-http's `InputDeserializer`, which parses text where the struct asks for a `bool`, a number or a unit enum, like serde_urlencoded did. A route with a form request fell from about 4k to 2.5k LLVM lines in the app; sabai-http no longer depends on serde_urlencoded
- [ ] **M1-50** Apps swap the error renderer for their own shape, such as `{"errors": [{"ref": "email", "type": "validation", "desc": [...]}]}`; Laravel's default `{"message", "errors": {"email": [...]}}` is done (M1-28a)
- [x] **M1-51** Nested and array fields: `items.*.qty`, `author.name`, and the `array()` rule; errors are named by position (`posts.1.title`) like Laravel; the blog imports posts with `POST /api/posts/import`
- [ ] **M1-52** Conditional rules: `nullable`, `sometimes`, `required_if`, `required_with`, `bail`
- [ ] **M1-53** More rules: `regex`, `url`, `uuid`, `date`, `before`/`after`, `numeric`, `integer`, `between`, `same`, `exists`, `enum`
  - [x] **M1-53a** The rules registration needs, with Laravel's messages: `email`, `lowercase`, `confirmed` (also for nested fields: `users.0.password_confirmation`) and `min`; rule names take list indexes, `posts.0.title`
- [ ] **M1-54** File rules: `file`, `image`, `mimes`, max size (with M7-04)
- [ ] **M1-55** Custom rules: `impl Rule` (sync) and `impl AsyncRule` for DB checks
- [ ] **M1-56** Custom messages and attribute names, Thai and English (with M7-23)
- [ ] **M1-57** `authorize()` on request structs, like Laravel FormRequest: 403 before validation runs. After M3-05, and `async` from the start because real checks read the database
- [ ] **M1-58** Web forms: failed validation redirects back with errors and old input; APIs get 422
- [ ] **M1-59** `Validator::make(data, rules)` for dynamic rules without a derive

HTTP caching

- [ ] **M1-60** `Cache-Control` helpers: `.cache_for(duration)`, `.no_store()`, `.private()`
- [ ] **M1-61** ETag / Last-Modified with automatic `304 Not Modified`
- [ ] **M1-62** Response cache middleware: stores whole GET responses in the cache store, keyed by route, query and user, with tags for invalidation
- [ ] **M1-63** Correct `Vary` headers (Accept, Accept-Language, Authorization)

## M2: ORM (Eloquent-style)

Goal: `Post::query().where("published", true).with("author").paginate(20)` against MySQL, SQLite and Postgres.

Connection

- [ ] **M2-01** Pool on top of `sqlx`: MySQL first (what most of our Laravel apps run), SQLite for tests, then Postgres
  - [x] **M2-01a** `Database` from `config/database.toml` (Laravel's `default` + `connections`), bound by `DatabaseServiceProvider`; `sqlx::Any` so `DB_CONNECTION` picks MySQL or SQLite at runtime; the pool starts on first use (sqlx needs tokio, which starts after boot); `:memory:` keeps one connection; a down database fails in 5 seconds with a hint; `docker-compose.yml` runs MySQL for dev; the blog's `/api/health` pings it
  - [x] **M2-01b** One tokio runtime per process (`sabai::runtime()`, `sabai::block_on`), used by `serve` and `migrate`, so a pool opened by one command still works in the next; before, `migrate` then a query lost an in-memory SQLite database and timed out on MySQL
  - [ ] **M2-01c** TLS to MySQL (`DB_SSL_MODE` like Laravel's `sslmode`, `require` outside local), and docs that say why: without TLS, sqlx fetches the server's RSA key over the plain connection, so someone in the middle can swap the key and read the password (found by M0-18's review of RUSTSEC-2023-0071)
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
  - [x] **M2-09a** `#[derive(Model)]` emits `TABLE` (`Post` -> `posts`, `BlogPost` -> `blog_posts`, `Category` -> `categories`; `#[model(table = "people")]` for irregular plurals), `COLUMNS` and `from_row`; `Post::all()` finds its database through `App::current()`; `Row::get` reads `u64` through `i64` and refuses negatives; `database.statement(sql)` like `DB::statement`
- [ ] **M2-10** `find`, `find_or_fail` (404), `all`, `first`, `create`, `save`, `delete`
  - [x] **M2-10a** `Post::find(id)`, `Post::find_or_fail(id)` (404, like `ModelNotFoundException`), `post.save()` inserts when `id` is `0` and fills it in, otherwise updates; values bind as `SqlValue` in one non-generic function; SQLite gets its id through `returning`, MySQL through `last_insert_id`. Checked on MySQL 8.4 and SQLite
  - [x] **M2-10b** The blog's `PostRepository` is a plain struct with `async fn`s over `Post::all`, `find_or_fail` and `save`, like a Laravel repository class; the in-memory store is gone. Text columns read as `String` even when MySQL sends `TEXT` as `BLOB` through sqlx's `Any`
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
  - [x] **M2-21a** `trait Migration { fn up(&self, schema); fn down(&self, schema) }`, `Migrations::new().add(name, migration)`, `database.migrate(&migrations)` runs the pending ones as one batch and records them in Laravel's `migrations` table (`increments id`, `string migration`, `integer batch`); a failed migration is named and not recorded
  - [x] **M2-21b** `migrate` refuses a list with a repeated name or names out of order, before anything runs, since Laravel always runs migrations sorted by file name
- [ ] **M2-22** Packages register migrations from their provider; `vendor:publish --tag=migrations` copies them into the app
- [ ] **M2-23** Schema builder: create/alter/drop table, indexes, foreign keys, `timestamps()`, `soft_deletes()`, `morphs()`. Every column type and modifier produces exactly Laravel's SQL for that driver (signed or unsigned, lengths, nullability, `morphs` as `{name}_type varchar(255)` plus `{name}_id bigint unsigned` and an index)
  - [x] **M2-23a** `schema.create("posts", |table| { table.id(); table.string("title"); table.text("body"); table.timestamps(); })`, `drop_if_exists`, `.nullable()`; `schema.to_sql(Driver::MySql | Driver::Sqlite)` gives Laravel's SQL for each, with quoted names; `increments` and `integer` for the `migrations` table
  - [x] **M2-23b** `.unique()` as Laravel's separate statement (`alter table ... add unique users_email_unique(...)` on MySQL, `create unique index` on SQLite) and `remember_token()`; the blog gets Laravel's `users` table, checked on MySQL
- [ ] **M2-24** Commands: `migrate`, `migrate:rollback --step`, `migrate:reset`, `migrate:fresh`, `migrate:status`, `--pretend` (print SQL only)
  - [x] **M2-24a** `sabai::orm::migrate(&app, &migrations)` prints like `php artisan migrate`; the blog runs it as `cargo run -p blog -- migrate`, keeps its migrations in `src/database/migrations`, and its tests call `TestApp::refresh_database(&blog::migrations())` like Laravel's `RefreshDatabase`
- [ ] **M2-25** One transaction per migration where the driver supports transactional DDL
- [ ] **M2-26** Advisory lock so two deploys never migrate at once
- [ ] **M2-27** `schema:dump` to squash old migrations into one SQL file
- [ ] **M2-28** Migrations live in the app's own `database` crate, so editing them never rebuilds handlers
- [ ] **M2-29** Seeders and factories (`Post::factory().count(10).create()`)
- [ ] **M2-30** Pagination: `paginate(n)` and `cursor_paginate(n)` with JSON shape
- [ ] **M2-31** Models find their connection like Laravel's, without passing `db`: `Post::all()`, `#[model(connection = "report")]`
  - [x] **M2-31a** `App::current()` (Laravel's `app()`), set per task by `serve`, `TestApp` and `migrate` through `app.scope(future)`; `sabai::spawn` carries it into background tasks, and a missing one says to use it. Measured against passing `db`: the same query time (+19 ns), fewer LLVM lines per route
  - [x] **M2-31b** Several connections: `DatabaseManager` reads every `[connections.*]` (one lazy pool each) and `connection("report")` like `DB::connection('report')`; `Database` stays bound as `default`; `#[model(connection = "report")]` like `$connection`; a migration's `fn connection()` runs it there while `migrations` stays on `default`. Checked on two SQLite and two MySQL databases. A connection missing from config is found at the first query, not at startup, since models are not route arguments
  - [x] **M2-31c** The blog keeps `post_views` on a second connection, `report`: a second MySQL container in docker compose for dev (port 3307, its own `MYSQL_DATABASE`, so no init script and no root grants), SQLite in memory in tests; `GET /api/posts/{id}` records a read and `GET /api/stats` counts them (with `all().len()` until `count()` in M2-07); schema gets `unsigned_big_integer`, Laravel's `bigint unsigned`
  - [ ] **M2-31d** A misspelled connection fails to compile: `#[model(connection = REPORT)]` takes a constant as well as a string; `migrate` checks every migration's connection exists before running any

## M3: Auth

Goal: session login for web routes and token auth for API routes, both behind one `Auth` handler argument.

Core

- [ ] **M3-01** `trait Authenticatable` (id, password hash) implemented by the `User` model
- [ ] **M3-02** `trait UserProvider` with an ORM-backed default
- [ ] **M3-03** Password hashing with Argon2id, `Hash::make` / `Hash::check`, rehash on login
  - [x] **M3-03a** `sabai::auth::Hash` (feature `auth`) like Laravel's: `config/hashing.toml` with `config/hashing.php`'s keys and defaults (bcrypt, 12 rounds, `verify`; or `argon2id`, 64 MiB, t=4, behind the `argon2` feature so bcrypt apps do not compile its nine crates); `$2y$` hashes that a PHP-made hash checks against; passwords over bcrypt's 72 bytes refused; `make` and `check` `async` on a blocking thread, at most one per CPU core; bcrypt and argon2 optimized in the dev profile
  - [ ] **M3-03b** The blog and the template hash passwords: creating a user, `config/hashing.toml`, and the `[profile.dev.package...]` lines in the app's `Cargo.toml`
- [ ] **M3-04** Guards as `Arc<dyn Guard>`: `session` and `token`, picked per route group
- [ ] **M3-05** Handler arguments: `Auth<User>` (required, else 401/redirect) and `OptionalAuth<User>`
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
- [ ] **M4-10** Retries with exponential backoff, then `failed_jobs` table; final failures go to the same error reporters as HTTP errors
- [ ] **M4-11** `queue:failed`, `queue:retry {id}`, `queue:flush`
- [ ] **M4-12** Job middleware: rate limit, without-overlapping

Beyond basics

- [ ] **M4-13** Chains: run B after A succeeds
- [ ] **M4-14** Batches with progress and `then`/`catch` callbacks
- [ ] **M4-15** Queued event listeners (from M0 events): a listener opts in like Laravel's `ShouldQueue`; the event is serialized and pushed to the configured queue connection (`sync` runs it inline, `database`, `redis`, ...). `listen` and `dispatch` keep their signatures
- [ ] **M4-16** Unique jobs (lock by key)
- [ ] **M4-17** Metrics: jobs processed, failed, wait time (tracing / Prometheus)
- [ ] **M4-18** Jobs carry the request id of the request that dispatched them, so worker logs link back to it (like Laravel's `Context`)

## M5: Console kernel & commands (Artisan-style)

Goal: one `sabai` binary for generators and framework tasks, plus user-defined commands and a scheduler.

Kernel

- [ ] **M5-01** `sabai-cli` on `clap`, installed as `cargo install sabai-cli`
  - [x] **M5-01a** The `sabai` binary (clap builder API, no derive): every command goes to the nearest app's binary as `cargo run -q -- <command>`, so app and package commands work at once; `sabai` alone lists them; outside an app it says to `cd` into one
  - [ ] **M5-01b** Publish: move `templates/` into `crates/sabai-cli` so `cargo publish` packs it, and have `sabai new` write `sabai = "<the CLI's version>"` instead of the git dependency, so template and framework always match
  - [ ] **M5-01c** `sabai new <name> --using <git-url>` for a team's or the community's starter kit, like `laravel new --using=`
- [ ] **M5-02** Two modes: global (`sabai new`) and in-app (delegates to the app's own binary so app commands see app code)
- [ ] **M5-03** `trait Command` with `signature`, `description`, `async fn handle(&self, io)`
  - [x] **M5-03a** `trait Command` with `name`, `description` and `handle(&self, app, args)`; apps and packages add commands in a provider with `app.command(...)`, like `$this->commands([...])`; they show in `list`, and a duplicate name is an error. Sabai's own commands are `Command`s in the crate that owns them: `ServeCommand` in sabai-http, `MigrateCommand` in sabai-orm
- [ ] **M5-04** `#[derive(Command)]` maps struct fields to args/options
- [ ] **M5-05** Console IO helpers: `info`, `warn`, `error`, `table`, `confirm`, `ask`, progress bar
- [ ] **M5-06** Commands registered in a `ConsoleKernel` like the HTTP router
  - [x] **M5-06a** `sabai::console::Console::new(app).routes(...).migrations(...).run()` in the app's `main`: `serve` (the default), `migrate` and `list`, read without clap so apps compile no argument parser; an unknown command says to run `--help`

Built-in commands

- [ ] **M5-07** `route:list`
- [ ] **M5-08** `new <app>`, `serve`, `about`, `env`
  - [x] **M5-08a** `sabai new <name>` writes `templates/app` (embedded with `include_str!`, so it works offline): `Cargo.toml`, `.env` copied from `.env.example`, `config/`, `main.rs` on `Console`, `AppServiceProvider`, `routes/web.rs` with one closure route for `/`, like Laravel's `routes/web.php`; `--sabai-path <clone>` for a local Sabai; refuses an existing folder or a name Cargo cannot use
  - [x] **M5-08c** `build.rs` lists every file in `templates/app`, so a new template file needs no code; `dot-` names become `.` names and `.stub` is dropped (`templates/README.md`); the app's library is named `bootstrap` (`[lib] name`), so `main.rs` and tests read `bootstrap::app(config)` in every app and the template needs no crate-name placeholder
  - [x] **M5-08b** The template gets the database like Laravel 11: SQLite in `database/database.sqlite` by default (MySQL lines in `.env.example`), the `users` migration, a `User` model that never serializes `password`, `.env.testing` and `ExampleTest` on SQLite in memory; `{{sabai}}` and `{{sabai-testing}}` name only where the crates come from
- [ ] **M5-09** `make:controller`, `make:model -m`, `make:migration`, `make:job`, `make:command`, `make:policy`, `make:middleware`; Laravel's folders by default, nested names like `make:controller Store/PostController` (to `app/http/controllers/store/post_controller.rs`), and config to point each kind elsewhere (`make.models = "app/domain/models"`)
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
- [ ] **M6-03** Providers can add routes, middleware, config defaults, migrations, commands, listeners, jobs, guards and handler arguments
- [ ] **M6-04** `vendor:publish --provider=<name> --tag=config|migrations|views` copies package files into the app
- [ ] **M6-05** Config merge: package defaults, overridden by the app's `config/<package>.toml`

Model extension (what a MediaLibrary needs)

- [ ] **M6-06** Model traits from other crates: `impl HasMedia for Post` gives `post.add_media(file)`
- [ ] **M6-07** Polymorphic relations: `morph_to`, `morph_many` (media, comments, tags all need them)
- [ ] **M6-08** Model events / observers: `creating`, `saved`, `deleted` hooks packages can subscribe to
- [ ] **M6-09** Filesystem layer: `Storage::disk("s3")` (built in M7)

Auth extension (what a Sanctum needs)

- [ ] **M6-10** Custom guards registered by name: `auth.extend("sanctum", ...)`
- [ ] **M6-11** `FromRequest` public and stable, so packages ship their own handler arguments
- [ ] **M6-12** Package commands show up in `sabai list`

Stability & tooling

- [ ] **M6-13** `sabai::contracts` module holds every trait packages implement, covered by semver
- [ ] **M6-14** `sabai make:package <name>` scaffolds provider, config, migration and a test
- [ ] **M6-15** `sabai-testing` boots a test app with just one provider, for package tests
- [ ] **M6-16** Dogfood: ship `sabai-sanctum` and `sabai-media` as separate crates, public APIs only
- [ ] **M6-17** Package directory page on getsabai.dev
- [ ] **M6-18** `sabai-sentry` package: an `ErrorReporter` plus a `tracing` layer, DSN, release and environment from `config/sentry.toml` (`${SENTRY_DSN}`), request context (route, user id, request id); public API only, so the `sentry` SDK stays out of apps that skip it; uses the location, context chain and backtrace from M1-49

## M7: Storage, cache, mail & app services

Goal: the services every real app needs, each a contract in `sabai-core` with a fake for tests, so packages and test helpers plug in without changes.

Storage (sabai-storage)

- [ ] **M7-01** `Storage::disk("local" | "public" | "s3")` configured in `config/storage.toml`
- [ ] **M7-02** `put`, `get`, `exists`, `delete`, `copy`, `move`, `size`, `last_modified`, list files and directories
- [ ] **M7-03** Streaming reads and writes for large files, never loaded fully into memory
- [ ] **M7-04** `UploadedFile` handler argument (multipart) with `store("avatars")` and size / MIME validation
- [ ] **M7-05** Public URLs and expiring signed URLs (S3 presigned, local via signed route)
- [ ] **M7-06** Per-file visibility: public or private
- [ ] **M7-07** `storage:link` command: public disk to `public/storage`
- [ ] **M7-08** S3-compatible driver (AWS, Cloudflare R2, MinIO) in the separate `sabai-storage-s3` crate
- [ ] **M7-09** `Storage::fake("s3")` with `assert_exists` / `assert_missing`

Cache (sabai-cache)

- [ ] **M7-10** App-facing `Cache` with Laravel's names (`get`, `put`, `forever`, `has`, `missing`, `pull`, `add`, `increment`, `remember`, `remember_forever`, `flush`), TTL, tags (Redis); built on the `CacheStore` contract from M0-13, so drivers only implement its atomic primitives
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

- [ ] **QA-01** In-process test client, like Laravel's feature tests: `app().get_json(route(ROUTE)).assert_status(200).assert_json_fragment(...)`
  - [x] **QA-01a** `TestApp` (prepares routes like `serve`, runs handlers on tokio, plain `#[test]`), `get_json(route(ROUTE).with("id", 1))`/`post_json(route(ROUTE), json!(...))` with `route()` as a free function like Laravel's helper, `assert_status`/`assert_ok`/`assert_created`/`assert_not_found`/`assert_json_fragment`; the blog has `src/lib.rs` and `tests/feature/<area>/<controller>/<action>.rs` in one test binary
  - [ ] **QA-01b** `assert_json_count(n, "data")`, `assert_json_structure(...)`, `assert_json_path`, `put_json`/`patch_json`/`delete_json`
- [ ] **QA-02** `acting_as(&user)` for authenticated tests
- [ ] **QA-03** `RefreshDatabase`: migrate once per run, one rolled-back transaction per test
  - [x] **QA-03a** `Config::load_for(base, "testing")` reads `.env.testing` with `APP_ENV` forced, like Laravel's tests, so the blog's feature tests run on SQLite in memory even when `.env` points at the development MySQL
- [ ] **QA-04** DB assertions: `assert_database_has`, `assert_database_missing`, `assert_soft_deleted`, `assert_model_exists`
- [ ] **QA-05** Factories: states (`.published()`), sequences, relations (`has`, `for`), `make()` without saving
- [ ] **QA-06** Fakes with assertions: Queue, Mail, Event, Notification, Storage, Http
- [ ] **QA-07** Time travel: `travel_to(date)`, `freeze_time()` (built on the `Clock` contract)
- [ ] **QA-08** Parallel tests with one database per worker
- [ ] **QA-09** `sabai test` wrapper over `cargo nextest`

Docs & example

- [ ] **DOC-01** `examples/blog` exercising every milestone, with the same nouns as the docs (posts, comments, users)
- [ ] **DOC-02** getsabai.dev docs site: install, routing, controllers, ORM, auth, queue, CLI
  - [x] **DOC-02a** VitePress site in `docs/` (Laravel's page names), built on every PR and deployed to GitHub Pages from `main`; its own theme (teal, Anuphan and IBM Plex Sans Thai, a hammock logo, Ferris in a hammock on the home page), `::: laravel` notes in Laravel red, Try it panels, Copy as Markdown and `llms.txt`; pages: Introduction, Installation, Configuration, Directory Structure, Routing, Controllers
  - [x] **DOC-02b** Requests (form requests), Validation, Responses, Errors, Service Container, Service Providers, in plain developer English; responses in the Try it panels come from the blog
  - [x] **DOC-02c** Database, Migrations, Eloquent, HTTP Tests
- [ ] **DOC-03** "Coming from Laravel" page mapping each Laravel concept to Sabai

Release

- [ ] **REL-01** Reserve names: crates `sabai`, `sabai-*`; GitHub org `sabai-rs`; domain `getsabai.dev`
- [ ] **REL-02** Publish `0.1.0` after M1 (routing + controllers usable on their own)
- [ ] **REL-03** Changelog and semver policy; MSRV pinned in CI
