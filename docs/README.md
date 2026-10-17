# Sabai docs

The source of the docs site, built with [VitePress](https://vitepress.dev). It is published to <https://sabai-rs.github.io/sabai/> on every push to `main`.

## Run it

The docs use [Bun](https://bun.sh).

```bash
cd docs
bun install
bun run dev
```

Open <http://localhost:5173/sabai/>. Pages reload when you save.

To check the production build:

```bash
bun run build
bun run preview
```

## Files

```text
docs/
  index.md              home page (renders .vitepress/theme/components/SabaiHome.vue)
  introduction.md       one page per topic, named like Laravel's docs
  routing.md
  ...
  public/               logo and other static files
  .vitepress/
    config.mts          nav, sidebar, search, `::: laravel`, llms.txt
    theme/
      style.css         colors and fonts
      components/       SabaiHome, TryIt, PageMarkdown
  ROADMAP.md            not part of the site
  README.md             this file, not part of the site
```

## Add a page

1. Create `docs/<topic>.md`. Use Laravel's page name when there is one (`validation.md`, `eloquent.md`).
2. Add it to `sidebar` in `.vitepress/config.mts`.

## How to write

Write for a developer who knows Laravel and is reading quickly.

- Show the code first, then explain it in one or two short sentences.
- One idea per sentence. Plain words: "does not start", not "refuses to listen".
- Tell the reader what to do: "Run `cargo run -p blog -- migrate`."
- Every example must work with the code on `main`. Take it from `examples/blog` when you can.
- Say plainly when something does not exist yet.

Show the Laravel version with a Laravel note:

````md
::: laravel
```php
Route::apiResource('posts', PostController::class);
```
:::
````

Use `::: tip` for a short hint. Use `::: code-group` to show several files together.

## Deploy

`.github/workflows/docs.yml` builds the site on every pull request that touches `docs/`, and deploys it to GitHub Pages on `main`. The repository's Pages source must be set to **GitHub Actions**.

The build also writes `llms.txt`, `llms-full.txt` and a `.md` copy of each page, so AI tools can read the docs.
