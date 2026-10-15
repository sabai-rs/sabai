use sabai_core::Method;

use crate::handler::{BoxedHandler, Handler, boxed};
use crate::router::Router;

/// The actions of a REST resource, started by [`Router::api_resource`] like Laravel's
/// `Route::apiResource('posts', PostController::class)`. Declare only the actions the
/// controller has, which is Laravel's `->only([...])`.
///
/// | Action    | Method       | Path          | Name            |
/// |-----------|--------------|---------------|-----------------|
/// | `index`   | `GET`        | `/posts`      | `posts.index`   |
/// | `store`   | `POST`       | `/posts`      | `posts.store`   |
/// | `show`    | `GET`        | `/posts/{id}` | `posts.show`    |
/// | `update`  | `PUT, PATCH` | `/posts/{id}` | `posts.update`  |
/// | `destroy` | `DELETE`     | `/posts/{id}` | `posts.destroy` |
pub struct ApiResource<'a> {
    router: &'a mut Router,
    name: String,
    collection: String,
    member: String,
}

impl Router {
    /// Routes the REST actions of `name`; see [`ApiResource`].
    ///
    /// ```ignore
    /// route.api_resource("posts", |posts| {
    ///     posts.index(PostController::index);
    ///     posts.store(PostController::store);
    ///     posts.show(PostController::show);
    /// });
    /// ```
    pub fn api_resource<F: FnOnce(&mut ApiResource)>(&mut self, name: &str, actions: F) {
        actions(&mut ApiResource::new(self, name));
    }
}

impl<'a> ApiResource<'a> {
    // `{id}`, not Laravel's `{post}`: without route model binding the name only matters to
    // `route("posts.show").with("id", 1)`, and English plurals need no guessing.
    fn new(router: &'a mut Router, name: &str) -> Self {
        let name = name.trim_matches('/');
        Self {
            router,
            name: name.to_owned(),
            collection: format!("/{name}"),
            member: format!("/{name}/{{id}}"),
        }
    }

    /// `GET /posts`, named `posts.index`.
    pub fn index<H: Handler<Args>, Args: 'static>(&mut self, handler: H) -> &mut Self {
        self.add(Method::GET, Path::Collection, "index", boxed(handler))
    }

    /// `POST /posts`, named `posts.store`.
    pub fn store<H: Handler<Args>, Args: 'static>(&mut self, handler: H) -> &mut Self {
        self.add(Method::POST, Path::Collection, "store", boxed(handler))
    }

    /// `GET /posts/{id}`, named `posts.show`.
    pub fn show<H: Handler<Args>, Args: 'static>(&mut self, handler: H) -> &mut Self {
        self.add(Method::GET, Path::Member, "show", boxed(handler))
    }

    /// `PUT` and `PATCH /posts/{id}`, both named `posts.update`.
    pub fn update<H: Handler<Args>, Args: 'static>(&mut self, handler: H) -> &mut Self {
        self.add_update(boxed(handler))
    }

    /// `DELETE /posts/{id}`, named `posts.destroy`.
    pub fn destroy<H: Handler<Args>, Args: 'static>(&mut self, handler: H) -> &mut Self {
        self.add(Method::DELETE, Path::Member, "destroy", boxed(handler))
    }

    fn add_update(&mut self, handler: BoxedHandler) -> &mut Self {
        self.add(Method::PUT, Path::Member, "update", handler.clone());
        self.add(Method::PATCH, Path::Member, "update", handler)
    }

    fn add(
        &mut self,
        method: Method,
        path: Path,
        action: &str,
        handler: BoxedHandler,
    ) -> &mut Self {
        let path = match path {
            Path::Collection => &self.collection,
            Path::Member => &self.member,
        };
        let name = format!("{}.{action}", self.name);
        self.router.push_route(method, path, handler).name(&name);
        self
    }
}

enum Path {
    Collection,
    Member,
}
