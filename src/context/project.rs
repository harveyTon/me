use crate::model::ProjectContext;
use std::path::Path;
use std::process::Command;

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

pub fn detect_fast() -> Option<ProjectContext> {
    let cwd = std::env::current_dir().ok()?;
    if cwd.join("Cargo.toml").exists() {
        return Some(ProjectContext {
            kind: "rust".into(),
            version: None,
        });
    }
    if cwd.join("package.json").exists() {
        return Some(ProjectContext {
            kind: "node".into(),
            version: None,
        });
    }
    None
}

fn version(bin: &str, arg: &str) -> Option<String> {
    if !Path::new(bin).exists() && which::which(bin).is_err() {
        return None;
    }
    let output = Command::new(bin).arg(arg).output().ok()?;
    let raw = output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())?;
    Some(extract_version(&raw))
}

fn extract_version(raw: &str) -> String {
    raw.split_whitespace()
        .find(|part| part.chars().next().is_some_and(|c| c.is_ascii_digit()))
        .map(|v| v.to_owned())
        .unwrap_or_else(|| raw.to_owned())
}
