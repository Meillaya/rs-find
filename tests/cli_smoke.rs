use std::process::Command;

mod support;

use support::TestDir;

fn bin() -> &'static str {
    env!("CARGO_BIN_EXE_rs-find")
}

#[test]
fn missing_args_prints_usage_and_fails() {
    let output = Command::new(bin()).output().expect("binary should run");
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Usage: rs-find"));
}

#[test]
fn cli_searches_successfully() {
    let fixture = TestDir::new();
    fixture.create_file("alpha/target.txt", "hello");

    let output = Command::new(bin())
        .arg("target")
        .arg(fixture.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("target.txt"));
}

#[test]
fn path_flag_changes_matching_semantics() {
    let fixture = TestDir::new();
    fixture.create_file("alpha/target.txt", "hello");

    let output = Command::new(bin())
        .arg("--path")
        .arg("alpha/target")
        .arg(fixture.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("target.txt"));
}

#[test]
fn exclude_hidden_flag_skips_hidden_entries() {
    let fixture = TestDir::new();
    fixture.create_file(".hidden-target.txt", "hello");
    fixture.create_file("visible-target.txt", "hello");

    let output = Command::new(bin())
        .arg("--exclude-hidden")
        .arg("target")
        .arg(fixture.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("visible-target.txt"));
    assert!(!stdout.contains(".hidden-target.txt"));
}

#[test]
fn cross_filesystems_flag_is_accepted() {
    let fixture = TestDir::new();
    fixture.create_file("alpha/target.txt", "hello");

    let output = Command::new(bin())
        .arg("--cross-filesystems")
        .arg("target")
        .arg(fixture.path())
        .output()
        .expect("binary should run");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("target.txt"));
}
