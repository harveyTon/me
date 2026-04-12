use super::detector::ProjectDetector;
use super::{project_context, version};
use crate::model::ProjectContext;
use std::path::Path;

pub struct GoDetector;

impl ProjectDetector for GoDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Vec<ProjectContext> {
        if !cwd.join("go.mod").exists() && !cwd.join("main.go").exists() {
            return Vec::new();
        }
        vec![project_context(
            "go",
            (!fast)
                .then(|| version::command_version_any("go", &["version"]))
                .flatten(),
            [],
        )]
    }
}
