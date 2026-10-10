use std::any::{Any, TypeId, type_name};
use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::sync::Arc;

use crate::{Error, Result};

// Holds an `Arc<T>`. Storing the `Arc` itself, not `T`, is what lets `T` be a trait
// object such as `dyn Mailer`: `Arc<dyn Mailer>` has a known size, `dyn Mailer` does not.
type BoxedService = Box<dyn Any + Send + Sync>;

struct Binding {
    name: &'static str,
    service: BoxedService,
}

/// Shared services looked up by type or by trait, like Laravel's `app()`: filled at boot, read-only after.
#[derive(Default)]
pub struct Container {
    bindings: HashMap<TypeId, Binding>,
}

impl Container {
    /// Registers `service` under `T`, replacing any earlier binding; `T` may be a trait: `bind::<dyn Mailer>(...)`.
    pub fn bind<T: ?Sized + Send + Sync + 'static>(&mut self, service: Arc<T>) {
        self.insert(TypeId::of::<T>(), type_name::<T>(), Box::new(service));
    }

    /// The service bound to `T`, or an error that says how to bind it.
    pub fn get<T: ?Sized + Send + Sync + 'static>(&self) -> Result<Arc<T>> {
        self.service(TypeId::of::<T>())
            .and_then(|service| service.downcast_ref::<Arc<T>>())
            .cloned()
            .ok_or_else(|| not_bound(type_name::<T>()))
    }

    fn insert(&mut self, id: TypeId, name: &'static str, service: BoxedService) {
        self.bindings.insert(id, Binding { name, service });
    }

    fn service(&self, id: TypeId) -> Option<&BoxedService> {
        self.bindings.get(&id).map(|binding| &binding.service)
    }
}

impl fmt::Debug for Container {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let names = self.bindings.values().map(|binding| binding.name);
        f.debug_set().entries(names).finish()
    }
}

#[derive(Debug)]
struct NotBound {
    name: &'static str,
}

impl fmt::Display for NotBound {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name;
        write!(
            f,
            "no service is bound to `{name}`; bind it in a provider: \
             `container.bind::<{name}>(Arc::new(...))`"
        )
    }
}

impl StdError for NotBound {}

fn not_bound(name: &'static str) -> Error {
    NotBound { name }.into()
}

#[cfg(test)]
mod tests {
    use super::*;

    trait Mailer: Send + Sync {
        fn transport(&self) -> &'static str;
    }

    struct SmtpMailer;
    struct FakeMailer;

    impl Mailer for SmtpMailer {
        fn transport(&self) -> &'static str {
            "smtp"
        }
    }

    impl Mailer for FakeMailer {
        fn transport(&self) -> &'static str {
            "fake"
        }
    }

    #[test]
    fn a_concrete_service_is_shared_not_copied() {
        let mut container = Container::default();
        let name = Arc::new(String::from("Blog"));
        container.bind(name.clone());

        let resolved = container.get::<String>().unwrap();

        assert!(Arc::ptr_eq(&name, &resolved));
    }

    #[test]
    fn a_trait_binding_can_be_swapped_for_a_fake() {
        let mut container = Container::default();
        container.bind::<dyn Mailer>(Arc::new(SmtpMailer));
        container.bind::<dyn Mailer>(Arc::new(FakeMailer));

        assert_eq!(container.get::<dyn Mailer>().unwrap().transport(), "fake");
    }

    #[test]
    fn a_missing_binding_says_how_to_bind_it() {
        let error = Container::default().get::<dyn Mailer>().err().unwrap();

        assert!(
            error
                .to_string()
                .starts_with("no service is bound to `dyn "),
            "{error}"
        );
        assert!(error.to_string().contains("container.bind::<dyn "));
    }

    #[test]
    fn debug_lists_what_is_bound() {
        let mut container = Container::default();
        container.bind(Arc::new(42_u16));

        assert_eq!(format!("{container:?}"), r#"{"u16"}"#);
    }

    #[test]
    fn the_container_can_be_shared_across_threads() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<Container>();
    }
}
