use assert_cmd::Command;
use std::{ffi::OsStr, fs::OpenOptions, io::Write};
use tempfile::tempdir_in;
use walkdir::WalkDir;

const CLIPPY_URL: &str = "https://github.com/rust-lang/rust-clippy";

// smoelius: `needless_return.rs` uses the `do` keyword (see:
// https://github.com/rust-lang/rust-clippy/pull/10109), which does not seem to be supported by
// `syn`.
// smoelius: This problem persists with `syn` 2.0.

// smoelius: Something weird is going on with `clippy_lints/src/derive.rs`. Remove the exception
// once the following is resolved: https://github.com/rust-lang/rustfmt/issues/5700

// smoelius: From: https://github.com/dtolnay/syn/releases/tag/2.0.0
//
//   Support for `box expr` syntax has been deleted, as it has been deleted recently from rustc.
//
// This is the reason for the boxed_local.rs, no_effect.rs, and unnecessary_operation.rs exceptions.
const EXCEPTIONS: &[&str] = &[
    "clippy_lints/src/derive.rs",
    "clippy_lints/src/lib.deprecated.rs",
    "tests/ui/boxed_local.rs",
    "tests/ui/needless_return.rs",
    "tests/ui/no_effect.rs",
    "tests/ui/unnecessary_operation.rs",
];

#[test]
fn clippy() {
    let tempdir = tempdir_in(".").unwrap();

    let crashes = tempdir.path().join("tests/ui/crashes");

    Command::new("git")
        .args(["clone", CLIPPY_URL, &tempdir.path().to_string_lossy()])
        .assert()
        .success();

    for entry in WalkDir::new(&tempdir) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() != Some(OsStr::new("rs"))
            || path.starts_with(&crashes)
            || EXCEPTIONS.iter().any(|child| path.ends_with(child))
        {
            continue;
        }
        Command::cargo_bin("rustfmt_if_chain")
            .unwrap()
            .current_dir(&tempdir)
            .env_remove("RUSTUP_TOOLCHAIN")
            .arg(path)
            .assert()
            .success();
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
