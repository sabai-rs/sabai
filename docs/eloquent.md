# Eloquent

## Define a Model

Derive `Model` on a struct. Its fields are the table's columns:

```rust
use sabai::orm::Model;
use serde::Serialize;

#[derive(Model, Serialize, Clone)]
pub struct Post {
    pub id: u64,
    pub title: String,
    pub body: String,
}
```

::: laravel
```php
class Post extends Model {}
```
:::

- The table name comes from the struct, like Laravel: `Post` is `posts`, `BlogPost` is `blog_posts`, `Category` is `categories`.
- Every model needs an `id: u64` field. It matches `$table->id()`.

### Table Names

Set the table when the plural is irregular:

```rust
#[derive(Model)]
#[model(table = "people")]
pub struct Person {
    pub id: u64,
}
```

### Connections

Use another connection, like Laravel's `$connection`:

```rust
#[derive(Model)]
#[model(connection = "report")]
pub struct PostView {
    pub id: u64,
    pub post_id: u64,
}
```

## Read Models

```rust
let posts = Post::all().await?;                 // Post::all()
let post = Post::find(7).await?;                 // Post::find(7), an Option
let post = Post::find_or_fail(7).await?;         // Post::findOrFail(7), or a 404
```

Bring the trait in first: `use sabai::orm::Model;`.

Models find their database by themselves. There is no `db` to pass, in a controller, a service, a repository or a test.

## Save Models

`save` inserts a model whose `id` is `0`, then fills in the new id:

```rust
let mut post = Post { id: 0, title: "Sabai".into(), body: "Rust, but sabai".into() };
post.save().await?;
println!("{}", post.id);    // 1
```

Change a saved model and `save` again to update it:

```rust
post.title = "Sabai 2".into();
post.save().await?;
```

## Field Types

| Rust | Column |
| --- | --- |
| `u64`, `u32`, `i64`, `i32` | integer |
| `f64` | float |
| `bool` | boolean |
| `String` | text |
| `Option<T>` | a nullable column |

A negative number in a `u64` field is an error, never a wrong value.

## Not Yet

These are on the roadmap: `where` and the query builder, `count`, `delete`, automatic timestamps, casts, relations and eager loading.
