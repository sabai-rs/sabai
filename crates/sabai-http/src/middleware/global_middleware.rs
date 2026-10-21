use std::any::TypeId;
use std::sync::Arc;

use super::{BoxedMiddleware, ConvertEmptyStringsToNull, Middleware, TraceRequests, TrimStrings};

/// Changes to the middleware every request runs, given to [`Router::global_middleware`](crate::Router::global_middleware)
/// like the `$middleware` Laravel's `withMiddleware` gets. Sabai's defaults run unless removed.
#[derive(Default)]
pub struct GlobalMiddleware {
    appended: Vec<(TypeId, BoxedMiddleware)>,
    removed: Vec<TypeId>,
}

impl GlobalMiddleware {
    /// Runs `middleware` after the defaults, like `$middleware->append(...)`.
    pub fn append<M: Middleware>(&mut self, middleware: M) -> &mut Self {
        self.push(TypeId::of::<M>(), Arc::new(middleware))
    }

    /// Stops running `M`, such as a default you do not want, like `$middleware->remove(...)`:
    /// `middleware.remove::<ConvertEmptyStringsToNull>()`.
    pub fn remove<M: Middleware>(&mut self) -> &mut Self {
        self.removed.push(TypeId::of::<M>());
        self
    }

    fn push(&mut self, id: TypeId, middleware: BoxedMiddleware) -> &mut Self {
        self.appended.push((id, middleware));
        self
    }

    /// The defaults, then what the app appended, without what it removed.
    pub(crate) fn into_stack(self) -> Vec<BoxedMiddleware> {
        let removed = self.removed;
        defaults()
            .into_iter()
            .chain(self.appended)
            .filter(|(id, _)| !removed.contains(id))
            .map(|(_, middleware)| middleware)
            .collect()
    }

    /// Only what was appended, for a group's router, which never gets the defaults twice.
    pub(crate) fn into_appended(self) -> impl Iterator<Item = BoxedMiddleware> {
        self.appended.into_iter().map(|(_, middleware)| middleware)
    }
}

// Laravel's default global middleware that Sabai has so far, in Laravel's order, inside
// TraceRequests, which comes first so its span and log line cover everything else.
fn defaults() -> Vec<(TypeId, BoxedMiddleware)> {
    vec![
        (TypeId::of::<TraceRequests>(), Arc::new(TraceRequests)),
        (TypeId::of::<TrimStrings>(), Arc::new(TrimStrings)),
        (
            TypeId::of::<ConvertEmptyStringsToNull>(),
            Arc::new(ConvertEmptyStringsToNull),
        ),
    ]
}
