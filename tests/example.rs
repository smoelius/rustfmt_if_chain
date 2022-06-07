#![cfg_attr(nightly, feature(bench_black_box, test))]

#[cfg(nightly)]
extern crate test;

use assert_cmd::Command;
use std::{fs::copy, path::Path};
use tempfile::tempdir;

#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[test]
fn example_test() {
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

#[cfg(nightly)]
#[cfg_attr(
    dylint_lib = "non_thread_safe_call_in_test",
    allow(non_thread_safe_call_in_test)
)]
#[bench]
fn example_bench(bencher: &mut test::Bencher) {
    let examples = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    let tempdir = tempdir().unwrap();

    let tempfile = tempdir.path().join("before.rs");

    bencher.iter(|| {
        copy(examples.join("before.rs"), &tempfile).unwrap();

        Command::cargo_bin("rustfmt_if_chain")
            .unwrap()
            .arg(&tempfile)
            .assert()
            .success();
    });
}
