use assert_cmd::Command;
use predicates::prelude::*;

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
