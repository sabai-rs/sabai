use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use std::{fs, process};

use sabai::contracts::Clock;
use sabai::orm::{
    Database, DatabaseManager, DatabaseServiceProvider, Migration, Migrations, Model, Schema,
};
use sabai::{App, Config, DateTime};
use sabai::{Provider, StatusCode};

#[derive(Model, Debug, PartialEq)]
struct Post {
    id: u64,
    title: String,
    body: Option<String>,
}

#[derive(Model, Debug, PartialEq)]
struct Meeting {
    id: u64,
    starts_at: DateTime,
    ended_at: Option<DateTime>,
}

#[derive(Model, Debug, PartialEq)]
struct Comment {
    id: u64,
    body: String,
    created_at: Option<DateTime>,
    updated_at: Option<DateTime>,
}

/// A clock that only moves when told to, like Laravel's `travel()`.
struct FrozenClock(Mutex<SystemTime>);

impl Clock for FrozenClock {
    fn now(&self) -> SystemTime {
        *self.0.lock().unwrap()
    }
}

struct FrozenClockProvider(Arc<FrozenClock>);

impl Provider for FrozenClockProvider {
    fn register(&self, app: &mut App) -> sabai::Result<()> {
        app.bind::<dyn Clock>(self.0.clone());
        Ok(())
    }
}

#[derive(Model)]
#[model(table = "people")]
struct Person {
    id: u64,
}

#[derive(Model, Debug, PartialEq)]
#[model(connection = "report")]
struct Sale {
    id: u64,
    amount: i64,
}

#[derive(Model, Debug)]
#[model(connection = "nope")]
struct Lost {
    id: u64,
}

const ONE_CONNECTION: &str =
    "default = \"sqlite\"\n[connections.sqlite]\ndriver = \"sqlite\"\ndatabase = \":memory:\"\n";

const TWO_CONNECTIONS: &str = "default = \"sqlite\"\n\
    [connections.sqlite]\ndriver = \"sqlite\"\ndatabase = \":memory:\"\n\
    [connections.report]\ndriver = \"sqlite\"\ndatabase = \":memory:\"\n";

fn app(name: &str) -> Arc<App> {
    app_with(name, ONE_CONNECTION)
}

fn app_with(name: &str, database_toml: &str) -> Arc<App> {
    let base = std::env::temp_dir().join(format!("sabai-it-model-{name}-{}", process::id()));
    fs::create_dir_all(base.join("config")).unwrap();
    fs::write(base.join("config/database.toml"), database_toml).unwrap();
    let config = Config::load(&base).unwrap();
    App::builder(config)
        .provider(DatabaseServiceProvider)
        .boot()
        .unwrap()
}

fn run<T>(app: &Arc<App>, statements: &[&str], query: impl Future<Output = T>) -> T {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    runtime.block_on(app.scope(async {
        let database = App::current().unwrap().get::<Database>().unwrap();
        for sql in statements {
            database.statement(sql).await.unwrap();
        }
        query.await
    }))
}

const CREATE_POSTS: &str =
    "create table posts (id integer primary key, title varchar not null, body text)";

#[test]
fn the_derive_names_the_table_in_plural_snake_case_and_lists_the_fields() {
    assert_eq!(Post::TABLE, "posts");
    assert_eq!(Post::COLUMNS, ["id", "title", "body"]);
}

#[test]
fn an_irregular_plural_names_its_table() {
    let people = run(
        &app("people"),
        &[
            "create table people (id integer primary key)",
            "insert into people (id) values (7)",
        ],
        Person::all(),
    )
    .unwrap();

    assert_eq!(Person::TABLE, "people");
    assert_eq!(
        people.iter().map(|person| person.id).collect::<Vec<_>>(),
        [7]
    );
}

#[test]
fn all_reads_every_row_into_the_model() {
    let posts = run(
        &app("all"),
        &[
            CREATE_POSTS,
            "insert into posts (id, title, body) values (1, 'First', 'a'), (2, 'Second', null)",
        ],
        Post::all(),
    )
    .unwrap();

    assert_eq!(
        posts,
        [
            Post {
                id: 1,
                title: "First".into(),
                body: Some("a".into())
            },
            Post {
                id: 2,
                title: "Second".into(),
                body: None
            },
        ]
    );
}

#[test]
fn a_negative_id_cannot_become_a_u64() {
    let error = run(
        &app("negative"),
        &[
            CREATE_POSTS,
            "insert into posts (id, title) values (-1, 'Broken')",
        ],
        Post::all(),
    )
    .err()
    .unwrap()
    .to_string();

    assert!(error.starts_with("could not read column `id`"), "{error}");
}

#[test]
fn save_inserts_a_new_row_fills_in_its_id_and_then_updates_it() {
    let (inserted, updated) = run(&app("save"), &[CREATE_POSTS], async {
        let mut post = Post {
            id: 0,
            title: "Draft".into(),
            body: None,
        };
        post.save().await.unwrap();
        let inserted = Post::find_or_fail(post.id).await.unwrap();
        post.title = "Published".into();
        post.save().await.unwrap();
        (inserted, Post::all().await.unwrap())
    });

    assert_eq!(
        inserted,
        Post {
            id: 1,
            title: "Draft".into(),
            body: None
        }
    );
    assert_eq!(
        updated,
        [Post {
            id: 1,
            title: "Published".into(),
            body: None
        }]
    );
}

#[test]
fn a_missing_row_is_none_for_find_and_a_404_for_find_or_fail() {
    let (found, error) = run(&app("missing"), &[CREATE_POSTS], async {
        (
            Post::find(9).await.unwrap(),
            Post::find_or_fail(9).await.err().unwrap(),
        )
    });

    assert_eq!(found, None);
    assert_eq!(error.status(), StatusCode::NOT_FOUND);
    assert_eq!(error.to_string(), "no `posts` row has id 9: Not Found");
}

struct CreateSalesTable;

impl Migration for CreateSalesTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("sales", |table| {
            table.id();
            table.integer("amount");
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("sales");
    }

    fn connection(&self) -> Option<&'static str> {
        Some("report")
    }
}

#[test]
fn a_model_on_another_connection_uses_that_database_only() {
    let migrations =
        Migrations::new().add("2026_10_17_000000_create_sales_table", CreateSalesTable);

    let (sales, on_default, recorded_on) = run(&app_with("report", TWO_CONNECTIONS), &[], async {
        let default = App::current().unwrap().get::<Database>().unwrap();
        default.migrate(&migrations).await.unwrap();
        Sale { id: 0, amount: 120 }.save().await.unwrap();
        let on_default = default.statement("select * from sales").await.is_ok();
        let report = App::current()
            .unwrap()
            .get::<DatabaseManager>()
            .unwrap()
            .connection("report")
            .unwrap();
        let recorded_on = report.statement("select * from migrations").await.is_ok();
        (Sale::all().await.unwrap(), on_default, recorded_on)
    });

    assert_eq!(sales, [Sale { id: 1, amount: 120 }]);
    assert!(
        !on_default,
        "the sales table must not exist on the default connection"
    );
    assert!(
        !recorded_on,
        "migrations are recorded on the default connection only"
    );
}

#[test]
fn a_connection_missing_from_config_says_where_to_add_it() {
    let error = run(&app("nope"), &[], Lost::all())
        .err()
        .unwrap()
        .to_string();

    assert!(
        error.contains("no database connection is named `nope`; add `[connections.nope]` to config/database.toml"),
        "{error}"
    );
}

struct CreatePostsTable;

impl Migration for CreatePostsTable {
    fn up(&self, schema: &mut Schema) {
        schema.create("posts", |table| {
            table.id();
            table.string("title");
            table.text("body").nullable();
        });
    }

    fn down(&self, schema: &mut Schema) {
        schema.drop_if_exists("posts");
    }
}

#[test]
fn a_query_after_migrate_in_the_same_process_reuses_the_pool() {
    let app = app("migrate-then-query");
    let migrations =
        Migrations::new().add("2026_10_19_000000_create_posts_table", CreatePostsTable);

    sabai::orm::migrate(&app, &migrations).unwrap();
    let posts = sabai::block_on(app.scope(Post::all())).unwrap();

    assert_eq!(posts.unwrap(), []);
}

const THREE_POSTS: &str = "insert into posts (id, title, body) values \
    (1, 'Sabai', 'a'), (2, 'Sabai', null), (3, 'Other', 'a')";

#[test]
fn where_eq_finds_rows_by_a_column_and_first_takes_one() {
    let (first, missing) = run(&app("where-first"), &[CREATE_POSTS, THREE_POSTS], async {
        let first = Post::where_eq("title", "Sabai").first().await;
        let missing = Post::where_eq("title", "Nope").first().await;
        (first, missing)
    });

    assert_eq!(
        first.unwrap(),
        Some(Post {
            id: 1,
            title: "Sabai".into(),
            body: Some("a".into())
        })
    );
    assert_eq!(missing.unwrap(), None);
}

#[test]
fn chained_where_eqs_all_have_to_match() {
    let posts = run(
        &app("where-and"),
        &[CREATE_POSTS, THREE_POSTS],
        Post::where_eq("title", "Sabai").where_eq("body", "a").get(),
    );

    assert_eq!(
        posts.unwrap(),
        [Post {
            id: 1,
            title: "Sabai".into(),
            body: Some("a".into())
        }]
    );
}

#[test]
fn where_eq_none_finds_null_columns() {
    let posts = run(
        &app("where-null"),
        &[CREATE_POSTS, THREE_POSTS],
        Post::where_eq("body", None::<String>).get(),
    );

    assert_eq!(
        posts.unwrap(),
        [Post {
            id: 2,
            title: "Sabai".into(),
            body: None
        }]
    );
}

#[test]
fn a_column_the_model_does_not_have_never_reaches_the_sql() {
    let error = run(
        &app("where-unknown"),
        &[CREATE_POSTS],
        Post::where_eq("title = '' or 1=1 --", "x").get(),
    )
    .unwrap_err();

    assert_eq!(
        error.to_string(),
        "`posts` has no column `title = '' or 1=1 --`; its columns are id, title, body"
    );
}

#[test]
fn a_date_time_field_is_saved_and_read_back() {
    let meeting = run(
        &app("date-time"),
        &[
            "create table meetings (id integer primary key, starts_at datetime not null, ended_at datetime)",
        ],
        async {
            let mut meeting = Meeting {
                id: 0,
                starts_at: "2026-10-23 10:05:09".parse().unwrap(),
                ended_at: None,
            };
            meeting.save().await.unwrap();
            Meeting::find_or_fail(meeting.id).await.unwrap()
        },
    );

    assert_eq!(meeting.starts_at.to_string(), "2026-10-23 10:05:09");
    assert_eq!(meeting.ended_at, None);
}

#[test]
fn a_date_time_column_holding_other_text_names_the_column() {
    let error = run(
        &app("date-time-text"),
        &[
            "create table meetings (id integer primary key, starts_at datetime not null, ended_at datetime)",
            "insert into meetings (starts_at) values ('tomorrow')",
        ],
        async { Meeting::find_or_fail(1).await.err().unwrap().to_string() },
    );

    assert!(
        error.contains("could not read column `starts_at`"),
        "{error}"
    );
    assert!(
        error.contains("`tomorrow` is not a date and time"),
        "{error}"
    );
}

#[test]
fn save_sets_both_timestamps_on_insert_and_only_updated_at_after() {
    let clock = Arc::new(FrozenClock(Mutex::new(
        SystemTime::UNIX_EPOCH + Duration::from_millis(1_792_000_000_250),
    )));
    let base = std::env::temp_dir().join(format!("sabai-it-model-timestamps-{}", process::id()));
    fs::create_dir_all(base.join("config")).unwrap();
    fs::write(base.join("config/database.toml"), ONE_CONNECTION).unwrap();
    let app = App::builder(Config::load(&base).unwrap())
        .provider(DatabaseServiceProvider)
        .provider(FrozenClockProvider(clock.clone()))
        .boot()
        .unwrap();

    let (inserted, updated) = run(
        &app,
        &[
            "create table comments (id integer primary key, body text not null, created_at datetime, updated_at datetime)",
        ],
        async {
            let mut comment = Comment {
                id: 0,
                body: "First".into(),
                created_at: None,
                updated_at: None,
            };
            comment.save().await.unwrap();
            let inserted = Comment::find_or_fail(comment.id).await.unwrap();
            *clock.0.lock().unwrap() += Duration::from_secs(3600);
            comment.body = "Edited".into();
            comment.save().await.unwrap();
            (inserted, Comment::find_or_fail(comment.id).await.unwrap())
        },
    );

    let created = inserted.created_at.unwrap().to_json();
    assert_eq!(created, "2026-10-14T17:46:40.000000Z");
    assert_eq!(inserted.updated_at, inserted.created_at);
    assert_eq!(updated.created_at, inserted.created_at);
    assert_eq!(
        updated.updated_at.unwrap().to_json(),
        "2026-10-14T18:46:40.000000Z"
    );
}

#[test]
fn the_saved_model_holds_the_same_timestamps_the_database_keeps() {
    let (saved, read) = run(
        &app("timestamps-match"),
        &[
            "create table comments (id integer primary key, body text not null, created_at datetime, updated_at datetime)",
        ],
        async {
            let mut comment = Comment {
                id: 0,
                body: "First".into(),
                created_at: None,
                updated_at: None,
            };
            comment.save().await.unwrap();
            let read = Comment::find_or_fail(comment.id).await.unwrap();
            (comment, read)
        },
    );

    assert_eq!(saved, read);
}
