use crate::model::ProjectContext;
use std::{path::Path, process::Command};

pub fn detect() -> Option<ProjectContext> {
    let cwd = std::env::current_dir().ok()?;
    if cwd.join("Cargo.toml").exists() {
        return Some(ProjectContext {
            kind: "rust".into(),
            version: version("rustc", "--version"),
        });
    }
    if cwd.join("package.json").exists() {
        return Some(ProjectContext {
            kind: "node".into(),
            version: version("node", "--version"),
        });
    }
    None
}

fn version(bin: &str, arg: &str) -> Option<String> {
    if !Path::new(bin).exists() && which::which(bin).is_err() {
        return None;
    }
    let output = Command::new(bin).arg(arg).output().ok()?;
    output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())
}
