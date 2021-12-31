use assert_cmd::Command;
use std::{fs::copy, path::Path};
use tempfile::NamedTempFile;

#[test]
fn example() {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    let tempfile = NamedTempFile::new().unwrap();

    copy(examples.join("before.rs"), tempfile.path()).unwrap();

    Command::cargo_bin("rustfmt_if_chain")
        .unwrap()
        .arg(tempfile.path())
        .assert()
        .success();

    Command::new("diff")
        .args(&[tempfile.path(), examples.join("after.rs").as_path()])
        .assert()
        .success();
}
