use sabai::Config;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AppConfig {
    name: String,
    debug: bool,
}

fn fixture() -> Config {
    Config::load(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/it/fixtures"
    ))
    .unwrap()
}

#[test]
fn each_toml_file_becomes_a_typed_section() {
    let app: AppConfig = fixture().get("app").unwrap();

    assert_eq!(app.name, "Todo from .env");
    assert!(app.debug);
}

#[test]
fn dotted_keys_read_across_files() {
    let config = fixture();

    assert_eq!(
        config.get::<String>("app.url").unwrap(),
        "http://localhost:3000"
    );
    assert_eq!(
        config
            .get::<String>("database.connections.sqlite.path")
            .unwrap(),
        "database.sqlite"
    );
}

#[test]
fn a_missing_directory_says_which_one() {
    let error = Config::load("does/not/exist").unwrap_err();

    assert!(
        error
            .to_string()
            .contains("could not read the config directory `does/not/exist/config`")
    );
}
