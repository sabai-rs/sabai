use sabai::http::Router;

pub fn routes(route: &mut Router) {
    route.get("/", || async { "Welcome to Sabai" });
}
