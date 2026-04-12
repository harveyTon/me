use assert_cmd::Command;
use predicates::prelude::*;
use std::{
    fs,
    io::{BufRead, BufReader},
    process::Stdio,
    thread,
    time::{Duration, Instant},
};
use tempfile::tempdir;

#[test]
fn plain_outputs_user_at_host() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--plain")
        .assert()
        .success()
        .stdout(predicate::str::contains("@"));
}

#[test]
fn help_is_available_as_long_flag() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Output:"))
        .stdout(predicate::str::contains("Fields:"))
        .stdout(predicate::str::contains("Examples:"))
        .stdout(predicate::str::contains("--help"))
        .stdout(predicate::str::contains("-h").and(predicate::str::contains("Select host")));
}

#[test]
fn man_page_documents_core_usage() {
    let man_page = std::fs::read_to_string("man/man1/me.1").unwrap();
    assert!(man_page.contains(".TH ME 1"));
    assert!(man_page.contains("\\-\\-compact"));
    assert!(man_page.contains("\\-\\-json"));
    assert!(man_page.contains("-h"));
}

#[test]
fn json_subset_keeps_predictable_keys() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["--json", "-u", "-h"])
        .assert()
        .success()
        .stdout(predicate::str::contains("\"user\""))
        .stdout(predicate::str::contains("\"host\""))
        .stdout(predicate::str::contains("\"uid\"").not());
}

#[test]
fn config_format_uses_equals() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["--format", "config", "-u"])
        .assert()
        .success()
        .stdout(predicate::str::contains("user = "));
}

#[test]
fn compact_single_field_subset_prints_value() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["--compact", "-u"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not())
        .stdout(predicate::str::contains("rust").not());
}

#[test]
fn selector_does_not_leak_project_context() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("-n")
        .assert()
        .success()
        .stdout(predicate::str::contains("network: "))
        .stdout(predicate::str::contains("--- context ---").not())
        .stdout(predicate::str::contains("project:").not());
}

#[test]
fn json_omits_null_values() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--json")
        .assert()
        .success()
        .stdout(predicate::str::contains("null").not());
}

#[test]
fn fast_flag_produces_output() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("--fast")
        .assert()
        .success()
        .stdout(predicate::str::is_empty().not());
}

#[test]
fn fast_json_still_includes_pwd() {
    let repo = tempdir().unwrap();
    let home = tempdir().unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args(["--fast", "--json"])
        .current_dir(repo.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"pwd\""))
        .stdout(predicate::str::contains("\"display\":"))
        .stdout(predicate::str::contains("\"raw\":"));
}

#[test]
fn fast_git_context_uses_short_oid_for_detached_head() {
    let repo = tempdir().unwrap();
    let home = tempdir().unwrap();
    let git = repo.path().join(".git");
    let tag = git.join("refs/tags");
    fs::create_dir_all(&tag).unwrap();
    fs::write(repo.path().join("package.json"), "{}").unwrap();
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

    Command::cargo_bin("me")
        .unwrap()
        .args(["--fast", "--no-color"])
        .current_dir(repo.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("git(1234567890ab)"))
        .stdout(predicate::str::contains("git(n8n@2.2.4)").not());
}

#[test]
fn json_project_version_is_present_without_fast_and_omitted_with_fast() {
    let repo = tempdir().unwrap();
    let home = tempdir().unwrap();
    fs::write(
        repo.path().join("Cargo.toml"),
        "[package]\nname = \"demo\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .arg("--json")
        .current_dir(repo.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"project\""))
        .stdout(predicate::str::contains("\"kind\": \"rust\""))
        .stdout(predicate::str::contains("\"version\":"));

    Command::cargo_bin("me")
        .unwrap()
        .args(["--fast", "--json"])
        .current_dir(repo.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"project\""))
        .stdout(predicate::str::contains("\"kind\": \"rust\""))
        .stdout(predicate::str::contains("\"version\"").not());
}

#[test]
fn json_snapshot_mode_outputs_one_valid_json_object() {
    let output = Command::cargo_bin("me")
        .unwrap()
        .arg("--json")
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    serde_json::from_str::<serde_json::Value>(&stdout).unwrap();
}

#[test]
fn watch_json_outputs_ndjson_lines_without_ansi() {
    let home = tempdir().unwrap();
    let mut child = std::process::Command::new(Command::cargo_bin("me").unwrap().get_program())
        .args(["--watch", "--json", "--interval", "1"])
        .env("HOME", home.path())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let stdout = child.stdout.take().unwrap();
    let mut reader = BufReader::new(stdout);
    let mut lines = Vec::new();

    for _ in 0..2 {
        let mut line = String::new();
        reader.read_line(&mut line).unwrap();
        lines.push(line);
    }

    let _ = child.kill();
    let _ = child.wait();

    assert_eq!(lines.len(), 2);
    for line in lines {
        assert!(!line.contains('\u{1b}'));
        serde_json::from_str::<serde_json::Value>(line.trim_end()).unwrap();
    }
}

#[test]
fn watch_json_exits_cleanly_when_pipe_closes() {
    let home = tempdir().unwrap();
    let mut child = std::process::Command::new(Command::cargo_bin("me").unwrap().get_program())
        .args(["--watch", "--json", "--interval", "1"])
        .env("HOME", home.path())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();

    let mut reader = BufReader::new(child.stdout.take().unwrap());
    let mut first_line = String::new();
    reader.read_line(&mut first_line).unwrap();
    drop(reader);

    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait().unwrap() {
            assert!(status.success());
            break;
        }
        if Instant::now() >= deadline {
            let _ = child.kill();
            panic!("watch json process did not exit after pipe closed");
        }
        thread::sleep(Duration::from_millis(50));
    }
}
