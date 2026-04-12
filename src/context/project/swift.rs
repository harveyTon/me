use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct SwiftDetector;

impl ProjectDetector for SwiftDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if !cwd.join("Package.swift").exists() {
            return Vec::new();
        }
        vec![project_context("swift", None, [])]
    }
}
