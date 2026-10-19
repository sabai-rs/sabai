use std::sync::{Arc, Mutex, PoisonError};

use sabai_core::{App, Command, Error, Result, StatusCode};

use crate::Router;

/// `serve`: serves the app's routes, like `php artisan serve`.
pub struct ServeCommand {
    // The server takes the router, so it is handed out once.
    router: Mutex<Option<Router>>,
}

impl ServeCommand {
    /// A `serve` command for `router`.
    pub fn new(router: Router) -> Self {
        Self {
            router: Mutex::new(Some(router)),
        }
    }
}

impl Command for ServeCommand {
    fn name(&self) -> &str {
        "serve"
    }

    fn description(&self) -> &str {
        "Serve the app on app.host:app.port (the default)"
    }

    fn handle(&self, app: &Arc<App>, _args: &[String]) -> Result<()> {
        let router = self
            .router
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .take()
            .ok_or_else(|| {
                Error::new(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "`serve` already ran in this process",
                )
            })?;
        crate::serve(Arc::clone(app), router)
    }
}
