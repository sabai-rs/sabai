//! `sabai new <name>`: a new app from `templates/app`, like `laravel new`.

use std::fs;
use std::path::{Path, PathBuf};

use clap::{Arg, ArgMatches, Command};

/// Every file in `templates/app` and where it goes in the new app, listed by `build.rs`.
const TEMPLATE: &[(&str, &str)] = include!(concat!(env!("OUT_DIR"), "/template.rs"));

const SABAI_REPOSITORY: &str = "https://github.com/sabai-rs/sabai";

/// Where the new app gets Sabai's crates from.
enum Source {
    GitHub,
    Clone(PathBuf),
}

impl Source {
    /// The inside of a dependency table for one of Sabai's crates: `git = "..."` or `path = "..."`.
    fn locate(&self, krate: &str) -> String {
        // `{:?}` quotes and escapes the way a TOML string needs.
        match self {
            Source::GitHub => format!("git = {SABAI_REPOSITORY:?}"),
            Source::Clone(clone) => {
                format!(
                    "path = {:?}",
                    clone.join("crates").join(krate).display().to_string()
                )
            }
        }
    }
}

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
    let source = match matches.get_one::<String>("sabai-path") {
        Some(clone) => clone_source(Path::new(clone))?,
        None => Source::GitHub,
    };
    create(Path::new("."), name, &source)?;
    println!("Created {name}. Next:\n\n  cd {name}\n  sabai migrate\n  sabai serve\n");
    Ok(())
}

/// Writes the template into `parent/name`, which must not exist yet.
fn create(parent: &Path, name: &str, source: &Source) -> Result<(), String> {
    check_name(name)?;
    let folder = parent.join(name);
    if folder.exists() {
        return Err(format!(
            "`{name}` already exists; pick another name or remove it first"
        ));
    }
    for (path, stub) in TEMPLATE {
        let contents = fill(stub, name, source);
        write_file(&folder.join(path), &contents)?;
        if *path == ".env.example" {
            write_file(&folder.join(".env"), &contents)?;
        }
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

fn fill(stub: &str, name: &str, source: &Source) -> String {
    stub.replace("{{name}}", name)
        .replace("{{sabai}}", &source.locate("sabai"))
        .replace("{{sabai-testing}}", &source.locate("sabai-testing"))
}

fn clone_source(clone: &Path) -> Result<Source, String> {
    let not_a_clone = |_| {
        format!(
            "{} is not a Sabai clone: it has no crates/sabai",
            clone.display()
        )
    };
    clone
        .join("crates/sabai")
        .canonicalize()
        .map_err(not_a_clone)?;
    Ok(Source::Clone(clone.canonicalize().map_err(not_a_clone)?))
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

        create(&parent, "my-shop", &Source::GitHub).unwrap();

        let app = parent.join("my-shop");
        for (path, _) in TEMPLATE {
            assert!(app.join(path).is_file(), "{path} is missing");
        }
        assert!(
            app.join(".env").is_file(),
            ".env was not copied from .env.example"
        );
        let main = fs::read_to_string(app.join("src/main.rs")).unwrap();
        assert!(main.contains("bootstrap::config()"), "{main}");
        let manifest = fs::read_to_string(app.join("Cargo.toml")).unwrap();
        assert!(manifest.contains("name = \"my-shop\""), "{manifest}");
        assert!(
            manifest.contains(
                r#"sabai = { git = "https://github.com/sabai-rs/sabai", features = ["orm"] }"#
            ),
            "{manifest}"
        );
    }

    #[test]
    fn template_names_become_the_app_names() {
        let paths: Vec<&str> = TEMPLATE.iter().map(|(path, _)| *path).collect();

        assert!(paths.contains(&"Cargo.toml"), "{paths:?}");
        assert!(paths.contains(&".gitignore"), "{paths:?}");
        assert!(paths.contains(&"src/routes/web.rs"), "{paths:?}");
        assert!(
            !paths
                .iter()
                .any(|path| path.contains("stub") || path.contains("dot-")),
            "{paths:?}"
        );
    }

    #[test]
    fn an_existing_folder_is_never_overwritten() {
        let parent = scratch("exists");
        fs::create_dir(parent.join("blog")).unwrap();

        let error = create(&parent, "blog", &Source::GitHub).unwrap_err();

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
    fn a_clone_gives_a_path_to_each_crate() {
        let clone = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");

        let source = clone_source(&clone).unwrap();

        let sabai = source.locate("sabai");
        let testing = source.locate("sabai-testing");
        assert!(
            sabai.starts_with("path = \"/") && sabai.ends_with("crates/sabai\""),
            "{sabai}"
        );
        assert!(testing.ends_with("crates/sabai-testing\""), "{testing}");
    }
}
