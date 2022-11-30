use assert_cmd::Command;
use std::{
    ffi::OsStr,
    io::{stderr, Write},
    path::{Path, PathBuf},
};
use walkdir::WalkDir;

#[test]
fn dogfood() {
    if Command::new("git")
        .args(["diff", "--exit-code"])
        .assert()
        .try_success()
        .is_err()
    {
        #[allow(clippy::explicit_write)]
        writeln!(stderr(), "Skipping as repository is dirty").unwrap();
        return;
    }

    let paths = paths();

    // smoelius: Sanity.
    assert!(paths
        .iter()
        .any(|path| path.file_name().unwrap() == "main.rs"));

    // smoelius: Format files individually.
    for path in &paths {
        Command::cargo_bin("rustfmt_if_chain")
            .unwrap()
            .arg(path)
            .assert()
            .success();
    }

    // smoelius: Format all files with one command.
    Command::cargo_bin("rustfmt_if_chain")
        .unwrap()
        .args(paths)
        .assert()
        .success();

    Command::new("git")
        .args(["diff", "--exit-code"])
        .assert()
        .success();
}

fn paths() -> Vec<PathBuf> {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");

    WalkDir::new(src)
        .into_iter()
        .filter_map(|entry| {
            let entry = entry.unwrap();
            let path = entry.path();
            if path.extension() == Some(OsStr::new("rs")) {
                Some(path.to_path_buf())
            } else {
                None
            }
        })
        .collect()
}
