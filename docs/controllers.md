# Controllers

## Writing Controllers

A controller groups related actions. An action that needs nothing from the app is a plain `async fn` without `self`, and is routed as `PostController::index`:

```rust
pub struct PostController;

impl PostController {
    pub async fn index() -> Result<Json<Vec<Post>>> {
        Ok(Json(Post::all().await?))
    }

    pub async fn show(Param(id): Param<u64>) -> Result<Json<Post>> {
        Ok(Json(Post::find_or_fail(id).await?))
    }

    pub async fn store(request: StorePostRequest) -> Result<(StatusCode, Json<Post>)> {
        let mut post = Post { id: 0, title: request.title, body: request.body };
        post.save().await?;
        Ok((StatusCode::CREATED, Json(post)))
    }
}
```

```rust
route.api_resource("posts", |posts| {
    posts.index(PostController::index);
    posts.show(PostController::show);
    posts.store(PostController::store);
});
```

Models find their database by themselves, so `Post::all()` works in a controller, a service or a repository alike.

::: laravel
```php
class PostController extends Controller
{
    public function index() { return Post::all(); }
    public function show(int $id) { return Post::findOrFail($id); }
}
```
:::

## Action Arguments

Like Laravel's method injection, an action declares what it needs and receives it filled in:

| Argument | What it gives |
| --- | --- |
| `request: StorePostRequest` | A form request, validated |
| `Param(id): Param<u64>` | The route parameter, parsed |
| `clock: Service<dyn Clock>` | A service bound in a provider |
| `request: Request` | The whole request |

An action takes up to 8 arguments, in any order.

## Constructor Injection

When every action needs the same service, take it in the constructor instead. Implement `FromApp` to build the controller from the app, and `Controller` to mark it; actions then take `self: Arc<Self>`:

```rust
pub struct PostController {
    post_service: PostService,
}

impl FromApp for PostController {
    fn new(app: &App) -> Result<Self> {
        Ok(Self {
            post_service: app.make::<PostService>()?,
        })
    }
}

impl Controller for PostController {}

impl PostController {
    pub async fn index(self: Arc<Self>) -> Result<Json<Vec<Post>>> {
        Ok(Json(self.post_service.all().await?))
    }
}
```

`app.make::<T>()` builds a class the way Laravel autowires one, through its own `FromApp`. `app.get::<dyn Trait>()` gives an interface bound in a provider.

The controller is made once, when `serve` starts, and shared by every request. If something it needs is not bound, the app does not start, and the message names the route, the controller and the missing service.

Services and repositories follow the same rules: a struct that needs something implements `FromApp`, and one that needs nothing can be a unit struct or plain functions.
