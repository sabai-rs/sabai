use sabai::http::{Router, Routes};

async fn page() -> &'static str {
    "page"
}

fn routes() -> Routes {
    let mut router = Router::new();
    router.get("/posts/{id}", page).name("posts.show");
    router.get("/files/{*path}", page).name("files.show");
    router.prefix("/api").group(|route| {
        route.get("/posts/{id}", page).name("api.posts.show");
    });
    router.compile().unwrap()
}

#[test]
fn a_named_route_builds_its_url_with_extra_params_as_the_query_string() {
    let routes = routes();

    assert_eq!(
        routes.route("posts.show", &[("id", 7)]).unwrap(),
        "/posts/7"
    );
    assert_eq!(
        routes
            .route(
                "api.posts.show",
                &[("id", "7"), ("page", "2"), ("sort", "new")]
            )
            .unwrap(),
        "/api/posts/7?page=2&sort=new"
    );
}

#[test]
fn values_are_percent_encoded_so_they_cannot_change_the_path() {
    let routes = routes();

    assert_eq!(
        routes
            .route("posts.show", &[("id", "a b/../c?d#e")])
            .unwrap(),
        "/posts/a%20b%2F..%2Fc%3Fd%23e"
    );
    assert_eq!(
        routes
            .route("files.show", &[("path", "docs/ไทย.txt")])
            .unwrap(),
        "/files/docs/%E0%B9%84%E0%B8%97%E0%B8%A2.txt"
    );
    assert_eq!(
        routes
            .route("posts.show", &[("id", "1"), ("q", "a&b=c")])
            .unwrap(),
        "/posts/1?q=a%26b%3Dc"
    );
}

#[test]
fn an_unknown_name_or_a_missing_param_says_how_to_fix_it() {
    let routes = routes();

    let unknown = routes
        .route("post.show", &[("id", 7)])
        .err()
        .unwrap()
        .to_string();
    let missing = routes
        .route::<&str>("posts.show", &[])
        .err()
        .unwrap()
        .to_string();

    assert!(
        unknown.contains("no route is named `post.show`"),
        "{unknown}"
    );
    assert!(
        missing.contains("route `posts.show` (/posts/{id}) needs `id`"),
        "{missing}"
    );
}

#[test]
fn two_routes_with_one_name_are_reported_at_boot() {
    let mut router = Router::new();
    router.get("/posts", page).name("posts.index");
    router.get("/articles", page).name("posts.index");

    let error = router.compile().err().unwrap().to_string();

    assert_eq!(
        error,
        "route `GET /articles` is invalid: the name `posts.index` is already used by another route"
    );
}
