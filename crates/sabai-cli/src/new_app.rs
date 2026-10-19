//! `sabai new <name>`: a new app from `templates/app`, like `laravel new`.

use std::fs;
use std::path::Path;

use clap::{Arg, ArgMatches, Command};

macro_rules! stub {
    ($path:literal) => {
        include_str!(concat!("../../../templates/app/", $path))
    };
}

// Dotfiles and `Cargo.toml` are stubs, so they do not act on this repository's git and Cargo.
const TEMPLATE: &[(&str, &str)] = &[
    ("Cargo.toml", stub!("Cargo.toml.stub")),
    (".gitignore", stub!("gitignore.stub")),
    (".env.example", stub!("env.stub")),
    (".env", stub!("env.stub")),
    ("config/app.toml", stub!("config/app.toml")),
    ("config/logging.toml", stub!("config/logging.toml")),
    ("src/main.rs", stub!("src/main.rs")),
    ("src/lib.rs", stub!("src/lib.rs")),
    ("src/app.rs", stub!("src/app.rs")),
    ("src/app/providers.rs", stub!("src/app/providers.rs")),
    (
        "src/app/providers/app_service_provider.rs",
        stub!("src/app/providers/app_service_provider.rs"),
    ),
    ("src/routes.rs", stub!("src/routes.rs")),
    ("src/routes/web.rs", stub!("src/routes/web.rs")),
];

const SABAI_FROM_GITHUB: &str = r#"{ git = "https://github.com/sabai-rs/sabai" }"#;

pub fn command() -> Command {
    Command::new("new")
        .about("Create a new Sabai app in a new folder")
        .arg(
            Arg::new("name")
                .required(true)
                .help("The app's name and folder, like `blog`"),
        )
        .arg(
            Arg::new("sabai-path")
                .long("sabai-path")
                .value_name("CLONE")
                .help("Use Sabai from a local clone instead of GitHub"),
        )
}

pub fn run(matches: &ArgMatches) -> Result<(), String> {
    let name = matches.get_one::<String>("name").map_or("", String::as_str);
    let sabai = match matches.get_one::<String>("sabai-path") {
        Some(clone) => sabai_from_clone(Path::new(clone))?,
        None => SABAI_FROM_GITHUB.to_owned(),
    };
    create(Path::new("."), name, &sabai)?;
    println!("Created {name}. Next:\n\n  cd {name}\n  sabai serve\n");
    Ok(())
}

/// Writes the template into `parent/name`, which must not exist yet.
fn create(parent: &Path, name: &str, sabai: &str) -> Result<(), String> {
    check_name(name)?;
    let folder = parent.join(name);
    if folder.exists() {
        return Err(format!(
            "`{name}` already exists; pick another name or remove it first"
        ));
    }
    for (path, stub) in TEMPLATE {
        write_file(&folder.join(path), &fill(stub, name, sabai))?;
    }
    Ok(())
}

fn check_name(name: &str) -> Result<(), String> {
    let starts_with_letter = name.starts_with(|c: char| c.is_ascii_lowercase());
    let allowed = |c: char| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-' || c == '_';
    if starts_with_letter && name.chars().all(allowed) && name != "sabai" {
        return Ok(());
    }
    Err(format!(
        "`{name}` cannot be an app name; use lowercase letters, digits, `-` and `_`, starting with a letter, like `blog` or `my-shop`"
    ))
}

fn fill(stub: &str, name: &str, sabai: &str) -> String {
    stub.replace("{{name}}", name)
        .replace("{{crate}}", &name.replace('-', "_"))
        .replace("{{sabai}}", sabai)
}

fn sabai_from_clone(clone: &Path) -> Result<String, String> {
    let facade = clone.join("crates/sabai");
    let facade = facade.canonicalize().map_err(|_| {
        format!(
            "{} is not a Sabai clone: it has no crates/sabai",
            clone.display()
        )
    })?;
    // `{:?}` quotes and escapes the path the way a TOML string needs.
    Ok(format!("{{ path = {:?} }}", facade.display().to_string()))
}

fn write_file(path: &Path, contents: &str) -> Result<(), String> {
    let failed = |error| format!("could not write {}: {error}", path.display());
    if let Some(folder) = path.parent() {
        fs::create_dir_all(folder).map_err(failed)?;
    }
    fs::write(path, contents).map_err(failed)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(test: &str) -> std::path::PathBuf {
        let folder = std::env::temp_dir().join(format!("sabai-new-{}-{test}", std::process::id()));
        let _ = fs::remove_dir_all(&folder);
        fs::create_dir_all(&folder).unwrap();
        folder
    }

    #[test]
    fn a_new_app_gets_every_template_file_with_its_name_filled_in() {
        let parent = scratch("files");

        create(&parent, "my-shop", SABAI_FROM_GITHUB).unwrap();

        let app = parent.join("my-shop");
        for (path, _) in TEMPLATE {
            assert!(app.join(path).is_file(), "{path} is missing");
        }
        let main = fs::read_to_string(app.join("src/main.rs")).unwrap();
        assert!(main.contains("my_shop::config()"), "{main}");
        let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("name = \"my-shop\""), "{manifest}");
        assert!(manifest.contains("sabai = { git = "), "{manifest}");
    }

    #[test]
    fn an_existing_folder_is_never_overwritten() {
        let parent = scratch("exists");
        fs::create_dir(parent.join("blog")).unwrap();

        let error = create(&parent, "blog", SABAI_FROM_GITHUB).unwrap_err();

        assert!(error.starts_with("`blog` already exists"), "{error}");
    }

    #[test]
    fn a_name_cargo_cannot_use_is_refused() {
        for name in ["", "Blog", "1blog", "my shop", "../blog", "sabai"] {
            assert!(check_name(name).is_err(), "{name:?} was allowed");
        }
        for name in ["blog", "my-shop", "shop_2"] {
            assert!(check_name(name).is_ok(), "{name:?} was refused");
        }
    }

    #[test]
    fn a_clone_path_becomes_a_path_dependency() {
        let clone = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        let sabai = sabai_from_clone(&clone).unwrap();

        assert!(
            sabai.starts_with("{ path = \"/") && sabai.ends_with("crates/sabai\" }"),
            "{sabai}"
        );
    }
}
