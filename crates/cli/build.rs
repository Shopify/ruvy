use std::{env, fs, path::Path};

use anyhow::Result;

fn main() -> Result<()> {
    let destination = Path::new(&env::var("OUT_DIR")?).join("engine.wasm");
    let is_clippy = env::var("CARGO_CFG_FEATURE").is_ok_and(|v| v == "cargo-clippy");
    if is_clippy {
        fs::write(destination, &[])?;
        println!("cargo:warning=using stubbed engine.wasm for static analysis");
    } else {
        println!("cargo:rerun-if-changed=build.rs");
        let cargo_manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
        let engine_path =
            format!("{cargo_manifest_dir}/../../target/wasm32-wasi/release/core.wasm");
        let engine_path = Path::new(&engine_path);
        println!("cargo:rerun-if-changed={}", engine_path.to_str().unwrap());
        fs::copy(&engine_path, &destination)?;
    }
    Ok(())
}
