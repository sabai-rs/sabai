use std::fmt::Display;

use sabai_http::Routes;

/// A named route to request, like Laravel's `route(self::ROUTE, ['id' => 7])` in a test:
/// `route("posts.show").with("id", 7)`. The [`TestApp`](crate::TestApp) turns it into a URL.
pub fn route(name: &str) -> RouteUrl {
    RouteUrl {
        name: name.to_owned(),
        params: Vec::new(),
    }
}

/// A route name and its params, made by [`route`].
pub struct RouteUrl {
    name: String,
    params: Vec<(String, String)>,
}

impl RouteUrl {
    /// Adds a param; one the path does not use goes to the query string.
    pub fn with(mut self, key: &str, value: impl Display) -> Self {
        self.params.push((key.to_owned(), value.to_string()));
        self
    }
}

/// Where a test request goes: a [`route`] or a plain URL such as `"/posts?page=2"`.
pub trait ToUri {
    /// The URL to request.
    fn to_uri(self, routes: &Routes) -> String;
}

impl ToUri for RouteUrl {
    fn to_uri(self, routes: &Routes) -> String {
        let params: Vec<_> = self
            .params
            .iter()
            .map(|(key, value)| (key.as_str(), value))
            .collect();
        routes.route(&self.name, &params).expect("the route exists")
    }
}

impl ToUri for &str {
    fn to_uri(self, _routes: &Routes) -> String {
        self.to_owned()
    }
}
