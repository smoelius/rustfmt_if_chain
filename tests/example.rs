use assert_cmd::Command;
use std::{fs::copy, path::Path};
use tempfile::tempdir;

#[test]
fn example() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    let tempdir = tempdir().unwrap();

    let tempfile = tempdir.path().join("before.rs");

    copy(examples.join("before.rs"), &tempfile).unwrap();

    Command::cargo_bin("rustfmt_if_chain")
        .unwrap()
        .arg(&tempfile)
        .assert()
        .success();

    Command::new("diff")
        .args(&[tempfile, examples.join("after.rs")])
        .assert()
        .success();
}
