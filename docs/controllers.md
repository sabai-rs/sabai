# Controllers

## Write a Controller

A controller groups actions. An action that needs nothing from the app is a plain `async fn` with no `self`:

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
        let mut post = Post { title: request.title, body: request.body, ..Default::default() };
        post.save().await?;
        Ok((StatusCode::CREATED, Json(post)))
    }
}
```

Route it by name:

```rust
route.api_resource("posts", |posts| {
    posts.index(PostController::index);
    posts.show(PostController::show);
    posts.store(PostController::store);
});
```

Models find their database on their own. `Post::all()` works the same in a controller, a service or a repository.

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

An action lists what it needs, and Sabai passes it in, like Laravel's method injection:

| Argument | You get |
| --- | --- |
| `request: StorePostRequest` | a form request, already validated |
| `Param(id): Param<u64>` | the route parameter |
| `clock: Service<dyn Clock>` | a service bound in a provider |
| `request: Request` | the whole request |

Up to 8 arguments, in any order.

## Constructor Injection

When every action needs the same service, put it in the constructor:

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

- `FromApp` builds the controller from the app.
- `Controller` marks it as a controller. Its actions take `self: Arc<Self>`.
- `app.make::<T>()` builds a class, like Laravel's autowiring.
- `app.get::<dyn Trait>()` gives an interface you bound in a provider.

The controller is built once, when the app starts, and shared by every request. If it needs a service nobody bound, the app does not start.

Services and repositories work the same way. Implement `FromApp` if they need something. If not, use a plain struct or plain functions.
