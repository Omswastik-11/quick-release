use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use predicates::prelude::*;
use std::fs;

fn make_zip(path: &std::path::Path) {
    let f = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
    zip.add_directory("bin/", options).unwrap();
    zip.start_file("bin/tool.txt", options).unwrap();
    use std::io::Write as _;
    zip.write_all(b"hello-integration").unwrap();
    zip.finish().unwrap();
}

#[test]
fn install_extracts_zip() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("asset.zip");
    make_zip(&archive);

    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    let mut cmd = std::process::Command::cargo_bin("quick-release").unwrap();
    cmd.args([
        "install",
        "--asset",
        archive.to_str().unwrap(),
        "--dir",
        outdir.to_str().unwrap(),
    ]);
    cmd.assert()
        .success()
        .stdout(predicate::str::contains("Successfully installed"));

    let extracted = outdir.join("bin/tool.txt");
    let content = fs::read_to_string(&extracted).unwrap();
    assert_eq!(content, "hello-integration");
}
