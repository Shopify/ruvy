use std::{
    env,
    path::Path,
    process::{Command, Output},
    str,
};

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
    assert_eq!("Hello world\n", output.stdout);
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
        output.stdout
    );
    Ok(())
}

struct Context {
    wasi: WasiP1Ctx,
    out_stream: MemoryOutputPipe,
    err_stream: MemoryOutputPipe,
}

impl Context {
    fn new(input: &[u8]) -> Context {
        let out_stream = MemoryOutputPipe::new(usize::MAX);
        let err_stream = MemoryOutputPipe::new(usize::MAX);
        Context {
            wasi: WasiCtxBuilder::new()
                .stdin(MemoryInputPipe::new(input.to_vec()))
                .stdout(out_stream.clone())
                .stderr(err_stream.clone())
                .build_p1(),
            out_stream,
            err_stream,
        }
    }
}

fn wasm_path(test_name: &str) -> String {
    format!("{}/{test_name}.wasm", env!("CARGO_TARGET_TMPDIR"))
}

fn exec_ruvy(wasm_path: &str, input_path: &str, preload: Option<&str>) -> Result<Output> {
    let mut args = vec![format!("-o{wasm_path}")];
    if let Some(preload) = preload {
        args.push(format!("--preload={preload}"));
    }
    args.push(input_path.to_string());

    Ok(Command::new(env!("CARGO_BIN_EXE_ruvy"))
        .args(args)
        .output()?)
}

fn run_ruvy(wasm_path: &str, input_path: &str, preload: Option<&str>) -> Result<()> {
    let output = exec_ruvy(wasm_path, input_path, preload)?;
    let status = output.status;
    if !status.success() {
        bail!("Failed to execute ruvy");
    }
    Ok(())
}

#[derive(Debug)]
struct WasmStream {
    stdout: String,
    stderr: String,
}

fn run_wasm(wasm_path: impl AsRef<Path>, input: &str) -> Result<WasmStream> {
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
    let stdout = String::from_utf8(context.out_stream.contents().to_vec())?;
    let stderr = String::from_utf8(context.err_stream.contents().to_vec())?;

    Ok(WasmStream { stdout, stderr })
}

#[test]
pub fn test_preload_error_handling() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let invalid_file = temp_dir.path().join("invalid.rb");
    let mut file = fs::File::create(&invalid_file)?;
    writeln!(file, "raise 'intentional preload error'")?;

    let wasm_path = wasm_path("preload_error");
    let input_path = "../../ruby_examples/hello_world.rb";
    let preload = temp_dir.path().to_string_lossy();

    let output = exec_ruvy(&wasm_path, input_path, Some(&preload))?;
    let stderr_str = String::from_utf8_lossy(&output.stderr);

    assert!(
        !output.status.success(),
        "Expected preload status to be nonzero. Status: {:?}, Stderr: {}",
        output.status,
        stderr_str
    );
    assert!(
        stderr_str.contains("intentional preload error"),
        "Expected preload stderr to contain Ruby exception. Status: {:?}, Stderr: {}",
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
    run_ruvy(&wasm_path, &temp_path.to_string_lossy(), None)?;

    let output = run_wasm(&wasm_path, "")?;
    assert!(
        output.stderr.contains("This is a runtime error"),
        "Expected runtime stderr to contain Ruby exception. Stderr: {}",
        output.stderr
    );
    Ok(())
}
