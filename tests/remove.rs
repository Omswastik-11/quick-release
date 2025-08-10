use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use std::fs;

#[test]
fn remove_deletes_asset_file() {
    let tmp = tempfile::tempdir().unwrap();
    let asset = tmp.path().join("dummy.zip");
    fs::write(&asset, b"dummy").unwrap();

    std::process::Command::cargo_bin("quick-release").unwrap()
        .args(["remove", "--asset", asset.to_str().unwrap()])
        .assert()
        .success();

    assert!(!asset.exists());
}
