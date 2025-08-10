use crate::Error;
use std::fs;
use std::fs::File;
use std::io::copy;
use std::path::Path;
use zip::ZipArchive;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ArchiveType {
    Zip,
    TarGz,
    TarXz,
    Tar,
}

pub(crate) fn detect_archive_type(path: &Path) -> Option<ArchiveType> {
    let name = path.file_name()?.to_string_lossy().to_lowercase();
    if name.ends_with(".tar.gz") || name.ends_with(".tgz") {
        Some(ArchiveType::TarGz)
    } else if name.ends_with(".tar.xz") {
        Some(ArchiveType::TarXz)
    } else if name.ends_with(".zip") {
        Some(ArchiveType::Zip)
    } else if name.ends_with(".tar") {
        Some(ArchiveType::Tar)
    } else {
        None
    }
}

pub(crate) fn extract_archive(archive_path: &Path, dest: &Path) -> Result<(), Error> {
    match detect_archive_type(archive_path) {
        Some(ArchiveType::Zip) => extract_zip(archive_path, dest),
        Some(ArchiveType::TarGz) => extract_tar_gz(archive_path, dest),
        Some(ArchiveType::TarXz) => extract_tar_xz(archive_path, dest),
        Some(ArchiveType::Tar) => extract_tar(archive_path, dest),
        None => Err(Error::UnsupportedArchive(
            archive_path.display().to_string(),
        )),
    }
}

pub(crate) fn list_archive_entries(
    archive_path: &Path,
) -> Result<Vec<(std::path::PathBuf, bool)>, Error> {
    match detect_archive_type(archive_path) {
        Some(ArchiveType::Zip) => list_zip_entries(archive_path),
        Some(ArchiveType::TarGz) => list_targz_entries(archive_path),
        Some(ArchiveType::TarXz) => list_tarxz_entries(archive_path),
        Some(ArchiveType::Tar) => list_tar_entries(archive_path),
        None => Err(Error::UnsupportedArchive(
            archive_path.display().to_string(),
        )),
    }
}

fn extract_zip(archive_path: &Path, dest: &Path) -> Result<(), Error> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;

    for i in 0..archive.len() {
        let mut file = archive.by_index(i)?;
        let outpath = match file.enclosed_name() {
            Some(path) => dest.join(path),
            None => continue,
        };

        if (&*file.name()).ends_with('/') {
            println!("Dir {} -> {}", i, outpath.display());
            fs::create_dir_all(&outpath)?;
        } else {
            println!(
                "File {} -> {} ({} bytes)",
                i,
                outpath.display(),
                file.size()
            );
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p)?;
                }
            }
            let mut outfile = fs::File::create(&outpath)?;
            copy(&mut file, &mut outfile)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    fs::set_permissions(&outpath, fs::Permissions::from_mode(mode))?;
                }
            }
        }
    }

    Ok(())
}

fn list_zip_entries(archive_path: &Path) -> Result<Vec<(std::path::PathBuf, bool)>, Error> {
    let file = File::open(archive_path)?;
    let mut archive = ZipArchive::new(file)?;
    let mut out = Vec::with_capacity(archive.len());
    for i in 0..archive.len() {
        let file = archive.by_index(i)?;
        if let Some(name) = file.enclosed_name() {
            let is_dir = file.name().ends_with('/');
            out.push((name.to_path_buf(), is_dir));
        }
    }
    Ok(out)
}

fn extract_tar_gz(archive_path: &Path, dest: &Path) -> Result<(), Error> {
    let file = File::open(archive_path)?;
    let dec = flate2::read::GzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest)?;
    Ok(())
}

fn list_targz_entries(archive_path: &Path) -> Result<Vec<(std::path::PathBuf, bool)>, Error> {
    let file = File::open(archive_path)?;
    let dec = flate2::read::GzDecoder::new(file);
    list_tar_like_entries(dec)
}

fn extract_tar_xz(archive_path: &Path, dest: &Path) -> Result<(), Error> {
    let file = File::open(archive_path)?;
    let dec = xz2::read::XzDecoder::new(file);
    let mut archive = tar::Archive::new(dec);
    archive.unpack(dest)?;
    Ok(())
}

fn list_tarxz_entries(archive_path: &Path) -> Result<Vec<(std::path::PathBuf, bool)>, Error> {
    let file = File::open(archive_path)?;
    let dec = xz2::read::XzDecoder::new(file);
    list_tar_like_entries(dec)
}

fn extract_tar(archive_path: &Path, dest: &Path) -> Result<(), Error> {
    let file = File::open(archive_path)?;
    let mut archive = tar::Archive::new(file);
    archive.unpack(dest)?;
    Ok(())
}

fn list_tar_entries(archive_path: &Path) -> Result<Vec<(std::path::PathBuf, bool)>, Error> {
    let file = File::open(archive_path)?;
    list_tar_like_entries(file)
}

fn list_tar_like_entries<R: std::io::Read>(
    reader: R,
) -> Result<Vec<(std::path::PathBuf, bool)>, Error> {
    let mut archive = tar::Archive::new(reader);
    let mut out = Vec::new();
    for entry in archive.entries()? {
        let entry = entry?;
        let path = entry.path()?;
        let is_dir = entry.header().entry_type().is_dir();
        if path.is_absolute() {
            continue;
        }
        out.push((path.to_path_buf(), is_dir));
    }
    Ok(out)
}
