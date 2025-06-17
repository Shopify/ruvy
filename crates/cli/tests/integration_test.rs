use std::{env, path::Path, process::Command, str};

use anyhow::{bail, Result};
use std::fs;
use std::io::Write;
use tempfile::NamedTempFile;
use tempfile::TempDir;
use wasmtime::{Engine, Linker, Module, Store};
use wasmtime_wasi::{
    pipe::{MemoryInputPipe, MemoryOutputPipe},
    preview1::WasiP1Ctx,
    WasiCtxBuilder,
};

#[test]
pub fn test_hello_world() -> Result<()> {
    let wasm_path = wasm_path("hello_world");
    run_ruvy(&wasm_path, "../../ruby_examples/hello_world.rb", None)?;
    let output = run_wasm(&wasm_path, "")?;
    assert_eq!("Hello world\n", output);
    Ok(())
}

#[test]
pub fn test_preludes() -> Result<()> {
    let wasm_path = wasm_path("preludes");
    run_ruvy(
        &wasm_path,
        "../../ruby_examples/use_preludes_and_stdin.rb",
        Some("../../prelude"),
    )?;
    let output = run_wasm(&wasm_path, "this is my input")?;
    assert_eq!(
        "{:discount_input=>\"this is my input\", :value=>100.0}\n",
        output
    );
    Ok(())
}

struct Context {
    wasi: WasiP1Ctx,
    out_stream: MemoryOutputPipe,
}

impl Context {
    fn new(input: &[u8]) -> Context {
        let out_stream = MemoryOutputPipe::new(usize::MAX);
        Context {
            wasi: WasiCtxBuilder::new()
                .stdin(MemoryInputPipe::new(input.to_vec()))
                .stdout(out_stream.clone())
                .build_p1(),
            out_stream,
        }
    }
}

fn wasm_path(test_name: &str) -> String {
    format!("{}/{test_name}.wasm", env!("CARGO_TARGET_TMPDIR"))
}

fn run_ruvy(wasm_path: &str, input_path: &str, preload: Option<&str>) -> Result<()> {
    let mut args = vec![format!("-o{wasm_path}")];
    if let Some(preload) = preload {
        args.push(format!("--preload={preload}"));
    }
    args.push(input_path.to_string());

    let status = Command::new(env!("CARGO_BIN_EXE_ruvy"))
        .args(args)
        .status()?;
    if !status.success() {
        bail!("Failed to execute ruvy");
    }
    Ok(())
}

fn run_wasm(wasm_path: impl AsRef<Path>, input: &str) -> Result<String> {
    let engine = Engine::default();
    let mut linker = Linker::new(&engine);
    wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |cx: &mut Context| &mut cx.wasi)?;
    let mut store = Store::new(&engine, Context::new(input.as_bytes()));

    let module = Module::from_file(&engine, wasm_path)?;
    linker
        .instantiate(&mut store, &module)?
        .get_typed_func::<(), ()>(&mut store, "_start")?
        .call(&mut store, ())?;

    let context = store.into_data();
    drop(context.wasi);
    let output = context.out_stream.contents();
    let output = String::from_utf8(output.to_vec())?;

    Ok(output)
}

#[test]
pub fn test_preload_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let invalid_file = temp_dir.path().join("invalid.rb");
    let mut file = fs::File::create(&invalid_file)?;
    writeln!(file, "raise 'intentional preload error'")?;

    let wasm_path = wasm_path("preload_error");

    let output = Command::new(env!("CARGO_BIN_EXE_ruvy"))
        .args([
            format!("-o{}", wasm_path),
            format!("--preload={}", temp_dir.path().to_string_lossy()),
            "../../ruby_examples/hello_world.rb".to_string(),
        ])
        .output()?;

    let stderr_str = String::from_utf8_lossy(&output.stderr);

    // Check that we get a wizer initialization error or process failure
    assert!(
        !output.status.success() || stderr_str.contains("wizer.initialize"),
        "Expected ruvy to fail with preload error. Status: {:?}, Stderr: {}",
        output.status,
        stderr_str
    );
    Ok(())
}

#[test]
pub fn test_ruby_runtime_error_in_wasm_execution() -> Result<()> {
    let mut temp_file = NamedTempFile::new()?;
    writeln!(temp_file, "raise 'This is a runtime error'")?;
    let temp_path = temp_file.path();

    let wasm_path = wasm_path("runtime_error");
    // The compilation should succeed - the error happens at runtime
    run_ruvy(&wasm_path, &temp_path.to_string_lossy(), None)?;

    // The error should be caught when we try to run the WASM
    let result = run_wasm(&wasm_path, "");
    assert!(
        result.is_err(),
        "Expected WASM execution to fail with Ruby runtime error"
    );
    Ok(())
}
