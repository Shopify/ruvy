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
        // TODO we're using this update the encoding of the 0 index for the
        // table used by `call_indirect` from a multi-byte LEB encoding to a
        // single byte encoding so Wizer doesn't fail saying reference types
        // aren't supported. I tried disabling reference-types in
        // https://github.com/Shopify/ruvy/pull/92 but that still resulted in
        // the engine Wasm module using a multi-byte LEB encoding for the table
        // index so we need to investigate why that setting the compiler flag
        // doesn't work as expected.
        wasm_opt::OptimizationOptions::new_opt_level_3() // Aggressively optimize for speed.
            .shrink_level(wasm_opt::ShrinkLevel::Level0) // Don't optimize for size at the cost of performance.
            .run(&engine_path, &destination)?;
    }
    Ok(())
}
