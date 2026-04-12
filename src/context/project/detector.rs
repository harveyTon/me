use crate::model::ProjectContext;
use std::path::Path;

pub trait ProjectDetector: Sync {
    fn detect(&self, cwd: &Path, fast: bool) -> Option<ProjectContext>;
}
