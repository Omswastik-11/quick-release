use assert_cmd::{assert::OutputAssertExt, cargo::CommandCargoExt};
use std::fs;

fn make_zip(path: &std::path::Path) {
    let f = fs::File::create(path).unwrap();
    let mut zip = zip::ZipWriter::new(f);
    let options: zip::write::FileOptions<'_, ()> = zip::write::FileOptions::default();
    zip.add_directory("bin/", options).unwrap();
    zip.start_file("bin/tool.txt", options).unwrap();
    use std::io::Write as _;
    zip.write_all(b"bye").unwrap();
    zip.finish().unwrap();
}

#[test]
fn uninstall_removes_files() {
    let tmp = tempfile::tempdir().unwrap();
    let archive = tmp.path().join("asset.zip");
    make_zip(&archive);

    let outdir = tmp.path().join("out");
    fs::create_dir_all(&outdir).unwrap();

    // install
    std::process::Command::cargo_bin("quick-release").unwrap()
        .args([
            "install",
            "--asset",
            archive.to_str().unwrap(),
            "--dir",
            outdir.to_str().unwrap(),
        ])
        .assert()
        .success();

    let file = outdir.join("bin/tool.txt");
    assert!(file.exists());

    // uninstall
    std::process::Command::cargo_bin("quick-release").unwrap()
        .args([
            "uninstall",
            "--asset",
            archive.to_str().unwrap(),
            "--dir",
            outdir.to_str().unwrap(),
        ])
        .assert()
        .success();

    assert!(!file.exists());
}
