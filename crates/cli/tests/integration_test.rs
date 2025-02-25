use std::{env, path::Path, process::Command, str};

use anyhow::{bail, Result};
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
