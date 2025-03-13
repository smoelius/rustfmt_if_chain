use assert_cmd::Command;
use regex::Regex;
use similar_asserts::SimpleDiff;
use std::{
    env::remove_var,
    fs::{read_to_string, write},
    path::Path,
    str::FromStr,
};
use tempfile::tempdir;

#[ctor::ctor]
fn initialize() {
    unsafe {
        remove_var("CARGO_TERM_COLOR");
    }
}

#[test]
fn clippy() {
    Command::new("cargo")
        // smoelius: Remove `CARGO` environment variable to work around:
        // https://github.com/rust-lang/rust/pull/131729
        .env_remove("CARGO")
        .args([
            "+nightly",
            "clippy",
            "--all-features",
            "--all-targets",
            "--",
            "--deny=warnings",
            "--warn=clippy::pedantic",
            "--allow=clippy::struct-field-names",
        ])
        .assert()
        .success();
}

#[test]
fn dylint() {
    Command::new("cargo")
        .args(["dylint", "--all", "--", "--all-features", "--all-targets"])
        .env("DYLINT_RUSTFLAGS", "--deny warnings")
        .assert()
        .success();
}

#[test]
fn license() {
    let re = Regex::new(r"^[^:]*\b(Apache-2.0|BSD-3-Clause|MIT)\b").unwrap();

    for line in std::str::from_utf8(
        &Command::new("cargo")
            .arg("license")
            .assert()
            .get_output()
            .stdout,
    )
    .unwrap()
    .lines()
    {
        assert!(re.is_match(line), "{line:?} does not match");
    }
}

#[test]
fn markdown_link_check() {
    let tempdir = tempdir().unwrap();

    // smoelius: Pin `markdown-link-check` to version 3.11 until the following issue is resolved:
    // https://github.com/tcort/markdown-link-check/issues/304
    Command::new("npm")
        .args(["install", "markdown-link-check@3.11"])
        .current_dir(&tempdir)
        .assert()
        .success();

    let readme_md = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");

    Command::new("npx")
        .args(["markdown-link-check", &readme_md.to_string_lossy()])
        .current_dir(&tempdir)
        .assert()
        .success();
}

#[test]
fn prettier() {
    let tempdir = tempdir().unwrap();

    Command::new("npm")
        .args(["install", "prettier"])
        .current_dir(&tempdir)
        .assert()
        .success();

    Command::new("npx")
        .args([
            "prettier",
            "--check",
            &format!("{}/**/*.md", env!("CARGO_MANIFEST_DIR")),
            &format!("{}/**/*.yml", env!("CARGO_MANIFEST_DIR")),
            &format!("!{}/target/**", env!("CARGO_MANIFEST_DIR")),
        ])
        .current_dir(&tempdir)
        .assert()
        .success();
}

#[test]
fn readme_contains_usage() {
    let readme = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");

    let contents = read_to_string(readme).unwrap();

    let stdout = Command::cargo_bin("rustfmt_if_chain")
        .unwrap()
        .arg("--help")
        .assert()
        .get_output()
        .stdout
        .clone();

    let usage = std::str::from_utf8(&stdout).unwrap();

    assert!(contents.contains(usage));
}

#[test]
fn sort() {
    Command::new("cargo")
        .args(["sort", "--check"])
        .assert()
        .success();
}

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn supply_chain() {
    Command::new("cargo")
        .args(["supply-chain", "update"])
        .assert()
        .success();

    let assert = Command::new("cargo")
        .args(["supply-chain", "json", "--no-dev"])
        .assert()
        .success();

    let stdout_actual = std::str::from_utf8(&assert.get_output().stdout).unwrap();
    let value = serde_json::Value::from_str(stdout_actual).unwrap();
    let stdout_normalized = serde_json::to_string_pretty(&value).unwrap();

    let path = Path::new("tests/supply_chain.json");

    let stdout_expected = read_to_string(path).unwrap();

    if enabled("BLESS") {
        write(path, stdout_normalized).unwrap();
    } else {
        assert!(
            stdout_expected == stdout_normalized,
            "{}",
            SimpleDiff::from_str(&stdout_expected, &stdout_normalized, "left", "right")
        );
    }
}

#[test]
fn udeps() {
    Command::new("cargo")
        .args(["+nightly", "udeps", "--all-features", "--all-targets"])
        .assert()
        .success();
}

#[must_use]
pub fn enabled(key: &str) -> bool {
    std::env::var(key).is_ok_and(|value| value != "0")
}
