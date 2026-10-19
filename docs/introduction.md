# Introduction

Sabai is a web framework for Rust that works like Laravel. Routes, controllers, form requests, providers, models, migrations and feature tests have the names and folders you know.

What Rust adds:

- One fast binary.
- Mistakes found when the app starts. A missing service or a duplicate route stops `serve`, and the message says how to fix it.

Sabai puts three things first, in this order:

1. **Developer experience.** The API reads like Laravel. Errors say how to fix them.
2. **Compile speed.** Every change is measured.
3. **Correctness and security.**

::: warning Early days
Sabai is not on crates.io yet, and the API still changes. The [roadmap](https://github.com/sabai-rs/sabai/blob/main/ROADMAP.md) shows what is done and what is next.
:::

## Coming From Laravel

| Laravel | Sabai |
| --- | --- |
| `routes/api.php` | `src/routes/api.rs` |
| `Route::apiResource('posts', PostController::class)` | `route.api_resource("posts", \|posts\| { ... })` |
| `app/Http/Controllers/PostController.php` | `src/app/http/controllers/post_controller.rs` |
| `StorePostRequest extends FormRequest` | `impl FormRequest for StorePostRequest` |
| `class Post extends Model` | `#[derive(Model)] struct Post` |
| `Post::findOrFail($id)` | `Post::find_or_fail(id).await?` |
| `php artisan migrate` | `cargo run -- migrate` |
| `$this->getJson(route('posts.show', ['id' => 1]))` | `app().get_json(route("posts.show").with("id", 1))` |
