# Migrations

## Write a Migration

Migrations live in `src/database/migrations/`, one file each:

```rust
// src/database/migrations/m2026_10_15_000000_create_posts_table.rs
use sabai::orm::{Migration, Schema};

pub struct CreatePostsTable;

impl Migration for CreatePostsTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("posts", |table| {
            table.id();
            table.string("title");
            table.text("body");
            table.timestamps();
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("posts");
    }
}
```

::: laravel
```php
return new class extends Migration {
    public function up(): void
    {
        Schema::create('posts', function (Blueprint $table) {
            $table->id();
            $table->string('title');
            $table->text('body');
            $table->timestamps();
        });
    }
};
```
:::

The file name starts with `m` because a Rust module cannot start with a digit.

## List Your Migrations

Add each one to `src/database/migrations.rs`, in date order:

```rust
pub fn all() -> Migrations {
    Migrations::new()
        .add("2026_10_15_000000_create_posts_table", CreatePostsTable)
        .add("2026_10_17_000000_create_post_views_table", CreatePostViewsTable)
}
```

A name added twice, or a list out of date order, stops `migrate` before it runs anything.

## Run Migrations

```bash
cargo run -p blog -- migrate
```

```text
Running migrations.
  2026_10_15_000000_create_posts_table DONE
```

Like `php artisan migrate`:

- Only migrations that have not run yet run.
- Each run is one new batch, saved in the `migrations` table.
- A migration that fails is named in the error and not saved, so the next run tries it again.

`migrate:rollback`, `migrate:fresh` and `sabai migrate` come with the console.

## Column Types

Every column gives the same SQL as Laravel for each database.

| Method | MySQL | SQLite |
| --- | --- | --- |
| `table.id()` | `bigint unsigned` auto-increment primary key | `integer` primary key |
| `table.increments("id")` | `int unsigned` auto-increment primary key | `integer` primary key |
| `table.integer("votes")` | `int` | `integer` |
| `table.unsigned_big_integer("post_id")` | `bigint unsigned` | `integer` |
| `table.string("title")` | `varchar(255)` | `varchar` |
| `table.text("body")` | `text` | `text` |
| `table.timestamp("sent_at")` | `timestamp` | `datetime` |
| `table.timestamps()` | nullable `created_at` and `updated_at` | |
| `table.remember_token()` | nullable `varchar(100)` named `remember_token` | nullable `varchar` |

Add `.nullable()` to allow `NULL`:

```rust
table.text("summary").nullable();
```

Add `.unique()` for a unique index. Like Laravel, it is named `{table}_{column}_unique` and added right after the table:

```rust
table.string("email").unique();
```

```sql
alter table `users` add unique `users_email_unique`(`email`)
```

More column types, indexes and foreign keys are coming.

## Another Connection

A migration for another database names its connection. The `migrations` table stays on the default one:

```rust
impl Migration for CreatePostViewsTable {
    fn up(&self, schema: &mut Schema) { /* ... */ }
    fn down(&self, schema: &mut Schema) { /* ... */ }

    fn connection(&self) -> Option<&'static str> {
        Some("report")
    }
}
```
