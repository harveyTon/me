use std::path::Path;
use std::process::Command;

pub fn command_version(bin: &str, arg: &str) -> Option<String> {
    #[cfg(test)]
    if let Some(mocked) = mocked_version(bin) {
        return Some(mocked);
    }

    if !Path::new(bin).exists() && which::which(bin).is_err() {
        return None;
    }
    let output = Command::new(bin).arg(arg).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let raw = if !stdout.is_empty() { stdout } else { stderr };
    let raw = output
        .status
        .success()
        .then_some(raw)
        .filter(|s| !s.is_empty())?;
    Some(extract_version(&raw))
}

pub fn command_version_any(bin: &str, args: &[&str]) -> Option<String> {
    #[cfg(test)]
    if let Some(mocked) = mocked_version(bin) {
        return Some(mocked);
    }

    if !Path::new(bin).exists() && which::which(bin).is_err() {
        return None;
    }
    let output = Command::new(bin).args(args).output().ok()?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    let raw = if !stdout.is_empty() { stdout } else { stderr };
    let raw = output
        .status
        .success()
        .then_some(raw)
        .filter(|s| !s.is_empty())?;
    Some(extract_version(&raw))
}

pub fn extract_version(raw: &str) -> String {
    raw.split_whitespace()
        .map(trim_token)
        .find_map(normalize_version_token)
        .unwrap_or_else(|| raw.to_owned())
}

pub fn major_version(raw: &str) -> String {
    raw.split('.').next().unwrap_or(raw).to_owned()
}

fn trim_token(part: &str) -> &str {
    part.trim_matches(|c| matches!(c, '"' | '\'' | ',' | '(' | ')' | '[' | ']'))
}

fn normalize_version_token(part: &str) -> Option<String> {
    if part.chars().next().is_some_and(|c| c.is_ascii_digit()) {
        return Some(part.to_owned());
    }
    for prefix in ["v", "go"] {
        if let Some(trimmed) = part.strip_prefix(prefix)
            && trimmed.chars().next().is_some_and(|c| c.is_ascii_digit())
        {
            return Some(trimmed.to_owned());
        }
    }
    None
}

#[cfg(test)]
fn mocked_version(bin: &str) -> Option<String> {
    let key = format!(
        "ME_TEST_VERSION_{}",
        bin.chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() {
                    ch.to_ascii_uppercase()
                } else {
                    '_'
                }
            })
            .collect::<String>()
    );
    std::env::var(key).ok().map(|value| extract_version(&value))
}

#[cfg(test)]
mod tests {
    use super::{extract_version, major_version};

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

    #[test]
    fn strips_go_prefix() {
        assert_eq!(
            extract_version("go version go1.24.0 darwin/arm64"),
            "1.24.0"
        );
    }

    #[test]
    fn trims_quoted_java_version() {
        assert_eq!(
            extract_version("openjdk version \"21.0.2\" 2024-01-16"),
            "21.0.2"
        );
    }

    #[test]
    fn keeps_major_component() {
        assert_eq!(major_version("21.0.2"), "21");
    }
}
