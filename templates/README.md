# Templates

`sabai new` writes every file in `app/` into the new app; `crates/sabai-cli/build.rs` lists them, so a new file needs no code.

- `{{name}}` becomes the app's name. `{{sabai}}` and `{{sabai-testing}}` become where those crates come from, `git = "..."` or `path = "..."`, so the template writes the rest: `sabai = { {{sabai}}, features = ["orm"] }`.
- A name starting with `dot-` starts with `.` instead: `dot-gitignore` is written as `.gitignore`.
- A name ending in `.stub` loses it: `Cargo.toml.stub` is written as `Cargo.toml`. This keeps Cargo from reading the template as a package.
- `.env.example` is also copied to `.env`, like `laravel new`.
