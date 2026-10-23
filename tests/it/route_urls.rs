use sabai::http::Router;

async fn page() -> &'static str {
    "page"
}

#[test]
fn a_named_route_builds_its_url() {
    let mut router = Router::new();
    router.get("/posts/{id}", page).name("posts.show");
    let routes = router.compile().unwrap();

    let url = routes.route("posts.show", &[("id", 7)]).unwrap();

    assert_eq!(url, "/posts/7");
}

#[test]
fn params_the_path_does_not_use_become_the_query_string() {
    let mut router = Router::new();
    router.prefix("/api").group(|route| {
        route.get("/posts/{id}", page).name("api.posts.show");
    });
    let routes = router.compile().unwrap();

    let url = routes
        .route(
            "api.posts.show",
            &[("id", "7"), ("page", "2"), ("sort", "new")],
        )
        .unwrap();

    assert_eq!(url, "/api/posts/7?page=2&sort=new");
}

#[test]
fn a_value_is_percent_encoded_so_it_cannot_change_the_path() {
    let mut router = Router::new();
    router.get("/posts/{id}", page).name("posts.show");
    let routes = router.compile().unwrap();

    let url = routes
        .route("posts.show", &[("id", "a b/../c?d#e")])
        .unwrap();

    assert_eq!(url, "/posts/a%20b%2F..%2Fc%3Fd%23e");
}

#[test]
fn a_wildcard_keeps_its_slashes_and_encodes_the_rest() {
    let mut router = Router::new();
    router.get("/files/{*path}", page).name("files.show");
    let routes = router.compile().unwrap();

    let url = routes
        .route("files.show", &[("path", "docs/ไทย.txt")])
        .unwrap();

    assert_eq!(url, "/files/docs/%E0%B9%84%E0%B8%97%E0%B8%A2.txt");
}

#[test]
fn a_query_string_value_is_percent_encoded_too() {
    let mut router = Router::new();
    router.get("/posts/{id}", page).name("posts.show");
    let routes = router.compile().unwrap();

    let url = routes
        .route("posts.show", &[("id", "1"), ("q", "a&b=c")])
        .unwrap();

    assert_eq!(url, "/posts/1?q=a%26b%3Dc");
}

#[test]
fn an_unknown_name_says_so() {
    let mut router = Router::new();
    router.get("/posts/{id}", page).name("posts.show");
    let routes = router.compile().unwrap();

    let error = routes
        .route("post.show", &[("id", 7)])
        .err()
        .unwrap()
        .to_string();

    assert!(error.contains("no route is named `post.show`"), "{error}");
}

#[test]
fn a_missing_param_names_the_route_and_the_param() {
    let mut router = Router::new();
    router.get("/posts/{id}", page).name("posts.show");
    let routes = router.compile().unwrap();

    let error = routes
        .route::<&str>("posts.show", &[])
        .err()
        .unwrap()
        .to_string();

    assert!(
        error.contains("route `posts.show` (/posts/{id}) needs `id`"),
        "{error}"
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
