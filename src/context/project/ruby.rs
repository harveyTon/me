use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct RubyDetector;

impl ProjectDetector for RubyDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if !cwd.join("Gemfile").exists() {
            return Vec::new();
        }
        vec![project_context("ruby", None, [])]
    }
}
