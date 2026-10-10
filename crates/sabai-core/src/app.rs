use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, PoisonError, RwLock};

use crate::events::Events;
use crate::{Config, Container, Error, Event, Listener, Result};

/// A package's or an app's entry point, like a Laravel service provider.
pub trait Provider: Send + Sync {
    /// Binds services. Runs for every provider before any `boot`, so do not resolve services here.
    fn register(&self, app: &mut App) -> Result<()>;

    /// Runs after every provider has registered, so services from other providers can be resolved.
    fn boot(&self, _app: &App) -> Result<()> {
        Ok(())
    }
}

/// A type the app builds from its services, like a class Laravel resolves by autowiring:
/// controllers, services and repositories take what they need in `new`.
///
/// ```ignore
/// impl FromApp for PostService {
///     fn new(app: &App) -> Result<Self> {
///         Ok(Self { posts: app.get()?, clock: app.get()? })
///     }
/// }
/// ```
pub trait FromApp: Sized {
    /// Builds `Self`, like a Laravel constructor whose arguments come from the container.
    fn new(app: &App) -> Result<Self>;
}

/// The application: config, the service container and event listeners, filled by providers at boot.
#[derive(Debug)]
pub struct App {
    config: Config,
    container: Container,
    events: Events,
    shared: Shared,
}

/// Instances built by [`App::make_shared`], kept for the life of the app.
#[derive(Default)]
struct Shared(RwLock<HashMap<TypeId, Arc<dyn Any + Send + Sync>>>);

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
                events: Events::default(),
                shared: Shared::default(),
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

    /// Adds a listener for `E`, run in the order added: a closure `|event: &E, app: &App|` or a [`Listener`].
    pub fn listen<E: Event, L: Listener<E>>(&mut self, listener: L) {
        self.events.listen(listener);
    }

    /// Runs every listener for `event`, stopping at the first error; no listeners is fine.
    pub fn dispatch<E: Event>(&self, event: &E) -> Result<()> {
        self.events.dispatch(TypeId::of::<E>(), event, self)
    }

    /// Resolves a service; see [`Container::get`].
    pub fn get<T: ?Sized + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.container.get()
    }

    /// Builds a new `T` from the app's services, like Laravel's `app()->make(PostService::class)`.
    pub fn make<T: FromApp>(&self) -> Result<T> {
        T::new(self)
            .map_err(|error| error.context(format!("could not make `{}`", type_name::<T>())))
    }

    /// Makes `T` the first time and returns that same instance after, like a Laravel singleton.
    pub fn make_shared<T: FromApp + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        let id = TypeId::of::<T>();
        if let Some(made) = self.shared.find(id).and_then(|made| made.downcast().ok()) {
            return Ok(made);
        }
        // Made outside the lock, so `T::new` can itself call `make_shared` without a deadlock.
        let made = Arc::new(self.make::<T>()?);
        Ok(self
            .shared
            .keep(id, made.clone())
            .downcast()
            .unwrap_or(made))
    }
}

impl Shared {
    // The lock is only held for one map lookup or insert, so even a poisoned lock holds a whole map.
    fn find(&self, id: TypeId) -> Option<Arc<dyn Any + Send + Sync>> {
        let shared = self.0.read().unwrap_or_else(PoisonError::into_inner);
        shared.get(&id).cloned()
    }

    // If two threads make `T` at once, both get the first one kept.
    fn keep(&self, id: TypeId, made: Arc<dyn Any + Send + Sync>) -> Arc<dyn Any + Send + Sync> {
        let mut shared = self.0.write().unwrap_or_else(PoisonError::into_inner);
        shared.entry(id).or_insert(made).clone()
    }
}

impl fmt::Debug for Shared {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Shared").finish_non_exhaustive()
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
