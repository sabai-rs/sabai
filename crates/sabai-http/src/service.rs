use std::future::ready;
use std::ops::Deref;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, Request, Result};

use crate::FromRequest;

/// A service from the app's container, like a type-hinted service in a Laravel controller method:
/// `clock: Service<dyn Clock>`, then `clock.now()`. A service that is not bound is a `500`.
pub struct Service<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized + Send + Sync + 'static> FromRequest for Service<T> {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(request.app().and_then(|app| app.get()).map(Service)))
    }

    fn prepare(app: &App) -> Result<()> {
        app.get::<T>().map(drop)
    }
}

impl<T: ?Sized> Deref for Service<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use pollster::block_on;
    use sabai_core::contracts::{Clock, SystemClock};
    use sabai_core::{Body, Config, Provider};

    use super::*;

    struct ClockProvider;

    impl Provider for ClockProvider {
        fn register(&self, app: &mut App) -> Result<()> {
            app.bind::<dyn Clock>(Arc::new(SystemClock));
            Ok(())
        }
    }

    #[test]
    fn a_service_comes_from_the_app_and_a_missing_one_says_how_to_bind_it() {
        let app = App::builder(Config::default())
            .provider(ClockProvider)
            .boot()
            .unwrap();
        let request = Request::from(http::Request::new(Body::empty())).with_app(app);

        let clock = block_on(Service::<dyn Clock>::from_request(&request)).unwrap();
        let missing = block_on(Service::<String>::from_request(&request))
            .err()
            .unwrap();

        assert!(clock.now() > SystemTime::UNIX_EPOCH);
        assert!(
            missing.to_string().contains("bind it in a provider"),
            "{missing}"
        );
    }
}
