use std::{env, fs, path::Path};

use anyhow::Result;

fn main() -> Result<()> {
    let destination = Path::new(&env::var("OUT_DIR")?).join("engine.wasm");
    let is_clippy = env::var("CARGO_CFG_FEATURE").is_ok_and(|v| v == "cargo-clippy");
    if is_clippy {
        fs::write(destination, [])?;
        println!("cargo:warning=using stubbed engine.wasm for static analysis");
    } else {
        println!("cargo:rerun-if-changed=build.rs");
        let engine_path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../target/wasm32-wasip1/release/core.wasm");
        println!("cargo:rerun-if-changed={}", engine_path.to_str().unwrap());
        fs::copy(&engine_path, &destination)?;
    }
    Ok(())
}
