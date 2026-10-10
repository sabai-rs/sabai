use sabai::{Config, ConfigSection};
use serde::Deserialize;

#[derive(Debug, Deserialize, ConfigSection)]
struct AppConfig {
    name: String,
    debug: bool,
}

#[derive(Debug, Deserialize, ConfigSection)]
#[config("database")]
struct Connections {
    connections: std::collections::HashMap<String, Connection>,
}

#[derive(Debug, Deserialize)]
struct Connection {
    path: String,
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
fn a_config_section_type_knows_its_own_key() {
    let app = fixture().section::<AppConfig>().unwrap();

    assert_eq!(app.name, "Todo from .env");
    assert!(app.debug);
}

#[test]
fn the_derive_reads_the_file_named_after_the_struct_or_the_attribute() {
    let config = fixture();

    let connections = config.section::<Connections>().unwrap();

    assert_eq!(<AppConfig as ConfigSection>::KEY, "app");
    assert_eq!(<Connections as ConfigSection>::KEY, "database");
    assert_eq!(connections.connections["sqlite"].path, "database.sqlite");
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
