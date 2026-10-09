# Sabai

Laravel-inspired web framework for Rust. Tagline: "Rust, but sabai."
Built from scratch (no code from the old oxalin project).
Full roadmap and to-do list: `docs/ROADMAP.md`.

## Priorities (in order)

1. Developer experience: APIs should feel like Laravel, errors should explain the fix.
2. Compile speed: every change is judged on its effect on build time.
3. Correctness and security.

## Compile-time rules (must follow)

- Derive macros emit metadata only (consts, field lists, `from_row`). Logic lives in runtime crates.
- Every public generic API is a one-line shim over a non-generic inner function.
  ```rust
  pub fn get<H: Handler<A>, A>(self, path: &str, h: H) -> Self {
      self.push_route(Method::GET, path, h.boxed()) // non-generic
  }
  ```
- Closures passed to builders (`group`, `transaction`) are called first, then work goes to a non-generic fn.
- Pluggable parts (middleware, guards, queue/cache/storage/mail drivers) are `Arc<dyn Trait>`, never nested generic layers.
- `impl Handler` for at most 8 extractor args.
- `sabai-macros` never depends on runtime crates; it emits `::sabai::...` paths.
- Heavy optional deps (AWS SDK, mail, views) live in their own crate or behind a feature.
- Integration tests live in one binary: `tests/it/main.rs` with `mod` per area.

## Workspace layout

```text
crates/
  sabai/           facade: re-exports + features
  sabai-core/      Error, Request/Response, config, container, contracts (Clock, Cache, Filesystem, Mailer)
  sabai-macros/    proc-macros (syn with minimal features)
  sabai-http/      router, handlers, extractors, middleware, validation, sessions
  sabai-orm/       query builder, models, relations, schema builder, migrator
  sabai-auth/      guards, hashing, gates, policies
  sabai-queue/     jobs, drivers, worker
  sabai-cache/  sabai-storage/  sabai-storage-s3/  sabai-mail/  sabai-notify/
  sabai-cli/       `sabai` binary
  sabai-testing/   test client, fakes, DB assertions
packages/          first-party packages using public API only (sabai-sanctum, sabai-media)
templates/         `sabai new` skeletons
examples/todo-app/ dogfood app: every feature must be used here
```

Dependency direction only points down toward `sabai-core`. Crates at the bottom change least.

## Stack decisions

- tokio + hyper 1 + matchit for HTTP
- sqlx under the ORM (Postgres first, then MySQL, SQLite)
- clap for the CLI, tracing for logs, thiserror for errors
- Edition 2024; pin MSRV in CI

## Workflow

- Build a walking skeleton first: `sabai new blog` -> `sabai migrate` -> `sabai serve` -> `GET /posts` returns JSON from the DB. Deepen milestones after that works.
- After each change that touches public APIs or macros, run:
  ```bash
  cargo build --timings
  cargo llvm-lines -p <crate> | head -20
  ```
  Report build-time impact in the summary.
- Use the feature in `examples/todo-app` before calling it done.
- Work one ticket at a time by its ID from `docs/ROADMAP.md` (e.g. `M1-03`). If a ticket is too big for one small step, split it into `M1-03a`, `M1-03b` and say so.
- Branch per ticket: `m1-03-named-routes`. Commit messages start with the ID: `M1-03: named routes and url_for`.
- Tick the ticket in `docs/ROADMAP.md` in the same commit that finishes it.

## Commands

```bash
cargo check --workspace
cargo nextest run --workspace
cargo clippy --workspace --all-targets -- -D warnings
cargo fmt --all
```

## Code style

Code should read like prose. If it needs a comment to be understood, rename or restructure it first.

- Names carry the meaning: `push_route`, `mount_group`, `find_or_fail`, not `handle`, `process`, `do_it`.
- Small functions that do one thing. Early returns over nested `if`.
- No comments that restate the code. Never write `// create the router`, `// loop over routes`, `// return the result`.
- No section banners (`// ===== Helpers =====`), no emoji, no "Step 1:" comments.
- A comment is allowed only for **why**: a non-obvious constraint, a security reason, a workaround with a link.
- Public items get a one-line doc comment. Add an example only when usage is not obvious.
- No `unwrap()` in library code outside tests; return `sabai::Error`.
- Code, docs and commit messages in English.

Bad:

```rust
// Create a new vector to hold the routes
let mut routes = Vec::new();
// Loop through each route in the group
for route in group.routes {
    // Add the prefix to the route path
    route.path = format!("{prefix}{}", route.path);
    routes.push(route);
}
```

Good:

```rust
let routes = group.routes.into_iter().map(|route| route.prefixed(prefix));
```

## How we work together (not vibe coding)

I want to understand every line that lands. Optimise for my learning as well as the result.

- Small steps: one concept per step, diffs I can read in a few minutes (aim for under ~150 lines).
- Before coding a step, say in 2–3 sentences what you will build and which design choice matters.
- After each step, explain in chat (not in code comments): what changed, why this design, and the one or two lines worth reading closely.
- When you use a Rust technique for the first time (non-generic inner fn, `Arc<dyn Trait>`, sealed traits, `proc-macro-crate`), name it and explain it in chat once.
- Stop after each step and wait for my review before moving on.
- If I say "let me write it": give me the signature, the tests and hints only, then review my code instead of writing it.
- When I ask "why", answer the why; don't rewrite the code unless I ask.
