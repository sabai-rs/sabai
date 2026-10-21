use std::collections::HashMap;
use std::error::Error as StdError;
use std::fmt::{self, Display};
use std::sync::Arc;

use sabai_core::contracts::BoxFuture;
use sabai_core::{App, Error, Method, Request, Response, Result, StatusCode};

use crate::handler::{BoxedHandler, Handler, boxed};
use crate::middleware::{BoxedMiddleware, GlobalMiddleware, Middleware, Pipeline};
use crate::route_urls::RouteUrls;

/// Collects routes, one statement each like Laravel's `routes/web.php`:
/// `route.get("/posts", index);`. Mistakes such as a duplicate route are reported by
/// [`Router::compile`] at boot, so declaring routes needs no `?` on every line.
#[derive(Default)]
pub struct Router {
    routes: Vec<Route>,
    global_middleware: GlobalMiddleware,
}

/// One route, returned by [`Router::get`] and friends so it can be named: `.name("posts.show")`.
pub struct Route {
    method: Method,
    path: String,
    handler: BoxedHandler,
    name: Option<String>,
    middleware: Vec<BoxedMiddleware>,
}

/// Routes that share a prefix or middleware, started by [`Router::prefix`] or [`Router::middleware`]
/// like Laravel's `Route::prefix('api')` and `Route::middleware('auth')`.
pub struct RouteGroup<'a> {
    router: &'a mut Router,
    prefix: String,
    middleware: Vec<BoxedMiddleware>,
}

/// Compiled routes, ready to dispatch requests.
pub struct Routes {
    matcher: matchit::Router<usize>,
    endpoints: Vec<Endpoint>,
    urls: RouteUrls,
    middleware: Arc<[BoxedMiddleware]>,
    not_found: Pipeline,
    method_not_allowed: Pipeline,
}

struct Endpoint {
    path: String,
    methods: Vec<(Method, Pipeline)>,
}

impl Router {
    /// An empty router.
    pub fn new() -> Self {
        Self::default()
    }

    /// Routes `GET path` to `handler`. Paths use `{id}` for parameters, read with `request.param("id")`.
    pub fn get<H: Handler<Args>, Args: 'static>(&mut self, path: &str, handler: H) -> &mut Route {
        let handler = boxed(handler);
        self.push_route(Method::GET, path, handler)
    }

    /// Routes `POST path` to `handler`.
    pub fn post<H: Handler<Args>, Args: 'static>(&mut self, path: &str, handler: H) -> &mut Route {
        let handler = boxed(handler);
        self.push_route(Method::POST, path, handler)
    }

    /// Routes `PUT path` to `handler`.
    pub fn put<H: Handler<Args>, Args: 'static>(&mut self, path: &str, handler: H) -> &mut Route {
        let handler = boxed(handler);
        self.push_route(Method::PUT, path, handler)
    }

    /// Routes `PATCH path` to `handler`.
    pub fn patch<H: Handler<Args>, Args: 'static>(&mut self, path: &str, handler: H) -> &mut Route {
        let handler = boxed(handler);
        self.push_route(Method::PATCH, path, handler)
    }

    /// Routes `DELETE path` to `handler`.
    pub fn delete<H: Handler<Args>, Args: 'static>(
        &mut self,
        path: &str,
        handler: H,
    ) -> &mut Route {
        let handler = boxed(handler);
        self.push_route(Method::DELETE, path, handler)
    }

    /// Starts a group whose routes all begin with `prefix`:
    /// `route.prefix("/api").group(|route| { route.get("/health", health); });`
    pub fn prefix(&mut self, prefix: &str) -> RouteGroup<'_> {
        RouteGroup {
            router: self,
            prefix: prefix.to_owned(),
            middleware: Vec::new(),
        }
    }

    /// Starts a group whose routes all run `middleware`, like Laravel's `Route::middleware(...)`:
    /// `route.middleware(Auth).group(|route| { route.get("/profile", profile); });`
    pub fn middleware<M: Middleware>(&mut self, middleware: M) -> RouteGroup<'_> {
        self.prefix("").middleware(middleware)
    }

    /// Changes the middleware every request runs, 404s and 405s included, like `withMiddleware`
    /// in Laravel's `bootstrap/app.php`. Sabai's defaults, `TrimStrings` and
    /// `ConvertEmptyStringsToNull`, run unless removed:
    /// `route.global_middleware(|middleware| { middleware.append(FrameGuard); });`
    pub fn global_middleware<F: FnOnce(&mut GlobalMiddleware)>(
        &mut self,
        configure: F,
    ) -> &mut Self {
        configure(&mut self.global_middleware);
        self
    }

    fn mount_group(&mut self, prefix: &str, mut middleware: Vec<BoxedMiddleware>, group: Router) {
        // Global middleware appended inside a group still runs, but only on the group's routes.
        middleware.extend(group.global_middleware.into_appended());
        let routes = group
            .routes
            .into_iter()
            .map(|route| route.prefixed(prefix).behind(&middleware));
        self.routes.extend(routes);
    }

    pub(crate) fn push_route(
        &mut self,
        method: Method,
        path: &str,
        handler: BoxedHandler,
    ) -> &mut Route {
        let index = self.routes.len();
        self.routes.push(Route {
            method,
            path: path.to_owned(),
            handler,
            name: None,
            middleware: Vec::new(),
        });
        &mut self.routes[index]
    }

    /// Checks every route and builds the matcher; call once at boot.
    pub fn compile(self) -> Result<Routes> {
        let middleware: Arc<[BoxedMiddleware]> = self.global_middleware.into_stack().into();
        let mut routes = Routes {
            matcher: matchit::Router::new(),
            endpoints: Vec::new(),
            urls: RouteUrls::default(),
            not_found: Pipeline::new(middleware.clone(), boxed(not_found)),
            method_not_allowed: Pipeline::new(middleware.clone(), boxed(method_not_allowed)),
            middleware,
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
            routes.add_name(&route)?;
            routes.add_method(index, route)?;
        }
        Ok(routes)
    }
}

impl RouteGroup<'_> {
    /// Runs `middleware` on the group's routes, after the middleware added before it.
    pub fn middleware<M: Middleware>(self, middleware: M) -> Self {
        self.push_middleware(Arc::new(middleware))
    }

    fn push_middleware(mut self, middleware: BoxedMiddleware) -> Self {
        self.middleware.push(middleware);
        self
    }

    /// Adds the routes declared in `routes`, a closure or a function such as `api::routes`.
    pub fn group<F: FnOnce(&mut Router)>(self, routes: F) {
        let mut group = Router::new();
        routes(&mut group);
        self.router
            .mount_group(&self.prefix, self.middleware, group);
    }
}

impl Route {
    /// Names the route, like Laravel's `->name('posts.show')`, so URLs can be built from the name.
    pub fn name(&mut self, name: &str) -> &mut Self {
        self.name = Some(name.to_owned());
        self
    }

    /// Runs `middleware` on this route only, after its groups' middleware, like Laravel's `->middleware(...)`.
    pub fn middleware<M: Middleware>(&mut self, middleware: M) -> &mut Self {
        self.push_middleware(Arc::new(middleware))
    }

    fn push_middleware(&mut self, middleware: BoxedMiddleware) -> &mut Self {
        self.middleware.push(middleware);
        self
    }

    // Like Laravel, `prefix("api")` and `prefix("/api/")` mean the same, and `/` inside the group is `/api`.
    fn prefixed(self, prefix: &str) -> Self {
        let prefix = prefix.trim_matches('/');
        let path = self.path.trim_start_matches('/');
        let path = match (prefix.is_empty(), path.is_empty()) {
            (true, _) => format!("/{path}"),
            (false, true) => format!("/{prefix}"),
            (false, false) => format!("/{prefix}/{path}"),
        };
        Self { path, ..self }
    }

    // A group's middleware runs before the middleware of groups nested inside it.
    fn behind(mut self, group_middleware: &[BoxedMiddleware]) -> Self {
        let mut middleware = group_middleware.to_vec();
        middleware.append(&mut self.middleware);
        Self { middleware, ..self }
    }
}

impl Routes {
    /// Checks every route's arguments against `app` once, before serving: a service that is
    /// not bound fails here with the route's name instead of on a request. `serve` calls this.
    pub fn prepare(&self, app: &App) -> Result<()> {
        for endpoint in &self.endpoints {
            for (method, pipeline) in &endpoint.methods {
                let path = &endpoint.path;
                pipeline.handler().prepare(app).map_err(|error| {
                    error.context(format!("route `{method} {path}` cannot start"))
                })?;
            }
        }
        Ok(())
    }

    /// The URL of the route named `name`, like Laravel's `route('posts.show', ['id' => 7])`:
    /// `route("posts.show", &[("id", 7)])` gives `/posts/7`. Params the path does not use
    /// become the query string, and every value is percent-encoded.
    pub fn route<V: Display>(&self, name: &str, params: &[(&str, V)]) -> Result<String> {
        let params: Vec<_> = params
            .iter()
            .map(|(key, value)| (*key, value.to_string()))
            .collect();
        self.urls.route(name, &params)
    }

    /// Runs the handler for `request`, or responds `404` or `405`.
    pub fn dispatch(&self, request: Request) -> BoxFuture<'static, Response> {
        let Ok(matched) = self.matcher.at(request.path()) else {
            return self.not_found.clone().run(request);
        };
        let endpoint = &self.endpoints[*matched.value];
        let params: Vec<_> = matched
            .params
            .iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect();
        let request = request.with_route_params(params);
        match endpoint
            .methods
            .iter()
            .find(|(method, _)| method == request.method())
        {
            Some((_, pipeline)) => pipeline.clone().run(request),
            None => self.method_not_allowed.clone().run(request),
        }
    }

    fn add_endpoint(&mut self, route: &Route) -> Result<usize> {
        let index = self.endpoints.len();
        self.matcher
            .insert(route.path.as_str(), index)
            .map_err(|source| invalid(route, source.to_string()))?;
        self.endpoints.push(Endpoint {
            path: route.path.clone(),
            methods: Vec::new(),
        });
        Ok(index)
    }

    fn add_name(&mut self, route: &Route) -> Result<()> {
        match &route.name {
            Some(name) if !self.urls.insert(name, &route.path) => Err(invalid(
                route,
                format!("the name `{name}` is already used by another route"),
            )),
            _ => Ok(()),
        }
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
        let mut middleware = self.middleware.to_vec();
        middleware.extend(route.middleware);
        let pipeline = Pipeline::new(middleware.into(), route.handler);
        endpoint.methods.push((route.method, pipeline));
        Ok(())
    }
}

async fn not_found() -> Error {
    Error::not_found()
}

async fn method_not_allowed() -> Error {
    Error::from_status(StatusCode::METHOD_NOT_ALLOWED)
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
