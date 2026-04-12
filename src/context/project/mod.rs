mod c_cpp;
mod csharp;
mod detector;
mod go;
mod java;
mod lua;
mod node;
mod php;
mod python;
mod r;
mod ruby;
mod rust;
mod swift;
mod version;

use self::detector::ProjectDetector;
use crate::model::ProjectContext;
use std::cmp::Reverse;
use std::path::Path;

static RUST_DETECTOR: rust::RustDetector = rust::RustDetector;
static NODE_DETECTOR: node::NodeDetector = node::NodeDetector;
static PYTHON_DETECTOR: python::PythonDetector = python::PythonDetector;
static GO_DETECTOR: go::GoDetector = go::GoDetector;
static JAVA_DETECTOR: java::JavaDetector = java::JavaDetector;
static RUBY_DETECTOR: ruby::RubyDetector = ruby::RubyDetector;
static C_CPP_DETECTOR: c_cpp::CCppDetector = c_cpp::CCppDetector;
static PHP_DETECTOR: php::PhpDetector = php::PhpDetector;
static LUA_DETECTOR: lua::LuaDetector = lua::LuaDetector;
static SWIFT_DETECTOR: swift::SwiftDetector = swift::SwiftDetector;
static R_DETECTOR: r::RDetector = r::RDetector;
static CSHARP_DETECTOR: csharp::CSharpDetector = csharp::CSharpDetector;

static DETECTORS: [&dyn ProjectDetector; 12] = [
    &RUST_DETECTOR,
    &NODE_DETECTOR,
    &PYTHON_DETECTOR,
    &GO_DETECTOR,
    &JAVA_DETECTOR,
    &RUBY_DETECTOR,
    &C_CPP_DETECTOR,
    &PHP_DETECTOR,
    &LUA_DETECTOR,
    &SWIFT_DETECTOR,
    &R_DETECTOR,
    &CSHARP_DETECTOR,
];

pub fn detect(fast: bool) -> Vec<ProjectContext> {
    let cwd = match std::env::current_dir() {
        Ok(cwd) => cwd,
        Err(_) => return Vec::new(),
    };
    detect_in(&cwd, fast)
}

fn detect_in(cwd: &Path, fast: bool) -> Vec<ProjectContext> {
    let mut contexts: Vec<_> = DETECTORS
        .iter()
        .flat_map(|detector| detector.detect(cwd, fast))
        .collect();
    contexts.sort_by(|left, right| {
        project_priority(&left.kind)
            .cmp(&project_priority(&right.kind))
            .then_with(|| left.kind.cmp(&right.kind))
    });
    contexts
}

pub fn project_priority(kind: &str) -> usize {
    match kind {
        "rust" => 0,
        "node" => 1,
        "python" => 2,
        "go" => 3,
        "java" => 4,
        "ruby" => 5,
        "csharp" => 6,
        "swift" => 7,
        "php" => 8,
        "lua" => 9,
        "r" => 10,
        "c/cpp" => 11,
        _ => usize::MAX,
    }
}

pub(super) fn project_context(
    kind: &str,
    version: Option<String>,
    details: impl IntoIterator<Item = String>,
) -> ProjectContext {
    let mut details: Vec<String> = details.into_iter().collect();
    details.sort_by_key(|detail| Reverse(detail_priority(detail)));
    ProjectContext {
        kind: kind.into(),
        version,
        details,
    }
}

fn detail_priority(detail: &str) -> usize {
    match detail {
        ".venv" => 100,
        "venv" => 99,
        "pnpm" => 90,
        "yarn" => 89,
        "npm" => 88,
        "turbo" => 80,
        "nx" => 79,
        "gradle" => 70,
        "maven" => 69,
        _ => 0,
    }
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

        let detected = detect_in(temp.path(), false);

        restore_path(previous_path);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "rust");
        assert_eq!(detected[0].version.as_deref(), Some("1.94.1"));
    }

    #[test]
    fn rust_detector_works_in_fast_mode() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();

        let detected = detect_in(temp.path(), true);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "rust");
        assert_eq!(detected[0].version, None);
    }

    #[test]
    fn node_detector_works_in_normal_mode() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();
        fs::write(temp.path().join("pnpm-lock.yaml"), "lockfileVersion: 9\n").unwrap();
        fs::write(temp.path().join("turbo.json"), "{}\n").unwrap();
        let bin_dir = temp.path().join("bin");
        write_fake_command(&bin_dir, "node", "v20.19.6");

        let _guard = ENV_LOCK.lock().unwrap();
        let previous_path = std::env::var_os("PATH");
        unsafe {
            std::env::set_var("PATH", prefixed_path(&bin_dir, previous_path.as_deref()));
        }

        let detected = detect_in(temp.path(), false);

        restore_path(previous_path);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "node");
        assert_eq!(detected[0].version.as_deref(), Some("20.19.6"));
        assert_eq!(
            detected[0].details,
            vec!["pnpm".to_string(), "turbo".to_string()]
        );
    }

    #[test]
    fn node_detector_works_in_fast_mode() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();

        let detected = detect_in(temp.path(), true);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "node");
        assert_eq!(detected[0].version, None);
    }

    #[test]
    fn detector_registry_supports_multiple_project_contexts() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();
        fs::write(temp.path().join("requirements.txt"), "requests==2.0.0\n").unwrap();

        let detected = detect_in(temp.path(), true);

        let kinds: Vec<_> = detected.into_iter().map(|context| context.kind).collect();
        assert_eq!(kinds, vec!["node".to_string(), "python".to_string()]);
    }

    #[test]
    fn detector_priority_is_fixed_not_detection_order() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("requirements.txt"), "requests==2.0.0\n").unwrap();
        fs::write(
            temp.path().join("Cargo.toml"),
            "[package]\nname = \"demo\"\n",
        )
        .unwrap();

        let detected = detect_in(temp.path(), true);
        let kinds: Vec<_> = detected.into_iter().map(|context| context.kind).collect();

        assert_eq!(kinds, vec!["rust".to_string(), "python".to_string()]);
    }

    #[test]
    fn python_detector_includes_virtualenv_name() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("requirements.txt"), "requests==2.0.0\n").unwrap();
        fs::create_dir_all(temp.path().join(".venv")).unwrap();

        let detected = detect_in(temp.path(), true);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "python");
        assert_eq!(detected[0].details, vec![".venv".to_string()]);
    }

    #[test]
    fn java_detector_distinguishes_gradle_and_maven() {
        let gradle = tempdir().unwrap();
        fs::write(gradle.path().join("build.gradle"), "plugins {}\n").unwrap();
        let gradle_detected = detect_in(gradle.path(), true);
        assert_eq!(gradle_detected[0].kind, "java");
        assert_eq!(gradle_detected[0].details, vec!["gradle".to_string()]);

        let maven = tempdir().unwrap();
        fs::write(maven.path().join("pom.xml"), "<project />\n").unwrap();
        let maven_detected = detect_in(maven.path(), true);
        assert_eq!(maven_detected[0].kind, "java");
        assert_eq!(maven_detected[0].details, vec!["maven".to_string()]);
    }

    #[test]
    fn c_cpp_only_triggers_on_build_signals() {
        let source_only = tempdir().unwrap();
        fs::write(
            source_only.path().join("main.cpp"),
            "int main() { return 0; }\n",
        )
        .unwrap();
        let detected = detect_in(source_only.path(), true);
        assert!(detected.is_empty());

        let with_cmake = tempdir().unwrap();
        fs::write(with_cmake.path().join("CMakeLists.txt"), "project(demo)\n").unwrap();
        let detected = detect_in(with_cmake.path(), true);
        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "c/cpp");
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
