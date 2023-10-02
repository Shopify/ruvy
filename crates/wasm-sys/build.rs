use anyhow::{anyhow, bail, Result};
use hyper::{body::HttpBody, Body, Client, Response};
use hyper_tls::HttpsConnector;
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::{Path, PathBuf},
    process::Command,
};

const WASI_SDK_VERSION_MAJOR: usize = 20;
const WASI_SDK_VERSION_MINOR: usize = 0;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let wasi_sdk_path = wasi_sdk_path().await?;
    let wasi_sdk_path = wasi_sdk_path.to_string_lossy();
    let sysroot = format!("--sysroot={}/share/wasi-sysroot", &wasi_sdk_path);
    let sysroot_lib = format!("{}/share/wasi-sysroot/lib/wasm32-wasi", &wasi_sdk_path);

    let ruby_wasm_dir = ruby_wasm_path().await?;
    let lib_dir = ruby_wasm_dir.join("lib");
    let include_dir = ruby_wasm_dir.join("include");
    let include_dir = fs::read_dir(include_dir)?
        .find(|e| {
            e.as_ref()
                .unwrap()
                .file_name()
                .to_str()
                .unwrap()
                .starts_with("ruby-")
        })
        .unwrap()?
        .path();
    let include_config_dir = include_dir.join("wasm32-wasi");

    env::set_var("CC", format!("{}/bin/clang", &wasi_sdk_path));
    env::set_var("LD", format!("{}/bin/clang", &wasi_sdk_path));
    env::set_var("AR", format!("{}/bin/ar", &wasi_sdk_path));
    env::set_var("CFLAGS", &sysroot);

    // Ruby lib directory
    println!("cargo:rustc-link-search={}", lib_dir.display());
    // WASI Sysroot directory
    println!("cargo:rustc-link-search={}", sysroot_lib);

    cc::Build::new()
        .file("foo.c")
        .flag_if_supported("-fdeclspec")
        .cargo_metadata(true)
        .include(&include_dir)
        .include(&include_config_dir)
        .target("wasm32-wasi")
        .compile("ruvy");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(&[
            "-fvisibility=default",
            "--target=wasm32-wasi",
            &sysroot,
            &format!("-I{}", include_dir.display()),
            &format!("-I{}", include_config_dir.display()),
        ])
        .generate()
        .unwrap();

    println!("cargo:rustc-link-lib=static=ruby-static");
    println!("cargo:rustc-link-lib=static=m");
    println!("cargo:rustc-link-lib=static=wasi-emulated-signal");
    println!("cargo:rustc-link-lib=static=wasi-emulated-mman");
    println!("cargo:rustc-link-lib=static=wasi-emulated-process-clocks");
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=crypt");
    println!("cargo:rustc-link-lib=static=pthread");
    println!("cargo:rustc-link-lib=static=rt");
    println!("cargo:rustc-link-lib=static=dl");
    println!("cargo:rustc-link-lib=static=resolv");
    println!("cargo:rustc-link-lib=static=util");

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    bindings.write_to_file(out_dir.join("bindings.rs"))?;
    Ok(())
}

async fn wasi_sdk_path() -> Result<PathBuf> {
    const WASI_SDK_PATH_ENV_VAR: &str = "RUVY_WASM_SYS_WASI_SDK_PATH";
    println!("cargo:rerun-if-env-changed={WASI_SDK_PATH_ENV_VAR}");
    if let Ok(path) = env::var(WASI_SDK_PATH_ENV_VAR) {
        return Ok(path.into());
    }
    download_wasi_sdk().await
}

async fn download_wasi_sdk() -> Result<PathBuf> {
    let mut wasi_sdk_dir: PathBuf = env::var("OUT_DIR")?.into();
    wasi_sdk_dir.push("wasi-sdk");
    fs::create_dir_all(&wasi_sdk_dir)?;

    const MAJOR_VERSION_ENV_VAR: &str = "RUVY_WASM_SYS_WASI_SDK_MAJOR_VERSION";
    const MINOR_VERSION_ENV_VAR: &str = "RUVY_WASM_SYS_WASI_SDK_MINOR_VERSION";
    println!("cargo:rerun-if-env-changed={MAJOR_VERSION_ENV_VAR}");
    println!("cargo:rerun-if-env-changed={MINOR_VERSION_ENV_VAR}");
    let major_version =
        env::var(MAJOR_VERSION_ENV_VAR).unwrap_or(WASI_SDK_VERSION_MAJOR.to_string());
    let minor_version =
        env::var(MINOR_VERSION_ENV_VAR).unwrap_or(WASI_SDK_VERSION_MINOR.to_string());

    let mut archive_path = wasi_sdk_dir.clone();
    archive_path.push(format!("wasi-sdk-{major_version}-{minor_version}.tar.gz"));

    let file_suffix = match (env::consts::OS, env::consts::ARCH) {
        ("linux", "x86") | ("linux", "x86_64") => "linux",
        ("macos", "x86") | ("macos", "x86_64") | ("macos", "aarch64") => "macos",
        ("windows", "x86") => "mingw-x86",
        ("windows", "x86_64") => "mingw",
        other => bail!("Unsupported platform tuple {:?}", other),
    };
    let uri = format!("https://github.com/WebAssembly/wasi-sdk/releases/download/wasi-sdk-{major_version}/wasi-sdk-{major_version}.{minor_version}-{file_suffix}.tar.gz");
    download_from_github(uri, &archive_path).await?;

    let mut test_binary = wasi_sdk_dir.clone();
    test_binary.extend(["bin", "wasm-ld"]);
    extract_archive(&archive_path, &test_binary, &wasi_sdk_dir, 1)?;

    Ok(wasi_sdk_dir)
}

async fn ruby_wasm_path() -> Result<PathBuf> {
    const RUBY_WASM_PATH_ENV_VAR: &str = "RUVY_WASM_SYS_RUBY_PATH";
    println!("cargo:rerun-if-env-changed={RUBY_WASM_PATH_ENV_VAR}");
    if let Ok(path) = env::var(RUBY_WASM_PATH_ENV_VAR) {
        return Ok(path.into());
    }
    download_ruby_wasm().await
}

async fn download_ruby_wasm() -> Result<PathBuf> {
    let mut ruby_wasm_dir: PathBuf = env::var("OUT_DIR")?.into();
    ruby_wasm_dir.push("ruby-wasm");
    fs::create_dir_all(&ruby_wasm_dir)?;
    let mut archive_path = ruby_wasm_dir.clone();

    const VERSION: &str = "2.1.0";
    const RUBY_VERSION: &str = "3_2";
    const TARGET: &str = "wasm32-unknown-wasi";
    const PROFILE: &str = "minimal";
    archive_path.push(format!(
        "{VERSION}-ruby-{RUBY_VERSION}-{TARGET}-{PROFILE}.tar.gz"
    ));

    download_from_github(format!("https://github.com/ruby/ruby.wasm/releases/download/{VERSION}/ruby-{RUBY_VERSION}-{TARGET}-{PROFILE}.tar.gz"), &archive_path).await?;
    let mut test_archive = ruby_wasm_dir.clone();
    test_archive.extend(["lib", "libruby-static.a"]);
    // Need to strip archive name, `usr`, and `local`.
    extract_archive(&archive_path, &test_archive, &ruby_wasm_dir, 3)?;

    Ok(ruby_wasm_dir)
}

async fn download_from_github(mut uri: String, path: &Path) -> Result<()> {
    let file_being_downloaded = path.file_name().unwrap().to_str().unwrap();
    if !path.try_exists()? {
        let client = Client::builder().build::<_, hyper::Body>(HttpsConnector::new());
        let mut response: Response<Body> = loop {
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
        while let Some(chunk) = response.body_mut().data().await {
            file.write_all(&chunk.map_err(|err| {
                anyhow!("Something went wrong when downloading {file_being_downloaded}: {err}",)
            })?)?;
        }
    }
    Ok(())
}

fn extract_archive(
    archive_path: &Path,
    test_path: &Path,
    out_path: &Path,
    components_to_strip: i32,
) -> Result<()> {
    if !test_path.try_exists()? {
        let output = Command::new("tar")
            .args([
                "-xf",
                archive_path.to_string_lossy().as_ref(),
                "--strip-components",
                &components_to_strip.to_string(),
            ])
            .current_dir(out_path)
            .output()?;
        if !output.status.success() {
            bail!(
                "Unpacking {} failed: {}",
                archive_path.to_str().unwrap(),
                String::from_utf8_lossy(&output.stderr)
            );
        }
    }
    Ok(())
}
