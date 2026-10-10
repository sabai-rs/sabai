use std::sync::Arc;

use sabai::contracts::{Clock, SystemClock};
use sabai::{App, Provider, Result};

/// Binds the blog's services, like Laravel's `AppServiceProvider`.
pub struct AppServiceProvider;

impl Provider for AppServiceProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.bind::<dyn Clock>(Arc::new(SystemClock));
        Ok(())
    }
}
