use anyhow::{bail, Result};
use std::{env, fs, path::PathBuf};

const WASI_SDK_VERSION_MAJOR: usize = 20;
const WASI_SDK_VERSION_MINOR: usize = 0;

fn main() -> Result<()> {
    let wasi_sdk_path = wasi_sdk_path()?;
    let wasi_sdk_path = wasi_sdk_path.to_string_lossy();
    let sysroot = format!("--sysroot={}/share/wasi-sysroot", &wasi_sdk_path);
    let sysroot_lib = format!("{}/share/wasi-sysroot/lib/wasm32-wasi", &wasi_sdk_path);

    let ruby_wasm_dir = ruby_wasm_path()?;
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

fn wasi_sdk_path() -> Result<PathBuf> {
    const WASI_SDK_PATH_ENV_VAR: &str = "RUVY_WASM_SYS_WASI_SDK_PATH";
    println!("cargo:rerun-if-env-changed={WASI_SDK_PATH_ENV_VAR}");
    if let Ok(path) = env::var(WASI_SDK_PATH_ENV_VAR) {
        return Ok(path.into());
    }
    download_wasi_sdk()
}

fn download_wasi_sdk() -> Result<PathBuf> {
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
    ruby_wasm_assets::download(uri, &archive_path)?;
    ruby_wasm_assets::extract_tar(&archive_path, &wasi_sdk_dir, 1)?;

    Ok(wasi_sdk_dir)
}

fn ruby_wasm_path() -> Result<PathBuf> {
    const RUBY_WASM_PATH_ENV_VAR: &str = "RUVY_WASM_SYS_RUBY_PATH";
    println!("cargo:rerun-if-env-changed={RUBY_WASM_PATH_ENV_VAR}");
    if let Ok(path) = env::var(RUBY_WASM_PATH_ENV_VAR) {
        return Ok(path.into());
    }
    download_ruby_wasm()
}

fn download_ruby_wasm() -> Result<PathBuf> {
    let mut ruby_wasm_dir: PathBuf = env::var("OUT_DIR")?.into();
    ruby_wasm_dir.push("ruby-wasm");
    fs::create_dir_all(&ruby_wasm_dir)?;
    let mut archive_path = ruby_wasm_dir.clone();
    archive_path.push(ruby_wasm_assets::ruby_wasm_base_name());
    archive_path.set_extension("tar.gz");

    ruby_wasm_assets::download_ruby_wasm(&archive_path)?;
    // Need to strip archive name, `usr`, and `local`.
    ruby_wasm_assets::extract_tar(&archive_path, &ruby_wasm_dir, 3)?;

    Ok(ruby_wasm_dir)
}
