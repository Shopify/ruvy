use anyhow::Result;
use clap::Parser;
use std::{fs, path::PathBuf, process, rc::Rc, sync::OnceLock};
use wasmtime::Linker;
use wasmtime_wasi::{
    pipe::MemoryInputPipe, preview1::WasiP1Ctx, DirPerms, FilePerms, WasiCtxBuilder,
};
use wizer::{StoreData, Wizer};

static mut WASI: OnceLock<WasiP1Ctx> = OnceLock::new();

#[derive(Debug, Parser)]
#[clap(name = "ruvy_cli", about = "Compile ruby code into a Wasm module.")]
struct Opt {
    /// Path of the Ruby input file.
    input: PathBuf,

    /// Path of a directory containing Ruby files to preload to be used by the input file.
    #[arg(long)]
    preload: Option<PathBuf>,

    #[arg(short, default_value = "index.wasm")]
    /// Desired path of the WebAssembly output file.
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::parse();
    let ruby_code = match fs::read_to_string(&opt.input) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error reading Ruby file {}: {}", opt.input.display(), err);
            process::exit(1);
        }
    };

    let engine = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
    let wizer = setup_wizer(&ruby_code, opt.preload)?;
    let user_wasm = wizer.run(engine)?;
    fs::write(opt.output, user_wasm)?;

    Ok(())
}

fn setup_wizer(ruby_code: &str, preload_path: Option<PathBuf>) -> Result<Wizer> {
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder
        .stdin(MemoryInputPipe::new(ruby_code.as_bytes().to_vec()))
        .inherit_stdout()
        .inherit_stderr();

    if let Some(preload_path) = preload_path {
        wasi_builder
            .env("RUVY_PRELOAD_PATH", &preload_path.to_string_lossy())
            .preopened_dir(
                &preload_path,
                preload_path.to_string_lossy(),
                DirPerms::READ,
                FilePerms::READ,
            )?;
    }

    // We can't move the WasiCtx into `make_linker` since WasiCtx doesn't implement the `Copy` trait.
    // So we move the WasiCtx into a mutable static OnceLock instead.
    // Setting the value in the OnceLock and getting the reference back from it should be safe given
    // we're never executing this code concurrently or more than once.
    if unsafe { WASI.set(wasi_builder.build_p1()) }.is_err() {
        panic!("Failed to set WASI static variable");
    };

    let mut wizer = Wizer::new();
    wizer
        .wasm_bulk_memory(true)
        .make_linker(Some(Rc::new(|engine| {
            let mut linker = Linker::new(engine);
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |cx: &mut StoreData| {
                if cx.wasi_ctx.is_none() {
                    cx.wasi_ctx = Some(unsafe { WASI.take() }.unwrap());
                }
                cx.wasi_ctx.as_mut().unwrap()
            })?;
            Ok(linker)
        })))?;

    Ok(wizer)
}
