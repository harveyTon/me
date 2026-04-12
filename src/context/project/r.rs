use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct RDetector;

impl ProjectDetector for RDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if cwd.join("DESCRIPTION").exists() || has_rproj(cwd) {
            return vec![project_context("r", None, [])];
        }
        Vec::new()
    }
}

fn has_rproj(cwd: &Path) -> bool {
    std::fs::read_dir(cwd)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "Rproj")
        })
}
