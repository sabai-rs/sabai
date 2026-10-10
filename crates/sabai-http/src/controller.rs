use std::future::ready;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, FromApp, Request, Result};

use crate::FromRequest;

/// A Laravel-style controller: made once from the app's services when `serve` starts,
/// then shared by every request. Actions take `self: Arc<Self>` and are routed as `PostController::store`.
///
/// ```ignore
/// impl PostController {
///     pub async fn store(self: Arc<Self>, request: StorePostRequest) -> Result<Json<Post>> {
///         Ok(Json(self.posts.create(request).await?))
///     }
/// }
/// ```
pub trait Controller: FromApp + Send + Sync + 'static {}

impl<C: Controller> FromRequest for Arc<C> {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(request.app().and_then(|app| app.singleton())))
    }

    fn prepare(app: &App) -> Result<()> {
        app.singleton::<C>().map(drop)
    }
}
