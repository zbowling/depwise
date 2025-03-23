use assert_cmd::prelude::*;
use predicates::prelude::*;
use std::process::Command;

#[test]
fn check_version() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("depwise")?;

    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("depwise"));

    Ok(())
}

#[test]
fn check_help() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("depwise")?;

    cmd.arg("--help");
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("A fast, comprehensive dependency analyzer for Python to detect unused, missing, and optional dependencies."));

    Ok(())
}
