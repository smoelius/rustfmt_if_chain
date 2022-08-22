#[cfg(feature = "ci")]
mod ci {
    use assert_cmd::Command;
    use regex::Regex;
    use std::{fs::read_to_string, path::Path, str::from_utf8};
    use tempfile::tempdir;

    #[test]
    fn clippy() {
        Command::new("cargo")
            .args(&[
                "clippy",
                "--tests",
                "--",
                "-D",
                "warnings",
                "-W",
                "clippy::pedantic",
            ])
            .assert()
            .success();
    }

    #[test]
    fn dylint() {
        Command::new("cargo")
            .args(&["dylint", "--all", "--", "--tests"])
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
            assert!(re.is_match(line), "{:?} does not match", line);
        }
    }

    #[test]
    fn markdown_link_check() {
        let tempdir = tempdir().unwrap();

        Command::new("npm")
            .args(&["install", "markdown-link-check"])
            .current_dir(tempdir.path())
            .assert()
            .success();

        let readme_md = Path::new(env!("CARGO_MANIFEST_DIR")).join("README.md");

        Command::new("npx")
            .args(&["markdown-link-check", &readme_md.to_string_lossy()])
            .current_dir(tempdir.path())
            .assert()
            .success();
    }

    #[test]
    fn prettier() {
        let tempdir = tempdir().unwrap();

        Command::new("npm")
            .args(&["install", "prettier"])
            .current_dir(tempdir.path())
            .assert()
            .success();

        Command::new("npx")
            .args(&[
                "prettier",
                "--check",
                &format!("{}/**/*.md", env!("CARGO_MANIFEST_DIR")),
                &format!("{}/**/*.yml", env!("CARGO_MANIFEST_DIR")),
                &format!("!{}/target/**", env!("CARGO_MANIFEST_DIR")),
            ])
            .current_dir(tempdir.path())
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

        let usage = from_utf8(&stdout).unwrap();

        assert!(contents.contains(usage));
    }

    #[test]
    fn sort() {
        Command::new("cargo")
            .args(&["sort", "--check"])
            .assert()
            .success();
    }

    #[test]
    fn udeps() {
        Command::new("cargo")
            .args(&["+nightly", "udeps", "--all-targets"])
            .assert()
            .success();
    }
}
