use assert_cmd::Command;
use std::{
    ffi::OsStr,
    io::{stderr, Write},
    path::Path,
};
use walkdir::WalkDir;

#[test]
fn dogfood() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    if Command::new("git")
        .args(&["diff", "--exit-code"])
        .assert()
        .try_success()
        .is_err()
    {
        #[allow(clippy::explicit_write)]
        writeln!(stderr(), "Skipping as repository is dirty").unwrap();
        return;
    }

    for entry in WalkDir::new(src) {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.extension() != Some(OsStr::new("rs")) {
            continue;
        }
        Command::cargo_bin("rustfmt_if_chain")
            .unwrap()
            .arg(path)
            .assert()
            .success();
    }

    Command::new("git")
        .args(&["diff", "--exit-code"])
        .assert()
        .success();
}
