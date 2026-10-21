# Contributing to Sabai

Thank you for considering contributing to Sabai! Work follows the [roadmap](ROADMAP.md), one ticket at a time:

- One branch per ticket, named after it: `m1-03-named-routes`.
- Commit messages start with the ticket: `feat(M1-03a): named routes ...`.
- Use the feature in [`examples/blog`](examples/blog) before calling it done, and document it in [`docs/`](docs) in the same pull request.
- Report the build-time impact (`cargo build --timings`, `cargo llvm-lines`) of changes to public APIs or macros.

## Compile Budgets

CI fails when a crate goes over its LLVM-lines budget in [`compile-budgets.toml`](compile-budgets.toml). The total is the `(TOTAL)` line of `cargo llvm-lines -p <crate> --lib`. Raising a budget is fine when the growth is worth it: change the number in the same pull request and say why.

## Before Opening a Pull Request

```bash
cargo fmt --all
cargo lint   # clippy on every crate and target, warnings are errors
cargo t      # nextest on the whole workspace
```

To preview the documentation, run `bun install` and then `bun run dev` in [`docs/`](docs).

[`CLAUDE.md`](CLAUDE.md) has the full conventions, including the compile-time rules every crate follows.
