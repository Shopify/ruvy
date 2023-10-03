use std::{
    env::consts,
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{bail, Result};

pub fn cargo_target_tmpdir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
}

pub fn ruby_wasm() -> Result<PathBuf> {
    let tmpdir = cargo_target_tmpdir();
    let ruby_wasm_base = ruby_wasm_assets::ruby_wasm_base_name();
    let ruby_wasm_dir = tmpdir.join(&ruby_wasm_base);
    let ruby_wasm = ruby_wasm_dir.join("usr/local/bin/ruby");
    if ruby_wasm.exists() {
        return Ok(ruby_wasm);
    }
    let archive = tmpdir.join(format!("{ruby_wasm_base}.tar.gz"));
    ruby_wasm_assets::download_ruby_wasm(&archive)?;
    ruby_wasm_assets::extract_tar(&archive, &ruby_wasm_dir, 1)?;
    Ok(ruby_wasm)
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
    download_wasi_vfs(&archive, VERSION)?;
    extract_wasi_vfs(&archive, &directory)?;
    Ok(wasi_vfs)
}

fn download_wasi_vfs(path: &Path, version: &str) -> Result<()> {
    let file_suffix = match (consts::OS, consts::ARCH) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("macos", "x86_64") => "x86_64-apple-darwin",
        ("macos", "aarch64") => "aarch64-apple-darwin",
        ("windows", "x86_64") => "x86_64-pc-windows-gnu",
        other => bail!("Unsupported platform tuple {:?}", other),
    };
    ruby_wasm_assets::download(format!("https://github.com/kateinoigakukun/wasi-vfs/releases/download/v{version}/wasi-vfs-cli-{file_suffix}.zip"), path)
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
