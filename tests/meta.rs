use assert_cmd::Command;
use std::{fs::read_to_string, path::Path, str::from_utf8};

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

    let usage = from_utf8(&stdout).unwrap();

    assert!(contents.contains(usage));
}
