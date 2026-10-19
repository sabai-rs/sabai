//! Lists every file in `templates/app` for `sabai new`, so a new template file needs no code.

use std::env;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn main() -> io::Result<()> {
    let manifest_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").unwrap_or_default());
    let template = manifest_dir.join("../../templates/app").canonicalize()?;
    println!("cargo::rerun-if-changed={}", template.display());

    let mut files = Vec::new();
    collect_files(&template, &mut files)?;
    files.sort();
    let entries: String = files
        .iter()
        .map(|file| {
            format!(
                "    ({:?}, include_str!({:?})),\n",
                app_path(&template, file),
                file
            )
        })
        .collect();

    let out_dir = PathBuf::from(env::var_os("OUT_DIR").unwrap_or_default());
    fs::write(out_dir.join("template.rs"), format!("&[\n{entries}]\n"))
}

fn collect_files(folder: &Path, files: &mut Vec<PathBuf>) -> io::Result<()> {
    for entry in fs::read_dir(folder)? {
        let path = entry?.path();
        if path.is_dir() {
            collect_files(&path, files)?;
        } else {
            files.push(path);
        }
    }
    Ok(())
}

/// Where `file` goes in the new app: `dot-gitignore` becomes `.gitignore`, `Cargo.toml.stub` becomes `Cargo.toml`.
fn app_path(template: &Path, file: &Path) -> String {
    let relative = file.strip_prefix(template).unwrap_or(file);
    let parts: Vec<String> = relative
        .iter()
        .map(|part| part.to_string_lossy().replacen("dot-", ".", 1))
        .collect();
    let path = parts.join("/");
    path.strip_suffix(".stub")
        .map_or(path.clone(), str::to_owned)
}
