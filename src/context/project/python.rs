use super::detector::ProjectDetector;
use super::{project_context, version};
use crate::model::ProjectContext;
use std::path::Path;

pub struct PythonDetector;

impl ProjectDetector for PythonDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Vec<ProjectContext> {
        if !python_signal(cwd) {
            return Vec::new();
        }
        vec![project_context(
            "python",
            (!fast).then(resolve_python_version).flatten(),
            python_details(cwd),
        )]
    }
}

fn python_signal(cwd: &Path) -> bool {
    [
        "pyproject.toml",
        "poetry.lock",
        "Pipfile",
        "requirements.txt",
    ]
    .into_iter()
    .any(|name| cwd.join(name).exists())
}

fn resolve_python_version() -> Option<String> {
    version::command_version("python3", "--version")
        .or_else(|| version::command_version("python", "--version"))
}

fn python_details(cwd: &Path) -> Vec<String> {
    if let Some(env_name) = std::env::var_os("VIRTUAL_ENV")
        .as_deref()
        .and_then(|path| Path::new(path).file_name())
        .map(|name| name.to_string_lossy().into_owned())
    {
        return vec![env_name];
    }
    if cwd.join(".venv").exists() {
        return vec![".venv".to_string()];
    }
    if cwd.join("venv").exists() {
        return vec!["venv".to_string()];
    }
    Vec::new()
}
