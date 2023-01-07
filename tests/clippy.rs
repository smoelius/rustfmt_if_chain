use assert_cmd::Command;
use std::{ffi::OsStr, fs::OpenOptions, io::Write};
use tempfile::tempdir_in;
use walkdir::WalkDir;

const CLIPPY_URL: &str = "https://github.com/rust-lang/rust-clippy";

#[test]
fn clippy() {
    let tempdir = tempdir_in(".").unwrap();

    let crashes = tempdir.path().join("tests").join("ui").join("crashes");

    Command::new("git")
        .args(["clone", CLIPPY_URL, &tempdir.path().to_string_lossy()])
        .assert()
        .success();

    for entry in WalkDir::new(&tempdir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        let assert = Command::cargo_bin("rustfmt_if_chain")
            .unwrap()
            .current_dir(&tempdir)
            .env_remove("RUSTUP_TOOLCHAIN")
            .arg(path)
            .assert();
        // smoelius: `needless_return.rs` uses the `do` keyword (see:
        // https://github.com/rust-lang/rust-clippy/pull/10109), which does not seem to be supported
        // by `syn`.
        assert!(
            assert.try_success().is_ok()
                || path.starts_with(&crashes)
                || path.file_name().unwrap() == "lib.deprecated.rs"
                || path.file_name().unwrap() == "needless_return.rs",
            "path = {:?}",
            path
        );
    }

    let mut file = OpenOptions::new()
        .append(true)
        .write(true)
        .open(tempdir.path().join("Cargo.toml"))
        .unwrap();
    writeln!(file, "[workspace]").unwrap();

    Command::new("cargo")
        .args(["build", "--all-features", "--all-targets"])
        .current_dir(&tempdir)
        .env_remove("RUSTUP_TOOLCHAIN")
        .assert()
        .success();
}
