use sabai::{App, Provider, Result};

/// Binds the app's services and adds its commands, like Laravel's `AppServiceProvider`.
pub struct AppServiceProvider;

impl Provider for AppServiceProvider {
    fn register(&self, _app: &mut App) -> Result<()> {
        Ok(())
    }
}
