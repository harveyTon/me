use super::detector::ProjectDetector;
use super::version;
use crate::model::ProjectContext;
use std::path::Path;

pub struct NodeDetector;

impl ProjectDetector for NodeDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Option<ProjectContext> {
        cwd.join("package.json").exists().then(|| ProjectContext {
            kind: "node".into(),
            version: (!fast)
                .then(|| version::command_version("node", "--version"))
                .flatten(),
        })
    }
}
