use super::detector::ProjectDetector;
use super::version;
use crate::model::ProjectContext;
use std::path::Path;

pub struct RustDetector;

impl ProjectDetector for RustDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Option<ProjectContext> {
        cwd.join("Cargo.toml").exists().then(|| ProjectContext {
            kind: "rust".into(),
            version: (!fast)
                .then(|| version::command_version("rustc", "--version"))
                .flatten(),
        })
    }
}
