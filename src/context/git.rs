use crate::model::GitContext;
use std::path::{Path, PathBuf};

const MAX_DEPTH: usize = 8;

pub fn detect() -> Option<GitContext> {
    let cwd = std::env::current_dir().ok()?;
    detect_from(&cwd)
}

pub fn detect_fast() -> Option<GitContext> {
    let cwd = std::env::current_dir().ok()?;
    detect_fast_from(&cwd)
}

fn detect_from(start: &Path) -> Option<GitContext> {
    let git_dir = find_git_dir(start)?;
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let branch = head_name(&git_dir, head.trim())?;
    Some(GitContext { branch })
}

fn detect_fast_from(start: &Path) -> Option<GitContext> {
    let git_dir = find_git_dir(start)?;
    let head = std::fs::read_to_string(git_dir.join("HEAD")).ok()?;
    let branch = head_name_fast(head.trim())?;
    Some(GitContext { branch })
}

fn head_name(git_dir: &Path, head: &str) -> Option<String> {
    if let Some(branch) = head
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned)
        .filter(|b| !b.is_empty())
    {
        return Some(branch);
    }

    if !head.chars().all(|c| c.is_ascii_hexdigit()) || head.len() < 7 {
        return None;
    }

    tag_for_oid(git_dir, head).or_else(|| Some(head[..12.min(head.len())].to_owned()))
}

fn head_name_fast(head: &str) -> Option<String> {
    if let Some(branch) = head
        .strip_prefix("ref: refs/heads/")
        .map(str::to_owned)
        .filter(|b| !b.is_empty())
    {
        return Some(branch);
    }

    if !head.chars().all(|c| c.is_ascii_hexdigit()) || head.len() < 7 {
        return None;
    }

    Some(head[..12.min(head.len())].to_owned())
}

fn tag_for_oid(git_dir: &Path, oid: &str) -> Option<String> {
    loose_tag_for_oid(&git_dir.join("refs/tags"), oid)
        .or_else(|| packed_tag_for_oid(&git_dir.join("packed-refs"), oid))
}

fn loose_tag_for_oid(tags_dir: &Path, oid: &str) -> Option<String> {
    if !tags_dir.is_dir() {
        return None;
    }
    let mut stack = vec![tags_dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).ok()?.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
                continue;
            }
            let value = std::fs::read_to_string(&path).ok()?;
            if value.trim() == oid {
                return path
                    .strip_prefix(tags_dir)
                    .ok()
                    .map(|tag| tag.to_string_lossy().replace('\\', "/"));
            }
        }
    }
    None
}

fn packed_tag_for_oid(path: &Path, oid: &str) -> Option<String> {
    let raw = std::fs::read_to_string(path).ok()?;
    let mut pending_tag = None;
    for line in raw.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some(peeled) = line.strip_prefix('^') {
            if peeled == oid {
                return pending_tag;
            }
            pending_tag = None;
            continue;
        }
        pending_tag = None;
        let mut parts = line.split_whitespace();
        let hash = parts.next()?;
        let reference = parts.next()?;
        if let Some(tag) = reference.strip_prefix("refs/tags/") {
            if hash == oid {
                return Some(tag.to_owned());
            }
            pending_tag = Some(tag.to_owned());
        }
    }
    None
}

fn find_git_dir(start: &Path) -> Option<PathBuf> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn detects_branch_from_head_ref() {
        let temp = tempdir().unwrap();
        let git = temp.path().join(".git");
        fs::create_dir_all(&git).unwrap();
        fs::write(git.join("HEAD"), "ref: refs/heads/main\n").unwrap();

        let detected = detect_from(temp.path()).unwrap();

        assert_eq!(detected.branch, "main");
    }

    #[test]
    fn detects_detached_head_at_tag() {
        let temp = tempdir().unwrap();
        let git = temp.path().join(".git");
        let tag = git.join("refs/tags");
        fs::create_dir_all(&tag).unwrap();
        fs::write(
            git.join("HEAD"),
            "1234567890abcdef1234567890abcdef12345678\n",
        )
        .unwrap();
        fs::write(
            tag.join("n8n@2.2.4"),
            "1234567890abcdef1234567890abcdef12345678\n",
        )
        .unwrap();

        let detected = detect_from(temp.path()).unwrap();

        assert_eq!(detected.branch, "n8n@2.2.4");
    }

    #[test]
    fn detects_detached_head_at_packed_tag() {
        let temp = tempdir().unwrap();
        let git = temp.path().join(".git");
        fs::create_dir_all(&git).unwrap();
        fs::write(
            git.join("HEAD"),
            "1234567890abcdef1234567890abcdef12345678\n",
        )
        .unwrap();
        fs::write(
            git.join("packed-refs"),
            "1234567890abcdef1234567890abcdef12345678 refs/tags/n8n@2.2.4\n",
        )
        .unwrap();

        let detected = detect_from(temp.path()).unwrap();

        assert_eq!(detected.branch, "n8n@2.2.4");
    }

    #[test]
    fn fast_detects_detached_head_as_short_oid() {
        let temp = tempdir().unwrap();
        let git = temp.path().join(".git");
        let tag = git.join("refs/tags");
        fs::create_dir_all(&tag).unwrap();
        fs::write(
            git.join("HEAD"),
            "1234567890abcdef1234567890abcdef12345678\n",
        )
        .unwrap();
        fs::write(
            tag.join("n8n@2.2.4"),
            "1234567890abcdef1234567890abcdef12345678\n",
        )
        .unwrap();

        let detected = detect_fast_from(temp.path()).unwrap();

        assert_eq!(detected.branch, "1234567890ab");
    }
}
