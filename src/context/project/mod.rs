mod c_cpp;
mod compose;
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
static COMPOSE_DETECTOR: compose::ComposeDetector = compose::ComposeDetector;

static DETECTORS: [&dyn ProjectDetector; 13] = [
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
    &COMPOSE_DETECTOR,
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
        "docker compose" => 12,
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
        project_name: None,
        service_count: None,
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

        let _guard = lock_env();
        let detected = with_mocked_version("rustc", "rustc 1.94.1 (abc123 2026-03-25)", || {
            detect_in(temp.path(), false)
        });

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

        let _guard = lock_env();
        let detected = with_mocked_version("node", "v20.19.6", || detect_in(temp.path(), false));

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

    #[test]
    fn compose_standard_filename_is_detected_with_summary() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("compose.yaml"),
            "name: myapp\nservices:\n  web:\n    image: nginx\n  db:\n    image: postgres\n",
        )
        .unwrap();

        let detected = detect_in(temp.path(), false);

        assert_eq!(detected.len(), 1);
        assert_eq!(detected[0].kind, "docker compose");
        assert_eq!(detected[0].project_name.as_deref(), Some("myapp"));
        assert_eq!(detected[0].service_count, Some(2));
    }

    #[test]
    fn compose_variant_and_override_filenames_are_detected() {
        let variant = tempdir().unwrap();
        fs::write(
            variant.path().join("docker-compose.dev.yml"),
            "services:\n  web:\n    image: nginx\n",
        )
        .unwrap();
        assert_eq!(detect_in(variant.path(), true)[0].kind, "docker compose");

        let override_file = tempdir().unwrap();
        fs::write(
            override_file.path().join("docker-compose.override.yaml"),
            "services:\n  worker:\n    image: busybox\n",
        )
        .unwrap();
        assert_eq!(
            detect_in(override_file.path(), true)[0].kind,
            "docker compose"
        );
    }

    #[test]
    fn compose_coexists_with_language_detector_after_languages() {
        let temp = tempdir().unwrap();
        fs::write(temp.path().join("package.json"), "{ \"name\": \"demo\" }\n").unwrap();
        fs::write(
            temp.path().join("compose.prod.yaml"),
            "name: demo\nservices:\n  app:\n    image: node\n",
        )
        .unwrap();

        let detected = detect_in(temp.path(), true);
        let kinds: Vec<_> = detected.into_iter().map(|context| context.kind).collect();

        assert_eq!(
            kinds,
            vec!["node".to_string(), "docker compose".to_string()]
        );
    }

    #[test]
    fn compose_fast_mode_skips_yaml_summary() {
        let temp = tempdir().unwrap();
        fs::write(
            temp.path().join("compose.yml"),
            "name: myapp\nservices:\n  web:\n    image: nginx\n",
        )
        .unwrap();

        let detected = detect_in(temp.path(), true);

        assert_eq!(detected[0].kind, "docker compose");
        assert_eq!(detected[0].project_name, None);
        assert_eq!(detected[0].service_count, None);
    }

    fn lock_env() -> std::sync::MutexGuard<'static, ()> {
        ENV_LOCK
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    fn with_mocked_version<T>(bin: &str, value: &str, run: impl FnOnce() -> T) -> T {
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
        let previous = std::env::var_os(&key);
        // SAFETY: tests in this module serialize environment mutation through
        // their local execution path and restore the previous value immediately.
        unsafe {
            std::env::set_var(&key, value);
        }
        let result = run();
        match previous {
            // SAFETY: restoring the prior process-global environment value is
            // safe under the same serialized test conditions above.
            Some(previous) => unsafe {
                std::env::set_var(&key, previous);
            },
            // SAFETY: removing the temporary test variable is safe for the same reason.
            None => unsafe {
                std::env::remove_var(&key);
            },
        }
        result
    }
}
