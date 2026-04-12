use std::path::Path;
use std::process::Command;

pub fn command_version(bin: &str, arg: &str) -> Option<String> {
    if !Path::new(bin).exists() && which::which(bin).is_err() {
        return None;
    }
    let output = Command::new(bin).arg(arg).output().ok()?;
    let raw = output
        .status
        .success()
        .then(|| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|s| !s.is_empty())?;
    Some(extract_version(&raw))
}

pub fn extract_version(raw: &str) -> String {
    raw.split_whitespace()
        .find_map(normalize_version_token)
        .unwrap_or_else(|| raw.to_owned())
}

fn normalize_version_token(part: &str) -> Option<String> {
    if part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Some(part.to_owned());
    }
    let trimmed = part.strip_prefix('v')?;
    trimmed
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_digit())
        .then(|| trimmed.to_owned())
}

#[cfg(test)]
mod tests {
    use super::extract_version;

    #[test]
    fn strips_node_v_prefix() {
        assert_eq!(extract_version("v20.19.6"), "20.19.6");
        assert_eq!(extract_version("node v20.19.6"), "20.19.6");
    }

    #[test]
    fn keeps_rust_style_version_plain() {
        assert_eq!(
            extract_version("rustc 1.94.1 (abc123 2026-03-25)"),
            "1.94.1"
        );
    }
}
