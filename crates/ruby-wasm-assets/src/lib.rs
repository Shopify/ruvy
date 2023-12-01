use std::{
    fs::{self, File},
    io::Write,
    path::Path,
    process::Command,
};

use anyhow::{anyhow, bail, Error, Result};
use http_body_util::{combinators::BoxBody, BodyExt};
use hyper::{
    body::{Bytes, Incoming},
    Response,
};
use hyper_tls::HttpsConnector;
use hyper_util::{client::legacy::Client, rt::TokioExecutor};
use lazy_static::lazy_static;
use tokio::runtime::Runtime;

const RUBY_WASM_VERSION: &str = "2.1.0";
const RUBY_WASM_RUBY_VERSION: &str = "3_2";
const RUBY_WASM_TARGET: &str = "wasm32-unknown-wasi";
const RUBY_WASM_PROFILE: &str = "minimal";

lazy_static! {
    static ref RT: Runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
}

pub fn download(uri: String, path: &Path) -> Result<()> {
    RT.block_on(download_async(uri, path))
}

async fn download_async(mut uri: String, path: &Path) -> Result<()> {
    let file_being_downloaded = path.file_name().unwrap().to_str().unwrap();
    if !path.try_exists()? {
        let client = Client::builder(TokioExecutor::new())
            .build::<_, BoxBody<Bytes, Error>>(HttpsConnector::new());
        let mut response: Response<Incoming> = loop {
            let response = client.get(uri.try_into()?).await?;
            let status = response.status();
            if status.is_redirection() {
                uri = response.headers().get("Location").ok_or_else(|| anyhow!("Received redirect without location header when downloading {file_being_downloaded} from GitHub"))?.to_str()?.to_string();
            } else if !status.is_success() {
                bail!("Received {status} when downloading from {file_being_downloaded}");
            } else {
                break response;
            }
        };
        let mut file = File::create(path)?;
        while let Some(next) = response.frame().await {
            let frame = next?;
            if let Some(chunk) = frame.data_ref() {
                file.write_all(chunk.as_ref())?;
            }
        }
    }
    Ok(())
}

pub fn ruby_wasm_base_name() -> String {
    format!(
        "{}-ruby-{}-{}-{}",
        RUBY_WASM_VERSION, RUBY_WASM_RUBY_VERSION, RUBY_WASM_TARGET, RUBY_WASM_PROFILE
    )
}

pub fn download_ruby_wasm(path: &Path) -> Result<()> {
    download(format!("https://github.com/ruby/ruby.wasm/releases/download/{RUBY_WASM_VERSION}/ruby-{RUBY_WASM_RUBY_VERSION}-{RUBY_WASM_TARGET}-{RUBY_WASM_PROFILE}.tar.gz"), path)
}

pub fn extract_tar(archive: &Path, extract_to: &Path, components_to_strip: i32) -> Result<()> {
    if !extract_to.exists() {
        fs::create_dir(extract_to)?;
    }
    let output = Command::new("tar")
        .args([
            "-xf",
            archive.to_str().unwrap(),
            "--strip-components",
            &components_to_strip.to_string(),
        ])
        .current_dir(extract_to)
        .output()?;
    if !output.status.success() {
        bail!(
            "Unpacking {} failed: {}",
            archive.to_string_lossy(),
            String::from_utf8_lossy(&output.stderr)
        );
    }
    Ok(())
}
