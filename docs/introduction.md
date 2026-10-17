# Introduction

Sabai is a web application framework for Rust, built to feel like Laravel. If you know Laravel, you should already know where things go and what they are called: routes, controllers, form requests, service providers, Eloquent models, migrations and feature tests all work the way you expect.

Sabai keeps what makes Laravel enjoyable and adds what Rust gives for free: one fast binary, and mistakes caught when the app starts instead of in production. A route whose service is not bound, or two routes with the same name, stop `serve` before it listens, with a message that says how to fix it.

Three things guide every decision, in this order:

1. **Developer experience.** APIs read like Laravel, and errors say how to fix them.
2. **Compile speed.** Every change is measured for its effect on build time.
3. **Correctness and security.**

> Sabai is in early development and not yet published to crates.io. APIs still change. The [roadmap](https://github.com/sabai-rs/sabai/blob/main/docs/ROADMAP.md) lists what is done and what comes next.

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
