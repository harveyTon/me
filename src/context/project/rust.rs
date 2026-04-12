use super::detector::ProjectDetector;
use super::{project_context, version};
use crate::model::ProjectContext;
use std::path::Path;

pub struct RustDetector;

impl ProjectDetector for RustDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Vec<ProjectContext> {
        if !cwd.join("Cargo.toml").exists() {
            return Vec::new();
        }
        vec![project_context(
            "rust",
            (!fast)
                .then(|| version::command_version("rustc", "--version"))
                .flatten(),
            [],
        )]
    }
}
