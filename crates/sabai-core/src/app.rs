use std::sync::Arc;

use crate::{Config, Container, Result};

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
    providers: Vec<Box<dyn Provider>>,
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
    pub fn provider(self, provider: impl Provider + 'static) -> Self {
        self.push_provider(Box::new(provider))
    }

    fn push_provider(mut self, provider: Box<dyn Provider>) -> Self {
        self.providers.push(provider);
        self
    }

    /// Runs every provider's `register`, then every provider's `boot`, and returns the app,
    /// shared and read-only from here on.
    pub fn boot(self) -> Result<Arc<App>> {
        let Self { mut app, providers } = self;
        for provider in &providers {
            provider.register(&mut app)?;
        }
        for provider in &providers {
            provider.boot(&app)?;
        }
        Ok(Arc::new(app))
    }
}
