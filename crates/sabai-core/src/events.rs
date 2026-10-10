use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::fmt;

use crate::{App, Error, Result};

/// Something that happened in the app, such as `OrderShipped`; listeners react to it.
pub trait Event: Any + Send + Sync {}

/// Reacts to an event `E`, like a Laravel listener class. Closures with this signature are listeners too.
pub trait Listener<E: Event>: Send + Sync + 'static {
    /// Runs when `E` is dispatched; an error stops the listeners after this one.
    fn handle(&self, event: &E, app: &App) -> Result<()>;
}

impl<E, F> Listener<E> for F
where
    E: Event,
    F: Fn(&E, &App) -> Result<()> + Send + Sync + 'static,
{
    fn handle(&self, event: &E, app: &App) -> Result<()> {
        self(event, app)
    }
}

// Queued listeners (M4-15) will be a second kind of erased listener stored here, so
// `listen` and `dispatch` keep the same signatures when queues arrive.
type ErasedListener = Box<dyn Fn(&dyn Any, &App) -> Result<()> + Send + Sync>;

struct RegisteredListener {
    name: &'static str,
    run: ErasedListener,
}

/// Listeners grouped by the event type they listen to.
#[derive(Default)]
pub(crate) struct Events {
    listeners: HashMap<TypeId, (&'static str, Vec<RegisteredListener>)>,
}

impl Events {
    pub(crate) fn listen<E: Event, L: Listener<E>>(&mut self, listener: L) {
        let run: ErasedListener = Box::new(move |event, app| match event.downcast_ref::<E>() {
            Some(event) => listener.handle(event, app),
            // Unreachable: listeners are stored under `TypeId::of::<E>()`, so only `E` reaches them.
            None => Ok(()),
        });
        let registered = RegisteredListener {
            name: type_name::<L>(),
            run,
        };
        self.push(TypeId::of::<E>(), type_name::<E>(), registered);
    }

    fn push(&mut self, event: TypeId, event_name: &'static str, listener: RegisteredListener) {
        let (_, listeners) = self
            .listeners
            .entry(event)
            .or_insert_with(|| (event_name, Vec::new()));
        listeners.push(listener);
    }

    pub(crate) fn dispatch(&self, event_type: TypeId, event: &dyn Any, app: &App) -> Result<()> {
        let Some((event_name, listeners)) = self.listeners.get(&event_type) else {
            return Ok(());
        };
        for listener in listeners {
            (listener.run)(event, app).map_err(|error: Error| {
                error.context(format!(
                    "listener `{}` for `{event_name}` failed",
                    listener.name
                ))
            })?;
        }
        Ok(())
    }
}

impl fmt::Debug for Events {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let counts = self
            .listeners
            .values()
            .map(|(event, listeners)| (event, listeners.len()));
        f.debug_map().entries(counts).finish()
    }
}
