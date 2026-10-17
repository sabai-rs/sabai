use std::sync::Arc;

use http::header::{ACCEPT, CONTENT_TYPE};
use sabai_core::{App, Body, Method, Request};
use sabai_http::{Router, Routes};
use serde::Serialize;
use tokio::runtime::Runtime;

use crate::{TestResponse, ToUri};

/// The app under test, like Laravel's `TestCase`: requests go straight to the router, no port.
///
/// ```ignore
/// let app = TestApp::new(blog::app(config)?, blog::routes());
/// app.get_json(route("posts.show").with("id", 1)).assert_status(200);
/// ```
pub struct TestApp {
    app: Arc<App>,
    routes: Routes,
    runtime: Runtime,
}

impl TestApp {
    /// Prepares `router` against `app` the way `serve` does, so a missing binding fails here.
    pub fn new(app: Arc<App>, router: Router) -> Self {
        let routes = router.compile().expect("the routes are valid");
        routes.prepare(&app).expect("every route can start");
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("a tokio runtime starts");
        Self {
            app,
            routes,
            runtime,
        }
    }

    /// Runs `migrations` on the app's database, like Laravel's `RefreshDatabase`; with SQLite in
    /// memory every `TestApp` starts from an empty database.
    #[cfg(feature = "orm")]
    pub fn refresh_database(self, migrations: &sabai_orm::Migrations) -> Self {
        let database = self
            .app
            .get::<sabai_orm::Database>()
            .expect("a database is bound");
        self.runtime
            .block_on(self.app.scope(database.migrate(migrations)))
            .expect("the migrations run");
        self
    }

    /// Sends `GET` as a JSON client, like Laravel's `$this->getJson(route(...))`.
    pub fn get_json(&self, uri: impl ToUri) -> TestResponse {
        self.send_json(Method::GET, uri.to_uri(&self.routes), Body::empty())
    }

    /// Sends `data` as a JSON body, like Laravel's `$this->postJson(...)`.
    pub fn post_json(&self, uri: impl ToUri, data: impl Serialize) -> TestResponse {
        let body = serde_json::to_string(&data).expect("the test data serializes to JSON");
        self.send_json(Method::POST, uri.to_uri(&self.routes), Body::from(body))
    }

    fn send_json(&self, method: Method, uri: String, body: Body) -> TestResponse {
        let request = http::Request::builder()
            .method(method)
            .uri(uri)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .body(body)
            .expect("the test request is valid");
        let request = Request::from(request).with_app(self.app.clone());
        let dispatch = self.routes.dispatch(request);
        TestResponse::new(self.runtime.block_on(self.app.scope(dispatch)))
    }
}
