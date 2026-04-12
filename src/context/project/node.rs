use super::detector::ProjectDetector;
use super::{project_context, version};
use crate::model::ProjectContext;
use std::path::Path;

pub struct NodeDetector;

impl ProjectDetector for NodeDetector {
    fn detect(&self, cwd: &Path, fast: bool) -> Vec<ProjectContext> {
        if !cwd.join("package.json").exists() {
            return Vec::new();
        }
        vec![project_context(
            "node",
            (!fast)
                .then(|| version::command_version("node", "--version"))
                .flatten(),
            node_details(cwd),
        )]
    }
}

fn node_details(cwd: &Path) -> Vec<String> {
    let mut details = Vec::new();
    if cwd.join("pnpm-lock.yaml").exists() || cwd.join("pnpm-workspace.yaml").exists() {
        details.push("pnpm".to_string());
    } else if cwd.join("yarn.lock").exists() {
        details.push("yarn".to_string());
    } else if cwd.join("package-lock.json").exists() {
        details.push("npm".to_string());
    }
    if cwd.join("turbo.json").exists() {
        details.push("turbo".to_string());
    }
    if cwd.join("nx.json").exists() {
        details.push("nx".to_string());
    }
    details
}
