use std::{env, io::Cursor, path::Path, process::Command, str};

use anyhow::{bail, Result};
use wasi_common::{
    pipe::{ReadPipe, WritePipe},
    sync::WasiCtxBuilder,
    WasiCtx,
};
use wasmtime::{Engine, Linker, Module, Store};

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
    wasi: WasiCtx,
    out_stream: WritePipe<Cursor<Vec<u8>>>,
}

impl Context {
    fn new(input: &[u8]) -> Context {
        let out_stream = WritePipe::new_in_memory();
        Context {
            wasi: WasiCtxBuilder::new()
                .stdin(Box::new(ReadPipe::from(input)))
                .stdout(Box::new(out_stream.clone()))
                .build(),
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
    wasi_common::sync::add_to_linker(&mut linker, |s: &mut Context| &mut s.wasi)?;
    let mut store = Store::new(&engine, Context::new(input.as_bytes()));

    let module = Module::from_file(&engine, wasm_path)?;
    linker
        .instantiate(&mut store, &module)?
        .get_typed_func::<(), ()>(&mut store, "_start")?
        .call(&mut store, ())?;

    let context = store.into_data();
    drop(context.wasi);
    let output = context.out_stream.try_into_inner().unwrap().into_inner();
    let output = String::from_utf8(output)?;

    Ok(output)
}
