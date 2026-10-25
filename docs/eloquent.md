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

### Where Clauses

`where_eq` keeps the rows where a column equals a value. Chain more for `and`, then take the `first()` or `get()` them all:

```rust
let user = User::where_eq("email", &request.email).first().await?;              // Option<User>
let drafts = Post::where_eq("author_id", 7).where_eq("published", false).get().await?; // Vec<Post>
let unread = Post::where_eq("read_at", None::<String>).get().await?;             // `read_at is null`
```

Values are always bound, never written into the SQL. A column must be one of the model's fields, so a name that came from a request can never reach the SQL either: an unknown one is an error when the query runs.

::: laravel
`User::where('email', $email)->first()`. Rust keeps `where` for its own use, so this one is `where_eq`, which also says it compares for equality. It is the only name that differs: the rest will keep Laravel's, such as `or_where`, `where_in` and `where_null`.
:::

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

### A Taken Unique Value

When a unique index refuses a row, such as a second user with the same email, `is_unique_violation` tells you, so you can answer like Laravel's `unique` rule:

```rust
use sabai::orm::is_unique_violation;

match user.save().await {
    Err(error) if is_unique_violation(&error) => {
        Err(Error::with_messages(&[("email", "The email has already been taken.")]))
    }
    saved => saved.map(|()| user),
}
```

The database checks while it saves, so two requests at once cannot both take the same email, which a check before saving could miss.

## Field Types

| Rust | Column |
| --- | --- |
| `u64`, `u32`, `i64`, `i32` | integer |
| `f64` | float |
| `bool` | boolean |
| `String` | text |
| `DateTime` | `timestamp` or `datetime` |
| `Option<T>` | a nullable column |

A negative number in a `u64` field is an error, never a wrong value.

### Dates

A [`DateTime`](/dates) field reads a `timestamp` column on MySQL and a `datetime` column on SQLite, and is written as `2026-10-23 10:00:00`, like Laravel does. The columns from `table.timestamps()` are nullable, so their fields are `Option<DateTime>`:

```rust
#[derive(Model, Serialize)]
struct Post {
    id: u64,
    title: String,
    created_at: Option<DateTime>,
    updated_at: Option<DateTime>,
}
```

Times are UTC: Sabai sets each MySQL connection's `time_zone` to `+00:00`, since a `timestamp` column converts to and from the session's zone. MySQL's zero date `0000-00-00 00:00:00` is an error when read; store `NULL` instead.

## Not Yet

These are on the roadmap: `where` and the query builder, `count`, `delete`, automatic timestamps, casts, relations and eager loading.
