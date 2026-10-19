use std::sync::Arc;

use sabai::contracts::{Clock, SystemClock};
use sabai::{App, Provider, Result};

use crate::app::console::Inspire;

/// Binds the blog's services and adds its commands, like Laravel's `AppServiceProvider`.
pub struct AppServiceProvider;

impl Provider for AppServiceProvider {
    fn register(&self, app: &mut App) -> Result<()> {
        app.bind::<dyn Clock>(Arc::new(SystemClock));
        app.command(Inspire)
    }
}
