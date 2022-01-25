use assert_cmd::Command;
use std::ffi::OsStr;
use tempfile::tempdir_in;
use walkdir::WalkDir;

const CLIPPY_URL: &str = "https://github.com/rust-lang/rust-clippy";

#[test]
fn clippy() {
    let tempdir = tempdir_in(".").unwrap();

    Command::new("git")
        .args(&["clone", CLIPPY_URL, &tempdir.path().to_string_lossy()])
        .assert()
        .success();

    for entry in WalkDir::new(tempdir.path()) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        Command::cargo_bin("rustfmt_if_chain")
            .unwrap()
            .args(&["--preformat-failure-is-warning", &path.to_string_lossy()])
            .assert()
            .success();
    }

    Command::new("cargo")
        .args(&["build", "--tests"])
        .current_dir(tempdir.path())
        .env_remove("RUSTUP_TOOLCHAIN")
        .assert()
        .success();
}
