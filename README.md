# Quick-Release

A Rust-powered CLI tool for managing GitHub release assets directly from your terminal. `quick-release` simplifies your workflow by allowing you to list all available releases for a repository, inspect the assets for a specific tag, and then download and install them with simple commands.

This tool is a practical demonstration of building a CLI application in Rust, using popular libraries like `clap`, `reqwest`, `tokio`, and `serde`.

## Prerequisites

Before you begin, ensure you have the following installed:

-   **Rust and Cargo**: This project is built with Rust, so you'll need the Rust toolchain, which includes `rustc` (the compiler) and `cargo` (the package manager and build tool). You can install them by following the official instructions at [rust-lang.org](https://www.rust-lang.org/tools/install).

## Installation

You can install `quick-release` directly from the source code using `cargo`.

1.  Clone this repository:
    ```bash
    git clone <repository_url>
    cd quick-release
    ```
2.  Install the binary using `cargo`:
    ```bash
    cargo install --path .
    ```
This will build the project and place the `quick-release` executable in your Cargo binary directory (`~/.cargo/bin`), making it available from anywhere in your terminal.

## Usage

`quick-release` provides a set of simple commands to interact with GitHub releases.

you can use `quick-release --help` for more info

### List Releases

To see all available releases for a repository:
```bash
quick-release list --repo "WasmEdge/WasmEdge"
```
**Output:**
```
Fetching releases for repo 'WasmEdge/WasmEdge'...
Available releases:
- 0.15.0 (WasmEdge 0.15.0)
- 0.15.0-rc.1 (WasmEdge 0.15.0-rc.1)
...
```

### List Assets for a Release

To see all the downloadable assets for a specific release tag:
```bash
quick-release list-assets --repo "WasmEdge/WasmEdge" --tag "0.15.0"
```
**Output:**
```
Fetching assets for release '0.15.0' in repo 'WasmEdge/WasmEdge'...
Available assets:
- sbom.tar.gz
- SHA256SUM
- WasmEdge-0.15.0-alpine3.16_aarch64_static.tar.gz
...
```

### Download an Asset

To download a specific asset from a release:
```bash
quick-release download --repo "WasmEdge/WasmEdge" --tag "0.15.0" --asset "WasmEdge-0.15.0-windows.zip"
```
**Output:**
```
Fetching release '0.15.0' for repo 'WasmEdge/WasmEdge'...
Downloading asset: WasmEdge-0.15.0-windows.zip
From URL: https://github.com/WasmEdge/WasmEdge/releases/download/0.15.0/WasmEdge-0.15.0-windows.zip
Successfully downloaded WasmEdge-0.15.0-windows.zip!
```

### Install an Asset

To extract a downloaded asset (`.zip`, `.tar.gz`, `.tar.xz`) into a directory:
```bash
quick-release install --asset "WasmEdge-0.15.0-windows.zip" --dir "wasmedge-installed"
```
**Output:**
```
Installing asset 'WasmEdge-0.15.0-windows.zip' to 'wasmedge-installed'...
File 0 extracted to "wasmedge-installed/include/"
File 1 extracted to "wasmedge-installed/lib/"
...
Successfully installed WasmEdge-0.15.0-windows.zip!
```

### Remove a Downloaded Asset

Delete an already-downloaded archive from disk:
```bash
quick-release remove --asset "WasmEdge-0.15.0-windows.zip"
```

### Uninstall Files Extracted from an Asset

Remove files previously extracted from an archive into a directory:
```bash
quick-release uninstall --asset "WasmEdge-0.15.0-windows.zip" --dir "wasmedge-installed"
```
What happens:
- Reads the archive to learn its file/dir entries.
- Removes matching files under the install directory first.
- Then prunes now-empty directories (deepest-first); leaves non-empty ones.

## Platform support

Extraction supported on Linux, macOS, and Windows for:
- .zip
- .tar.gz / .tgz
- .tar.xz
- .tar

## Development

Run tests (unit + integration):
```bash
cargo test --all
```

CI (GitHub Actions):
- Builds and tests on ubuntu-latest, macos-latest, windows-latest
- Runs rustfmt check and clippy lints
Workflow: `.github/workflows/ci.yml`