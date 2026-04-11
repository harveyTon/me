use crate::model::GitContext;
use std::path::Path;

const MAX_DEPTH: usize = 8;

pub fn detect() -> Option<GitContext> {
    let cwd = std::env::current_dir().ok()?;
    let git_dir = find_git_dir(&cwd)?;
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let branch = head
        .trim()
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned)
        .filter(|b| !b.is_empty())?;
    Some(GitContext { branch })
}

fn find_git_dir(start: &Path) -> Option<std::path::PathBuf> {
    let home = dirs::home_dir();
    let mut dir = start;
    for _ in 0..MAX_DEPTH {
        let git = dir.join(".git");
        if git.exists() {
            if git.is_dir() {
                return Some(git);
            }
            if let Ok(content) = std::fs::read_to_string(&git) {
                let gitdir = content.trim().strip_prefix("gitdir: ")?;
                let resolved = dir.join(gitdir);
                if resolved.join("HEAD").exists() {
                    return Some(resolved);
                }
            }
            return None;
        }
        if home.as_ref().is_some_and(|h| dir == h) {
            return None;
        }
        dir = dir.parent()?;
    }
    None
}
