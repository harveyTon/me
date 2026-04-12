use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct PhpDetector;

impl ProjectDetector for PhpDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if !cwd.join("composer.json").exists() {
            return Vec::new();
        }
        vec![project_context("php", None, [])]
    }
}
