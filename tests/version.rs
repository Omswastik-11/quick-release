use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use predicates::prelude::*;

#[test]
fn prints_version() {
    let mut cmd = std::process::Command::cargo_bin("quick-release").unwrap();
    cmd.arg("--version");
    cmd.assert()
        .success()
        .stdout(predicate::str::is_match(r"^quick-release \d+\.\d+\.\d+").unwrap());
}
