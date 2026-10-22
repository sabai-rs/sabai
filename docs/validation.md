# Validation

## Rules

Rules live in the form request's `rules()`, one `field` per input:

```rust
impl FormRequest for StorePostRequest {
    fn rules() -> Rules {
        Rules::new()
            .field("title", [required(), string(), max(255)])
            .field("body", [required(), string()])
    }
}
```

::: laravel
```php
public function rules(): array
{
    return [
        'title' => ['required', 'string', 'max:255'],
        'body'  => ['required', 'string'],
    ];
}
```
:::

Rules are Rust functions, so a typo does not compile and your editor suggests them. Import them from `sabai::http::rules`.

## Available Rules

| Rule | Passes when |
| --- | --- |
| `required()` | the field is there and not empty: not `null`, `""`, `[]` or `{}` |
| `sometimes()` | always; the field may be left out |
| `string()` | the value is a string |
| `array()` | the value is a list |
| `email()` | one `@`, no spaces, and a domain of letters, digits, `-` and `.` |
| `lowercase()` | the value has no uppercase letters |
| `confirmed()` | the value equals `{field}_confirmation`, such as `password_confirmation` |
| `min(n)` | a string has at least `n` characters, a number is at least `n`, a list has at least `n` items |
| `max(n)` | a string has at most `n` characters, a number is at most `n`, a list has at most `n` items |

Laravel Breeze's registration rules read the same way:

```rust
Rules::new()
    .field("name", [required(), string(), max(255)])
    .field("email", [required(), string(), lowercase(), email(), max(255)])
    .field("password", [required(), confirmed(), min(8)])
```

More rules are coming: `in`, `numeric`, `nullable`, `unique` and the rest of Laravel's list.

## Nested and Array Fields

A field name reaches into an object with `.` and into every item of a list with `*`. The blog imports several posts at once this way:

```rust
Rules::new()
    .field("posts", [required(), array(), max(50)])
    .field("posts.*.title", [required(), string(), max(255)])
    .field("posts.*.body", [required(), string()])
```

Each item is checked on its own, and an error names the item by its position, so the client knows which one to fix:

```json
{
    "message": "The posts.1.title field is required.",
    "errors": {
        "posts.1.title": ["The posts.1.title field is required."]
    }
}
```

A `*` over something that is not a list checks nothing, so pair it with `array()` on the list itself. The struct holds the items as a `Vec`:

```rust
#[derive(Deserialize)]
pub struct ImportPostsRequest {
    pub posts: Vec<StorePostRequest>,
}
```

::: laravel
The same as Laravel's `'posts.*.title' => ['required', 'string', 'max:255']`, with the same error keys.
:::

## How Rules Run

These work like Laravel:

- A field that is missing or `""` skips every rule except `required`.
- When `required` fails, the other rules of that field do not run. You get one message, not three.
- `device_uuid` is called "device uuid" in messages.

## Failed Validation

A failure answers `422` with Laravel's shape. Fields are listed in the order of your rules:

```json
{
  "message": "The title field is required. (and 1 more error)",
  "errors": {
    "title": ["The title field is required."],
    "body": ["The body field is required."]
  }
}
```

Try it on the blog's `POST /api/posts`:

<TryIt :requests="[
  { label: 'POST {}', status: '422 Unprocessable Entity', ok: false, body: '{&quot;message&quot;:&quot;The title field is required. (and 1 more error)&quot;,&quot;errors&quot;:{&quot;title&quot;:[&quot;The title field is required.&quot;],&quot;body&quot;:[&quot;The body field is required.&quot;]}}', why: 'Both fields are missing. required stops each field at one message.' },
  { label: 'POST {title: 7}', status: '422 Unprocessable Entity', ok: false, body: '{&quot;message&quot;:&quot;The title field must be a string.&quot;,&quot;errors&quot;:{&quot;title&quot;:[&quot;The title field must be a string.&quot;]}}', why: 'title is a number, so string() fails.' },
  { label: 'POST 256-char title', status: '422 Unprocessable Entity', ok: false, body: '{&quot;message&quot;:&quot;The title field must not be greater than 255 characters.&quot;,&quot;errors&quot;:{&quot;title&quot;:[&quot;The title field must not be greater than 255 characters.&quot;]}}', why: 'max(255) counts characters, not bytes, so Thai text counts right.' },
  { label: 'POST form body', status: '201 Created', ok: true, body: '{&quot;id&quot;:1,&quot;title&quot;:&quot;From a form&quot;,&quot;body&quot;:&quot;b&quot;}', why: 'title=From+a+form&body=b as a form passes the same rules.' },
]" />

## Errors From Your Own Code

Return the same `422` from anywhere, like `ValidationException::withMessages`:

```rust
return Err(Error::with_messages(&[("email", "These credentials do not match our records.")]));
```

Repeat a field to give it more than one message.
