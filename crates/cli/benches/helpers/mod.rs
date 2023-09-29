use std::{
    env::consts,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{anyhow, bail, Result};
use hyper::{body::HttpBody, Body, Client, Response};
use hyper_tls::HttpsConnector;

pub fn cargo_target_tmpdir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

pub fn ruby_wasm() -> Result<PathBuf> {
    let tmpdir = cargo_target_tmpdir();
    const VERSION: &str = "2.1.0";
    const RUBY_VERSION: &str = "3_2";
    const TARGET: &str = "wasm32-unknown-wasi";
    const PROFILE: &str = "minimal";
    let ruby_wasm_base = format!("ruby-wasm-{VERSION}-{RUBY_VERSION}-{TARGET}-{PROFILE}");
    let ruby_wasm_dir = tmpdir.join(&ruby_wasm_base);
    let ruby_wasm = ruby_wasm_dir.join("usr/local/bin/ruby");
    if ruby_wasm.exists() {
        return Ok(ruby_wasm);
    }
    let archive = tmpdir.join(format!("{ruby_wasm_base}.tar.gz"));
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(download_ruby_wasm(
        &archive,
        VERSION,
        RUBY_VERSION,
        TARGET,
        PROFILE,
    ))?;
    extract_ruby_wasm(&archive, &ruby_wasm_dir)?;
    Ok(ruby_wasm)
}

async fn download_ruby_wasm(
    path: &Path,
    version: &str,
    ruby_version: &str,
    target: &str,
    profile: &str,
) -> Result<()> {
    download(format!("https://github.com/ruby/ruby.wasm/releases/download/{version}/ruby-{ruby_version}-{target}-{profile}.tar.gz"), path).await
}

fn extract_ruby_wasm(archive: &Path, extract_to: &Path) -> Result<()> {
    if !extract_to.exists() {
        fs::create_dir(extract_to)?;
    }
    let output = Command::new("tar")
        .args(["-xf", archive.to_str().unwrap(), "--strip-components", "1"])
        .current_dir(extract_to)
        .output()?;
    if !output.status.success() {
        bail!(
            "Unpacking ruby.wasm failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

pub fn wasi_vfs() -> Result<PathBuf> {
    let tmpdir = cargo_target_tmpdir();
    const VERSION: &str = "0.4.0";
    let wasi_vfs_base = format!("wasi-vfs-{VERSION}");
    let directory = tmpdir.join(&wasi_vfs_base);
    let wasi_vfs = directory.join("wasi-vfs");
    if wasi_vfs.exists() {
        return Ok(wasi_vfs);
    }
    let archive = tmpdir.join(format!("{wasi_vfs_base}.tar.gz"));
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    rt.block_on(download_wasi_vfs(&archive, VERSION))?;
    extract_wasi_vfs(&archive, &directory)?;
    Ok(wasi_vfs)
}

async fn download_wasi_vfs(path: &Path, version: &str) -> Result<()> {
    let file_suffix = match (consts::OS, consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-gnu",
        other => bail!("Unsupported platform tuple {:?}", other),
    };
    download(format!("https://github.com/kateinoigakukun/wasi-vfs/releases/download/v{version}/wasi-vfs-cli-{file_suffix}.zip"), path).await
}

fn extract_wasi_vfs(archive: &Path, extract_to: &Path) -> Result<()> {
    let output = Command::new("unzip")
        .arg(archive)
        .arg("-d")
        .arg(extract_to)
        .output()?;
    if !output.status.success() {
        bail!(
            "Unpacking wasi-vfs failed: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}

async fn download(mut uri: String, path: &Path) -> Result<()> {
    let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
    let mut response: Response<Body> = loop {
        let response = client.get(uri.try_into()?).await?;
        let status = response.status();
        if status.is_redirection() {
            uri = response
                .headers()
                .get("Location")
                .ok_or_else(|| {
                    anyhow!(
                        "Received redirect without location header when downloading {} from GitHub",
                        path.to_str().unwrap()
                    )
                })?
                .to_str()?
                .to_string();
        } else if !status.is_success() {
            bail!(
                "Received {status} when downloading {} from GitHub",
                path.to_str().unwrap()
            );
        } else {
            break response;
        }
    };
    let mut archive = File::create(path)?;
    while let Some(chunk) = response.body_mut().data().await {
        archive.write_all(&chunk.map_err(|err| {
            anyhow!(
                "Something went wrong when downloading {}: {err}",
                path.to_str().unwrap()
            )
        })?)?;
    }
    Ok(())
}
