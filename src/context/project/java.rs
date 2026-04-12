use super::detector::ProjectDetector;
use super::{project_context, version};
use crate::model::ProjectContext;
use std::path::Path;

pub struct JavaDetector;

impl ProjectDetector for JavaDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Vec<ProjectContext> {
        let Some(tool) = java_build_tool(cwd) else {
            return Vec::new();
        };
        let version = (!fast)
            .then(|| version::command_version_any("java", &["-version"]))
            .flatten()
            .map(|value| version::major_version(&value));
        vec![project_context("java", version, [tool.to_string()])]
    }
}

fn java_build_tool(cwd: &Path) -> Option<&'static str> {
    if [
        "build.gradle",
        "build.gradle.kts",
        "settings.gradle",
        "settings.gradle.kts",
    ]
    .into_iter()
    .any(|name| cwd.join(name).exists())
    {
        return Some("gradle");
    }
    if cwd.join("pom.xml").exists() {
        return Some("maven");
    }
    None
}
