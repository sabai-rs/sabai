use std::future::ready;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, FromApp, Request, Result};

use crate::FromRequest;

/// A controller with constructor injection: made once from the app when `serve` starts, then
/// shared by every request through actions that take `self: Arc<Self>`. Actions that need nothing
/// from the app skip this trait and `self`: `pub async fn index() -> ...`.
///
/// ```ignore
/// impl PostController {
///     pub async fn store(self: Arc<Self>, request: StorePostRequest) -> Result<Json<Post>> {
///         Ok(Json(self.post_service.create(request).await?))
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
