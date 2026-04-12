use assert_cmd::Command;
use assert_cmd::assert::OutputAssertExt;
use predicates::prelude::*;
use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
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
    assert!(man_page.contains("me install"));
    assert!(man_page.contains("me uninstall"));
    assert!(man_page.contains("me update"));
    assert!(man_page.contains("\\-\\-yes"));
    assert!(man_page.contains("\\-\\-check"));
    assert!(man_page.contains("-h"));
}

#[test]
fn version_consistency_script_passes() {
    std::process::Command::new("bash")
        .arg("scripts/check-version-consistency.sh")
        .assert()
        .success();
}

#[test]
fn shell_integration_help_keeps_install_and_uninstall_flags_separate() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["install", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--login"))
        .stdout(predicate::str::contains("--interactive"))
        .stdout(predicate::str::contains("--yes").not());

    Command::cargo_bin("me")
        .unwrap()
        .args(["uninstall", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--yes"))
        .stdout(predicate::str::contains("--login").not())
        .stdout(predicate::str::contains("--interactive").not());
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
        .stdout(predicate::str::contains("\"projects\""))
        .stdout(predicate::str::contains("\"kind\": \"rust\""))
        .stdout(predicate::str::contains("\"version\":"));

    Command::cargo_bin("me")
        .unwrap()
        .args(["--fast", "--json"])
        .current_dir(repo.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("\"projects\""))
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

#[test]
fn install_non_interactive_writes_managed_block() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".zshrc");

    Command::cargo_bin("me")
        .unwrap()
        .args([
            "install",
            "--non-interactive",
            "--shell",
            "zsh",
            "--login",
            "full",
            "--interactive",
            "compact",
            "--file",
            target.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("installed"))
        .stdout(predicate::str::contains(target.display().to_string()))
        .stdout(predicate::str::contains(
            "Installed me shell integration for zsh",
        ))
        .stdout(predicate::str::contains(format!(
            "- login: full ({})",
            target.display()
        )))
        .stdout(predicate::str::contains(format!(
            "- interactive: compact ({})",
            target.display()
        )))
        .stdout(predicate::str::contains(
            "Restart or reopen your shell to start using it.",
        ));

    let contents = fs::read_to_string(&target).unwrap();
    assert_eq!(contents.matches("# >>> me install >>>").count(), 1);
    assert!(
        contents.contains("# me-managed: shell=zsh login=full interactive=compact version=v0.3.3")
    );
    assert!(contents.contains("me\n"));
    assert!(contents.contains("me --compact"));
}

#[test]
fn reinstall_replaces_existing_managed_block_without_duplication() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".zshrc");

    run_install(&target, "compact");
    run_install(&target, "none");

    let contents = fs::read_to_string(&target).unwrap();
    assert_eq!(contents.matches("# >>> me install >>>").count(), 1);
    assert!(contents.contains("# me-managed: shell=zsh login=full interactive=none"));
    assert!(!contents.contains("interactive=compact"));
}

#[test]
fn uninstall_non_interactive_removes_only_managed_block_from_file() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".zshrc");
    fs::write(&target, "before\n").unwrap();
    run_install(&target, "compact");
    fs::write(
        &target,
        format!("{}\nafter\n", fs::read_to_string(&target).unwrap()),
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args([
            "uninstall",
            "--non-interactive",
            "--file",
            target.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stdout(predicate::str::contains("removed"))
        .stdout(predicate::str::contains(
            "Removed me shell integration from 1 file(s).",
        ))
        .stdout(predicate::str::contains(
            "Restart or reopen your shell to finish uninstalling.",
        ));

    let contents = fs::read_to_string(&target).unwrap();
    assert!(contents.contains("before"));
    assert!(contents.contains("after"));
    assert!(!contents.contains("# >>> me install >>>"));
    assert!(!contents.contains("# me-managed:"));
}

#[test]
fn uninstall_non_interactive_global_requires_yes() {
    let home = tempdir().unwrap();
    let zshrc = home.path().join(".zshrc");
    fs::write(
        &zshrc,
        "# >>> me install >>>\n# me-managed: shell=zsh login=none interactive=compact version=v0.3.3\nme --compact\n# <<< me install <<<\n",
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args(["uninstall", "--non-interactive"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .assert()
        .failure()
        .stderr(predicate::str::contains("--yes"));

    assert!(
        fs::read_to_string(&zshrc)
            .unwrap()
            .contains("# >>> me install >>>")
    );
}

#[test]
fn uninstall_non_interactive_yes_removes_all_managed_blocks() {
    let home = tempdir().unwrap();
    let zshrc = home.path().join(".zshrc");
    let bashrc = home.path().join(".bashrc");
    fs::write(
        &zshrc,
        "# >>> me install >>>\n# me-managed: shell=zsh login=none interactive=compact version=v0.3.3\nme --compact\n# <<< me install <<<\n",
    )
    .unwrap();
    fs::write(
        &bashrc,
        "# >>> me install >>>\n# me-managed: shell=bash login=none interactive=compact version=v0.3.3\nme --compact\n# <<< me install <<<\n",
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args(["uninstall", "--non-interactive", "--yes"])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Removed me shell integration from 2 file(s).",
        ));

    assert!(
        !fs::read_to_string(&zshrc)
            .unwrap()
            .contains("# >>> me install >>>")
    );
    assert!(
        !fs::read_to_string(&bashrc)
            .unwrap()
            .contains("# >>> me install >>>")
    );
}

#[test]
fn install_interactive_uses_natural_prompt_wording() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".zshrc");

    Command::cargo_bin("me")
        .unwrap()
        .args([
            "install",
            "--shell",
            "zsh",
            "--file",
            target.to_str().unwrap(),
        ])
        .write_stdin("3\n3\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "How should me run in login shells?",
        ))
        .stdout(predicate::str::contains(
            "How should me run in interactive shells?",
        ));
}

#[test]
fn uninstall_does_not_remove_partial_or_unmanaged_me_snippets() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".zshrc");
    fs::write(
        &target,
        "# me-managed: shell=zsh login=full interactive=compact version=v0.3.3\nme --compact\n",
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args([
            "uninstall",
            "--non-interactive",
            "--file",
            target.to_str().unwrap(),
        ])
        .assert()
        .success()
        .stderr(predicate::str::contains("manual cleanup"));

    let contents = fs::read_to_string(&target).unwrap();
    assert!(contents.contains("# me-managed:"));
    assert!(contents.contains("me --compact"));
}

#[test]
fn install_non_interactive_refuses_prompt_when_prompt_looks_complex() {
    let dir = tempdir().unwrap();
    let target = dir.path().join(".zshrc");
    fs::write(&target, "eval \"$(starship init zsh)\"\n").unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args([
            "install",
            "--non-interactive",
            "--shell",
            "zsh",
            "--login",
            "none",
            "--interactive",
            "prompt",
            "--file",
            target.to_str().unwrap(),
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("prompt integration looks unsafe"));

    let contents = fs::read_to_string(&target).unwrap();
    assert!(!contents.contains("# >>> me install >>>"));
}

#[test]
fn install_non_interactive_warns_about_other_shell_integrations() {
    let home = tempdir().unwrap();
    let zprofile = home.path().join(".zprofile");
    let bashrc = home.path().join(".bashrc");
    fs::write(
        &bashrc,
        "# >>> me install >>>\n# me-managed: shell=bash login=full interactive=none version=v0.3.3\nme\n# <<< me install <<<\n",
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args([
            "install",
            "--non-interactive",
            "--shell",
            "zsh",
            "--login",
            "full",
            "--interactive",
            "none",
        ])
        .env("HOME", home.path())
        .env("XDG_CONFIG_HOME", home.path().join(".config"))
        .assert()
        .success()
        .stderr(predicate::str::contains("existing me integration"))
        .stdout(predicate::str::contains("~/.zprofile"));

    assert!(fs::read_to_string(&bashrc).unwrap().contains("shell=bash"));
    assert!(fs::read_to_string(&zprofile).unwrap().contains("shell=zsh"));
}

fn run_install(target: &Path, interactive: &str) {
    Command::cargo_bin("me")
        .unwrap()
        .args([
            "install",
            "--non-interactive",
            "--shell",
            "zsh",
            "--login",
            "full",
            "--interactive",
            interactive,
            "--file",
            target.to_str().unwrap(),
        ])
        .assert()
        .success();
}

#[test]
fn home_directory_still_shows_project_context_when_detected() {
    let home = tempdir().unwrap();
    fs::write(
        home.path().join("package.json"),
        "{ \"name\": \"home-demo\" }\n",
    )
    .unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args(["--compact", "--fast", "--no-color"])
        .current_dir(home.path())
        .env("HOME", home.path())
        .assert()
        .success()
        .stdout(predicate::str::contains(" · node"))
        .stdout(predicate::str::contains(" · local · "));
}

#[test]
fn update_check_reports_current_and_latest_versions() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["update", "--check"])
        .env("ME_UPDATE_LATEST_VERSION", "9.9.9")
        .env("ME_UPDATE_SOURCE", "unknown")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "current: {}",
            env!("CARGO_PKG_VERSION")
        )))
        .stdout(predicate::str::contains("latest: 9.9.9"))
        .stdout(predicate::str::contains("install source: unknown"))
        .stdout(predicate::str::contains("update available"));
}

#[test]
fn update_interactive_shows_versions_before_prompting() {
    Command::cargo_bin("me")
        .unwrap()
        .arg("update")
        .env("ME_UPDATE_LATEST_VERSION", "9.9.9")
        .env("ME_UPDATE_SOURCE", "unknown")
        .write_stdin("n\n")
        .assert()
        .success()
        .stdout(predicate::str::contains(format!(
            "current: {}",
            env!("CARGO_PKG_VERSION")
        )))
        .stdout(predicate::str::contains("latest: 9.9.9"))
        .stdout(predicate::str::contains("Upgrade now? [Y/n]"))
        .stdout(predicate::str::contains("aborted"));
}

#[test]
fn update_non_interactive_unknown_source_fails_safely() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["update", "--non-interactive"])
        .env("ME_UPDATE_LATEST_VERSION", "9.9.9")
        .env("ME_UPDATE_SOURCE", "unknown")
        .assert()
        .failure()
        .stderr(predicate::str::contains(
            "unsupported or unknown install source",
        ));
}

#[test]
fn update_non_interactive_homebrew_path_can_be_dry_run() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["update", "--non-interactive"])
        .env("ME_UPDATE_LATEST_VERSION", "9.9.9")
        .env("ME_UPDATE_SOURCE", "homebrew")
        .env("ME_UPDATE_DRY_RUN", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("install source: homebrew"))
        .stdout(predicate::str::contains("would run: brew update"))
        .stdout(predicate::str::contains("would run: brew upgrade me"));
}

#[test]
fn update_check_does_not_run_update_actions_for_homebrew() {
    Command::cargo_bin("me")
        .unwrap()
        .args(["update", "--check"])
        .env("ME_UPDATE_LATEST_VERSION", "9.9.9")
        .env("ME_UPDATE_SOURCE", "homebrew")
        .env("ME_UPDATE_DRY_RUN", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("install source: homebrew"))
        .stdout(predicate::str::contains("update available"))
        .stdout(predicate::str::contains("would run: brew update").not())
        .stdout(predicate::str::contains("would run: brew upgrade me").not());
}

#[test]
fn update_non_interactive_release_binary_replaces_target_safely() {
    let dir = tempdir().unwrap();
    let current = dir.path().join(if cfg!(windows) { "me.exe" } else { "me" });
    let artifact = dir.path().join("replacement");
    fs::write(&current, "old").unwrap();
    fs::write(&artifact, "new").unwrap();

    Command::cargo_bin("me")
        .unwrap()
        .args(["update", "--non-interactive"])
        .env("ME_UPDATE_LATEST_VERSION", "9.9.9")
        .env("ME_UPDATE_SOURCE", "release")
        .env("ME_UPDATE_EXE", &current)
        .env("ME_UPDATE_RELEASE_ARTIFACT", &artifact)
        .assert()
        .success()
        .stdout(predicate::str::contains("install source: release binary"))
        .stdout(predicate::str::contains("updated release binary"));

    assert_eq!(fs::read_to_string(&current).unwrap(), "new");
}
