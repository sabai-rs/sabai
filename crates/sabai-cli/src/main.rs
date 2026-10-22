//! The `sabai` command: creates apps like `laravel new`, and runs an app's own commands
//! from anywhere inside it, like `php artisan`.

mod new_app;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, ExitCode, ExitStatus};

use clap::{ArgMatches, Command, value_parser};

fn main() -> ExitCode {
    let matches = cli().get_matches();
    let ran = match matches.subcommand() {
        Some(("new", new)) => new_app::run(new).map(|()| ExitCode::SUCCESS),
        _ => run_in_app(&app_args(&matches)).map(exit_code),
    };
    match ran {
        Ok(code) => code,
        Err(problem) => {
            eprintln!("Error: {problem}");
            ExitCode::FAILURE
        }
    }
}

fn cli() -> Command {
    Command::new("sabai")
        .version(env!("CARGO_PKG_VERSION"))
        .about("Creates Sabai apps and runs their commands")
        .after_help(
            "Other commands run in your app: `sabai migrate` is `cargo run -q -- migrate`, and `sabai` alone lists them.",
        )
        .subcommand(new_app::command())
        .disable_help_subcommand(true)
        .allow_external_subcommands(true)
        .external_subcommand_value_parser(value_parser!(String))
}

/// The arguments for the app's binary; with none, its `list`.
fn app_args(matches: &ArgMatches) -> Vec<String> {
    let Some((command, rest)) = matches.subcommand() else {
        return vec!["list".to_owned()];
    };
    let rest = rest.get_many::<String>("").into_iter().flatten().cloned();
    [command.to_owned()].into_iter().chain(rest).collect()
}

fn run_in_app(args: &[String]) -> Result<ExitStatus, String> {
    let here = env::current_dir()
        .map_err(|error| format!("could not read the current folder: {error}"))?;
    let app = find_app(&here).ok_or_else(|| {
        format!(
            "{} is not inside a Sabai app; `cd` into your app's folder first",
            here.display()
        )
    })?;
    // `cargo run` builds the app first, so its commands always run the code on disk.
    process::Command::new("cargo")
        .args(["run", "--quiet", "--"])
        .args(args)
        .current_dir(app)
        .status()
        .map_err(|error| format!("could not run `cargo`; is Rust installed? ({error})"))
}

/// The nearest folder at or above `dir` whose `Cargo.toml` is a package that uses `sabai`.
fn find_app(dir: &Path) -> Option<PathBuf> {
    dir.ancestors()
        .find(|folder| {
            fs::read_to_string(folder.join("Cargo.toml"))
                .is_ok_and(|manifest| is_sabai_app(&manifest))
        })
        .map(Path::to_path_buf)
}

// A workspace root has no `[package]`, so `cargo run` there would not know which app to run.
fn is_sabai_app(manifest: &str) -> bool {
    let lines = || manifest.lines().map(str::trim_start);
    lines().any(|line| line == "[package]") && lines().any(|line| line.starts_with("sabai "))
}

fn exit_code(status: ExitStatus) -> ExitCode {
    status
        .code()
        .and_then(|code| u8::try_from(code).ok())
        .map_or(ExitCode::FAILURE, ExitCode::from)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args_for(command_line: &[&str]) -> Vec<String> {
        app_args(&cli().try_get_matches_from(command_line).unwrap())
    }

    #[test]
    fn a_command_and_its_arguments_go_to_the_app_untouched() {
        assert_eq!(
            args_for(&["sabai", "greet", "Somchai", "--loud"]),
            ["greet", "Somchai", "--loud"]
        );
    }

    #[test]
    fn no_command_lists_the_app_commands() {
        assert_eq!(args_for(&["sabai"]), ["list"]);
    }

    #[test]
    fn an_app_is_a_package_that_depends_on_sabai() {
        let blog = "[package]\nname = \"blog\"\n\n[dependencies]\nsabai = { workspace = true }\n";
        let workspace = "[workspace]\nmembers = [\"crates/*\"]\n\n[workspace.dependencies]\nsabai = { path = \"crates/sabai\" }\n";
        let other = "[package]\nname = \"tool\"\n\n[dependencies]\nsabai-core = \"0.1\"\n";

        assert!(is_sabai_app(blog));
        assert!(!is_sabai_app(workspace));
        assert!(!is_sabai_app(other));
    }

    #[test]
    fn the_app_is_found_from_a_folder_inside_it() {
        let blog = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/blog");

        let found = find_app(&blog.join("src/app")).unwrap();

        assert_eq!(found.canonicalize().unwrap(), blog.canonicalize().unwrap());
    }
}
