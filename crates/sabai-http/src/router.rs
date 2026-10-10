use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt;
use std::future::ready;

use sabai_core::contracts::BoxFuture;
use sabai_core::{Error, IntoResponse, Method, Request, Response, Result, StatusCode};

use crate::handler::{BoxedHandler, Handler, boxed};

/// Collects routes: `Router::new().get("/posts", index)`. Mistakes such as a duplicate
/// route are reported by [`Router::compile`] at boot, so building stays a plain chain.
#[derive(Default)]
pub struct Router {
    routes: Vec<Route>,
}

struct Route {
    method: Method,
    path: String,
    handler: BoxedHandler,
}

/// Compiled routes, ready to dispatch requests.
pub struct Routes {
    matcher: matchit::Router<usize>,
    endpoints: Vec<Endpoint>,
}

struct Endpoint {
    methods: Vec<(Method, BoxedHandler)>,
}

impl Router {
    /// An empty router.
    pub fn new() -> Self {
        Self::default()
    }

    /// Routes `GET path` to `handler`. Paths use `{id}` for parameters.
    pub fn get<H: Handler<Args>, Args: 'static>(self, path: &str, handler: H) -> Self {
        let handler = boxed(handler);
        self.push_route(Method::GET, path, handler)
    }

    fn push_route(mut self, method: Method, path: &str, handler: BoxedHandler) -> Self {
        self.routes.push(Route {
            method,
            path: path.to_owned(),
            handler,
        });
        self
    }

    /// Checks every route and builds the matcher; call once at boot.
    pub fn compile(self) -> Result<Routes> {
        let mut routes = Routes {
            matcher: matchit::Router::new(),
            endpoints: Vec::new(),
        };
        let mut endpoint_by_path = HashMap::new();
        for route in self.routes {
            let index = match endpoint_by_path.get(&route.path) {
                Some(&index) => index,
                None => {
                    let index = routes.add_endpoint(&route)?;
                    endpoint_by_path.insert(route.path.clone(), index);
                    index
                }
            };
            routes.add_method(index, route)?;
        }
        Ok(routes)
    }
}

impl Routes {
    /// Runs the handler for `request`, or responds `404` or `405`.
    pub fn dispatch(&self, request: Request) -> BoxFuture<'static, Response> {
        let Ok(matched) = self.matcher.at(request.path()) else {
            return respond(Error::not_found());
        };
        let endpoint = &self.endpoints[*matched.value];
        match endpoint
            .methods
            .iter()
            .find(|(method, _)| method == request.method())
        {
            Some((_, handler)) => handler.call_erased(request),
            None => respond(Error::from_status(StatusCode::METHOD_NOT_ALLOWED)),
        }
    }

    fn add_endpoint(&mut self, route: &Route) -> Result<usize> {
        let index = self.endpoints.len();
        self.matcher
            .insert(route.path.as_str(), index)
            .map_err(|source| invalid(route, source.to_string()))?;
        self.endpoints.push(Endpoint {
            methods: Vec::new(),
        });
        Ok(index)
    }

    fn add_method(&mut self, index: usize, route: Route) -> Result<()> {
        let endpoint = &mut self.endpoints[index];
        if endpoint
            .methods
            .iter()
            .any(|(method, _)| *method == route.method)
        {
            return Err(invalid(
                &route,
                "it is defined twice; remove one of them".into(),
            ));
        }
        endpoint.methods.push((route.method, route.handler));
        Ok(())
    }
}

fn respond(error: Error) -> BoxFuture<'static, Response> {
    Box::pin(ready(error.into_response()))
}

#[derive(Debug)]
struct InvalidRoute {
    route: String,
    problem: String,
}

impl fmt::Display for InvalidRoute {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "route `{}` is invalid: {}", self.route, self.problem)
    }
}

impl StdError for InvalidRoute {}

fn invalid(route: &Route, problem: String) -> Error {
    InvalidRoute {
        route: format!("{} {}", route.method, route.path),
        problem,
    }
    .into()
}
