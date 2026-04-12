use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct CCppDetector;

impl ProjectDetector for CCppDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if !["CMakeLists.txt", "Makefile", "meson.build"]
            .into_iter()
            .any(|name| cwd.join(name).exists())
        {
            return Vec::new();
        }
        vec![project_context("c/cpp", None, [])]
    }
}
