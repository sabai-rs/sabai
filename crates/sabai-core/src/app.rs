use std::any::type_name;
use std::sync::Arc;

use crate::{Config, Container, Error, Result};

/// A package's or an app's entry point, like a Laravel service provider.
pub trait Provider: Send + Sync {
    /// Binds services. Runs for every provider before any `boot`, so do not resolve services here.
    fn register(&self, app: &mut App) -> Result<()>;

    /// Runs after every provider has registered, so services from other providers can be resolved.
    fn boot(&self, _app: &App) -> Result<()> {
        Ok(())
    }
}

/// The application: config plus the service container, filled by providers at boot.
#[derive(Debug)]
pub struct App {
    config: Config,
    container: Container,
}

/// Collects providers before boot; [`AppBuilder::boot`] turns it into an [`App`].
pub struct AppBuilder {
    app: App,
    providers: Vec<NamedProvider>,
}

struct NamedProvider {
    name: &'static str,
    provider: Box<dyn Provider>,
}

impl App {
    /// Starts an app from loaded config: `App::builder(Config::load(".")?).provider(...).boot()?`.
    pub fn builder(config: Config) -> AppBuilder {
        AppBuilder {
            app: App {
                config,
                container: Container::default(),
            },
            providers: Vec::new(),
        }
    }

    /// The app config.
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Binds a service; see [`Container::bind`].
    pub fn bind<T: ?Sized + Send + Sync + 'static>(&mut self, service: Arc<T>) {
        self.container.bind(service);
    }

    /// Resolves a service; see [`Container::get`].
    pub fn get<T: ?Sized + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.container.get()
    }
}

impl AppBuilder {
    /// Adds a provider. Providers register and boot in the order they are added.
    pub fn provider<P: Provider + 'static>(self, provider: P) -> Self {
        self.push_provider(type_name::<P>(), Box::new(provider))
    }

    fn push_provider(mut self, name: &'static str, provider: Box<dyn Provider>) -> Self {
        self.providers.push(NamedProvider { name, provider });
        self
    }

    /// Runs every provider's `register`, then every provider's `boot`, and returns the app,
    /// shared and read-only from here on.
    pub fn boot(self) -> Result<Arc<App>> {
        let Self { mut app, providers } = self;
        for NamedProvider { name, provider } in &providers {
            let failed =
                |error: Error| error.context(format!("provider `{name}` failed to register"));
            provider.register(&mut app).map_err(failed)?;
        }
        for NamedProvider { name, provider } in &providers {
            let failed = |error: Error| error.context(format!("provider `{name}` failed to boot"));
            provider.boot(&app).map_err(failed)?;
        }
        Ok(Arc::new(app))
    }
}
