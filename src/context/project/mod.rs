mod detector;
mod node;
mod rust_project;
mod version;

use self::detector::ProjectDetector;
use crate::model::ProjectContext;
use std::path::Path;

static RUST_DETECTOR: rust_project::RustDetector = rust_project::RustDetector;
static NODE_DETECTOR: node::NodeDetector = node::NodeDetector;
static DETECTORS: [&dyn ProjectDetector; 2] = [&RUST_DETECTOR, &NODE_DETECTOR];

pub fn detect(fast: bool) -> Option<ProjectContext> {
    let cwd = std::env::current_dir().ok()?;
    detect_in(&cwd, fast)
}

fn detect_in(cwd: &Path, fast: bool) -> Option<ProjectContext> {
    DETECTORS
        .iter()
        .find_map(|detector| detector.detect(cwd, fast))
}

#[cfg(test)]
mod tests {
    use super::detect_in;
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::sync::Mutex;
    use tempfile::tempdir;

    static ENV_LOCK: Mutex<()> = Mutex::new(());

    #[test]
    fn rust_detector_works_in_normal_mode() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        let bin_dir = temp.path().join("bin");
        write_fake_command(&bin_dir, "rustc", "rustc 1.94.1 (abc123 2026-03-25)");

        let _guard = ENV_LOCK.lock().unwrap();
        let previous_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", prefixed_path(&bin_dir, previous_path.as_deref()));
        }

        let detected = detect_in(temp.path(), false).unwrap();

        restore_path(previous_path);

        assert_eq!(detected.kind, "rust");
        assert_eq!(detected.version.as_deref(), Some("1.94.1"));
    }

    #[test]
    fn rust_detector_works_in_fast_mode() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();

        let detected = detect_in(temp.path(), true).unwrap();

        assert_eq!(detected.kind, "rust");
        assert_eq!(detected.version, None);
    }

    #[test]
    fn node_detector_works_in_normal_mode() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();
        let bin_dir = temp.path().join("bin");
        write_fake_command(&bin_dir, "node", "v20.19.6");

        let _guard = ENV_LOCK.lock().unwrap();
        let previous_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", prefixed_path(&bin_dir, previous_path.as_deref()));
        }

        let detected = detect_in(temp.path(), false).unwrap();

        restore_path(previous_path);

        assert_eq!(detected.kind, "node");
        assert_eq!(detected.version.as_deref(), Some("20.19.6"));
    }

    #[test]
    fn node_detector_works_in_fast_mode() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();

        let detected = detect_in(temp.path(), true).unwrap();

        assert_eq!(detected.kind, "node");
        assert_eq!(detected.version, None);
    }

    #[test]
    fn detector_registry_priority_prefers_rust_over_node() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();
        let bin_dir = temp.path().join("bin");
        write_fake_command(&bin_dir, "rustc", "rustc 1.94.1 (abc123 2026-03-25)");
        write_fake_command(&bin_dir, "node", "v20.19.6");

        let _guard = ENV_LOCK.lock().unwrap();
        let previous_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", prefixed_path(&bin_dir, previous_path.as_deref()));
        }

        let detected = detect_in(temp.path(), false).unwrap();

        restore_path(previous_path);

        assert_eq!(detected.kind, "rust");
        assert_eq!(detected.version.as_deref(), Some("1.94.1"));
    }

    fn prefixed_path(bin_dir: &Path, existing: Option<&std::ffi::OsStr>) -> std::ffi::OsString {
        let mut paths = vec![PathBuf::from(bin_dir)];
        if let Some(existing) = existing {
            paths.extend(std::env::split_paths(existing));
        }
        std::env::join_paths(paths).unwrap()
    }

    fn restore_path(previous_path: Option<std::ffi::OsString>) {
        match previous_path {
            Some(path) => unsafe {
                std::env::set_var("PATH", path);
            },
            None => unsafe {
                std::env::remove_var("PATH");
            },
        }
    }

    fn write_fake_command(bin_dir: &Path, name: &str, output: &str) {
        fs::create_dir_all(bin_dir).unwrap();
        if cfg!(windows) {
            let path = bin_dir.join(format!("{name}.cmd"));
            fs::write(&path, format!("@echo {output}\r\n")).unwrap();
        } else {
            let path = bin_dir.join(name);
            fs::write(&path, format!("#!/bin/sh\necho '{output}'\n")).unwrap();
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mut perms = fs::metadata(&path).unwrap().permissions();
                perms.set_mode(0o755);
                fs::set_permissions(&path, perms).unwrap();
            }
        }
    }
}
