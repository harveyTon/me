use super::detector::ProjectDetector;
use crate::model::ProjectContext;
use serde_yaml::Value;
use std::{ffi::OsStr, fs, path::Path};

pub struct ComposeDetector;

impl ProjectDetector for ComposeDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Vec<ProjectContext> {
        let Some(path) = compose_file(cwd) else {
            return Vec::new();
        };

        let (project_name, service_count) = if fast {
            (None, None)
        } else {
            compose_summary(&path)
        };

        vec![ProjectContext {
            kind: "docker compose".into(),
            version: None,
            project_name,
            service_count,
            details: Vec::new(),
        }]
    }
}

fn compose_file(cwd: &Path) -> Option<std::path::PathBuf> {
    for name in [
        "compose.yaml",
        "compose.yml",
        "docker-compose.yaml",
        "docker-compose.yml",
        "docker-compose.override.yaml",
        "docker-compose.override.yml",
    ] {
        let path = cwd.join(name);
        if path.exists() {
            return Some(path);
        }
    }

    let mut variants = fs::read_dir(cwd)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| is_compose_variant(path.file_name()))
        .collect::<Vec<_>>();
    variants.sort();
    variants.into_iter().next()
}

fn is_compose_variant(name: Option<&OsStr>) -> bool {
    let Some(name) = name.and_then(OsStr::to_str) else {
        return false;
    };
    (name.starts_with("compose.") || name.starts_with("docker-compose."))
        && (name.ends_with(".yaml") || name.ends_with(".yml"))
}

fn compose_summary(path: &Path) -> (Option<String>, Option<usize>) {
    let Ok(contents) = fs::read_to_string(path) else {
        return (None, None);
    };
    let Ok(value) = serde_yaml::from_str::<Value>(&contents) else {
        return (None, None);
    };
    let project_name = value
        .get("name")
        .and_then(Value::as_str)
        .filter(|name| !name.trim().is_empty())
        .map(|name| name.trim().to_owned());
    let service_count = value
        .get("services")
        .and_then(Value::as_mapping)
        .map(|services| services.len());
    (project_name, service_count)
}
