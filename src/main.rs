use clap::Parser;
use serde::Deserialize;
use std::fs::{self, File};
use std::io::{copy, Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;
use zip::ZipArchive;

#[derive(Error, Debug)]
enum Error {
    #[error("Request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Failed to parse JSON: {0}")]
    JsonParseFailed(reqwest::Error),
    #[error("Asset '{0}' not found in release")]
    AssetNotFound(String),
    #[error("Release tag '{0}' not found for repo '{1}'")]
    ReleaseNotFound(String, String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Zip archive error: {0}")]
    ZipError(#[from] zip::result::ZipError),
    #[error("API call failed with status: {0} for url: {1}")]
    ApiError(reqwest::StatusCode, String),
}

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Parser, Debug)]
enum Commands {
    /// Download a release asset
    Download {
        /// GitHub repository in the format "owner/repo"
        #[arg(short, long)]
        repo: String,

        /// The release tag to fetch assets from
        #[arg(short, long)]
        tag: String,

        /// The name of the asset to download
        #[arg(short, long)]
        asset: String,
    },
    /// List releases for a repository
    List {
        /// GitHub repository in the format "owner/repo"
        #[arg(short, long)]
        repo: String,
    },
    /// List assets for a specific release
    ListAssets {
        /// GitHub repository in the format "owner/repo"
        #[arg(short, long)]
        repo: String,

        /// The release tag to fetch assets from
        #[arg(short, long)]
        tag: String,
    },
    /// Install a downloaded asset
    Install {
        /// The name of the asset to install
        #[arg(short, long)]
        asset: String,

        /// The directory to install to
        #[arg(short, long)]
        dir: Option<PathBuf>,
    },
}

#[derive(Deserialize, Debug)]
struct Release {
    assets: Vec<Asset>,
}

#[derive(Deserialize, Debug)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Deserialize, Debug)]
struct ListReleaseItem {
    tag_name: String,
    name: String,
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    let cli = Cli::parse();

    let client = reqwest::Client::builder()
        .user_agent("quick-release-cli")
        .build()?;

    match cli.command {
        Commands::Download { repo, tag, asset } => {
            println!("Fetching release '{}' for repo '{}'...", tag, repo);

            let url = format!("https://api.github.com/repos/{}/releases/tags/{}", repo, tag);

            let response = client.get(&url).send().await?;
            if !response.status().is_success() {
                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    return Err(Error::ReleaseNotFound(tag, repo));
                }
                return Err(Error::ApiError(response.status(), url));
            }

            let release = response.json::<Release>().await.map_err(Error::JsonParseFailed)?;

            let asset_to_download = release
                .assets
                .iter()
                .find(|a| a.name == asset)
                .ok_or_else(|| Error::AssetNotFound(asset.clone()))?;

            println!("Downloading asset: {}", asset_to_download.name);
            println!("From URL: {}", asset_to_download.browser_download_url);

            let response = client.get(&asset_to_download.browser_download_url).send().await?;

            let mut dest = File::create(&asset_to_download.name)?;
            let content = response.bytes().await?;
            copy(&mut content.as_ref(), &mut dest)?;

            println!("Successfully downloaded {}!", asset_to_download.name);
        }
        Commands::List { repo } => {
            println!("Fetching releases for repo '{}'...", repo);

            let url = format!("https://api.github.com/repos/{}/releases", repo);

            let response = client.get(&url).send().await?;
            if !response.status().is_success() {
                return Err(Error::ApiError(response.status(), url));
            }

            let releases = response
                .json::<Vec<ListReleaseItem>>()
                .await
                .map_err(Error::JsonParseFailed)?;

            if releases.is_empty() {
                println!("No releases found for repo '{}'.", repo);
            } else {
                println!("Available releases:");
                for release in releases {
                    println!("- {} ({})", release.tag_name, release.name);
                }
            }
        }
        Commands::ListAssets { repo, tag } => {
            println!("Fetching assets for release '{}' in repo '{}'...", tag, repo);

            let url = format!("https://api.github.com/repos/{}/releases/tags/{}", repo, tag);

            let response = client.get(&url).send().await?;
            if !response.status().is_success() {
                if response.status() == reqwest::StatusCode::NOT_FOUND {
                    return Err(Error::ReleaseNotFound(tag, repo));
                }
                return Err(Error::ApiError(response.status(), url));
            }

            let release = response.json::<Release>().await.map_err(Error::JsonParseFailed)?;

            println!("Available assets:");
            if release.assets.is_empty() {
                println!("No assets found for this release.");
            } else {
                for asset in release.assets {
                    println!("- {}", asset.name);
                }
            }
        }
        Commands::Install { asset, dir } => {
            let install_dir = dir.unwrap_or_else(|| PathBuf::from("."));
            println!(
                "Installing asset '{}' to '{}'...",
                asset,
                install_dir.display()
            );

            let file = File::open(&asset)?;
            let mut archive = ZipArchive::new(file)?;

            for i in 0..archive.len() {
                let mut file = archive.by_index(i)?;
                let outpath = match file.enclosed_name() {
                    Some(path) => install_dir.join(path),
                    None => continue,
                };

                if (&*file.name()).ends_with('/') {
                    println!("File {} extracted to \"{}\"", i, outpath.display());
                    fs::create_dir_all(&outpath)?;
                } else {
                    println!(
                        "File {} extracted to \"{}\" ({} bytes)",
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
                }
            }

            println!("Successfully installed {}!", asset);
        }
    }

    Ok(())
}
