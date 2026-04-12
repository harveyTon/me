use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct CSharpDetector;

impl ProjectDetector for CSharpDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if has_csharp_signal(cwd) {
            return vec![project_context("csharp", None, [])];
        }
        Vec::new()
    }
}

fn has_csharp_signal(cwd: &Path) -> bool {
    std::fs::read_dir(cwd)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "csproj" || extension == "sln")
        })
}
