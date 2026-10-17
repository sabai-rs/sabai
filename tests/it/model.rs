use std::sync::Arc;
use std::{fs, process};

use sabai::orm::{Database, DatabaseServiceProvider, Model};
use sabai::{App, Config};

#[derive(Model, Debug, PartialEq)]
struct Post {
    id: u64,
    title: String,
    body: Option<String>,
}

#[derive(Model)]
#[model(table = "people")]
struct Person {
    id: u64,
}

fn app(name: &str) -> Arc<App> {
    let base = std::env::temp_dir().join(format!("sabai-it-model-{name}-{}", process::id()));
    fs::create_dir_all(base.join("config")).unwrap();
    fs::write(
        base.join("config/database.toml"),
        "default = \"sqlite\"\n[connections.sqlite]\ndriver = \"sqlite\"\ndatabase = \":memory:\"\n",
    )
    .unwrap();
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
fn the_derive_names_the_table_like_laravel_and_lists_the_fields() {
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
