use std::any::type_name;
use std::fmt::Display;
use std::future::ready;
use std::ops::Deref;
use std::str::FromStr;
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, Request, Result, StatusCode, log};

/// A type a handler can take as an argument, built from the request before the handler runs,
/// like a type-hinted argument in a Laravel controller. Failing rejects the request with that error.
pub trait FromRequest: Sized + Send + 'static {
    /// Builds the argument from `request`.
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>>;
}

impl FromRequest for Request {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(Ok(request.clone())))
    }
}

/// A service from the app's container, like a type-hinted service in a Laravel controller method:
/// `clock: Service<dyn Clock>`, then `clock.now()`. A service that is not bound is a `500`.
pub struct Service<T: ?Sized>(pub Arc<T>);

impl<T: ?Sized + Send + Sync + 'static> FromRequest for Service<T> {
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(request.app().and_then(|app| app.get()).map(Service)))
    }
}

impl<T: ?Sized> Deref for Service<T> {
    type Target = T;

    fn deref(&self) -> &T {
        &self.0
    }
}

/// The route's parameter, parsed: `{id}` in `/posts/{id}` gives `Param(7)` for `/posts/7`.
/// A value that does not parse is a `404`, like Laravel's `whereNumber('id')`.
pub struct Param<T>(pub T);

impl<T> FromRequest for Param<T>
where
    T: FromStr + Send + 'static,
    T::Err: Display,
{
    fn from_request(request: &Request) -> BoxFuture<'_, Result<Self>> {
        Box::pin(ready(parse_param(request)))
    }
}

fn parse_param<T: FromStr>(request: &Request) -> Result<Param<T>>
where
    T::Err: Display,
{
    let (name, value) = only_param(request)?;
    match value.parse() {
        Ok(value) => Ok(Param(value)),
        Err(error) => Err(not_a_valid(
            name,
            value,
            type_name::<T>(),
            error.to_string(),
        )),
    }
}

fn only_param(request: &Request) -> Result<(&str, &str)> {
    match request.params() {
        [(name, value)] => Ok((name, value)),
        params => Err(
            Error::from_status(StatusCode::INTERNAL_SERVER_ERROR).context(format!(
                "`Param<T>` needs a route with exactly one parameter, but `{}` has {}; \
             read the others with `request.param(name)`",
                request.path(),
                params.len()
            )),
        ),
    }
}

fn not_a_valid(name: &str, value: &str, type_name: &str, reason: String) -> Error {
    log::debug!("route parameter {name} = {value:?} is not a valid {type_name}: {reason}");
    Error::not_found()
}

#[cfg(test)]
mod tests {
    use std::time::SystemTime;

    use pollster::block_on;
    use sabai_core::contracts::{Clock, SystemClock};
    use sabai_core::{App, Body, Config, Provider};

    use super::*;

    fn request_with(params: &[(&str, &str)]) -> Request {
        let request = http::Request::builder()
            .uri("/posts/7")
            .body(Body::empty())
            .unwrap();
        let params = params
            .iter()
            .map(|(name, value)| (name.to_string(), value.to_string()))
            .collect();
        Request::from(request).with_route_params(params)
    }

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
        let request = request_with(&[]).with_app(app);

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

    #[test]
    fn a_param_parses_into_its_type() {
        let Param(id) =
            block_on(Param::<u64>::from_request(&request_with(&[("id", "7")]))).unwrap();

        assert_eq!(id, 7);
    }

    #[test]
    fn a_param_that_does_not_parse_is_a_404() {
        let request = request_with(&[("id", "abc")]);

        let error = block_on(Param::<u64>::from_request(&request))
            .err()
            .unwrap();

        assert_eq!(error.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn a_route_with_two_params_explains_what_to_use_instead() {
        let request = request_with(&[("user", "1"), ("post", "2")]);

        let error = block_on(Param::<u64>::from_request(&request))
            .err()
            .unwrap();

        assert_eq!(error.status(), StatusCode::INTERNAL_SERVER_ERROR);
        assert!(error.to_string().contains("`/posts/7` has 2"), "{error}");
    }
}
