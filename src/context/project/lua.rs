use super::detector::ProjectDetector;
use super::project_context;
use crate::model::ProjectContext;
use std::path::Path;

pub struct LuaDetector;

impl ProjectDetector for LuaDetector {
    fn detect(&self, cwd: &Path, _fast: bool) -> Vec<ProjectContext> {
        if !has_luarocks_signal(cwd) {
            return Vec::new();
        }
        vec![project_context("lua", None, [])]
    }
}

fn has_luarocks_signal(cwd: &Path) -> bool {
    std::fs::read_dir(cwd)
        .ok()
        .into_iter()
        .flat_map(|entries| entries.flatten())
        .any(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "rockspec")
        })
}
