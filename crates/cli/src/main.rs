use anyhow::Result;
use clap::Parser;
use std::{
    fs,
    path::{Path, PathBuf},
    process,
    rc::Rc,
    sync::OnceLock,
};
use wasmtime::Linker;
use wasmtime_wasi::{pipe::MemoryInputPipe, DirPerms, FilePerms, WasiCtxBuilder};
use wizer::{StoreData, Wizer};

static INPUT: OnceLock<MemoryInputPipe> = OnceLock::new();
static PRELOAD_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();

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
    // We lose the ability to return an error when performing this operation
    // inside `add_to_linker_sync` so we perform the same operation here so
    // can return an error if it fails.
    if let Some(preload_path) = &preload_path {
        let mut wasi_builder = WasiCtxBuilder::new();
        add_preload_path_to_wasi_ctx(&mut wasi_builder, preload_path)?;
    }

    // We can't move the ruby code or preload path into the `make_linker`
    // since they don't implement copy so put them in a static `OnceLock`
    // instead. This assumes this code is only only called once during the
    // process's lifetime.
    INPUT
        .set(MemoryInputPipe::new(ruby_code.as_bytes().to_vec()))
        .expect("Input OnceLock should not be set at this point");
    PRELOAD_PATH
        .set(preload_path)
        .expect("Preload path OnceLock should not be set at this point");

    let mut wizer = Wizer::new();
    wizer
        .wasm_bulk_memory(true)
        .make_linker(Some(Rc::new(|engine| {
            let mut linker = Linker::new(engine);
            wasmtime_wasi::preview1::add_to_linker_sync(&mut linker, |cx: &mut StoreData| {
                if cx.wasi_ctx.is_none() {
                    let mut wasi_builder = WasiCtxBuilder::new();
                    wasi_builder
                        .stdin(INPUT.get().unwrap().clone())
                        .inherit_stdout()
                        .inherit_stderr();
                    if let Some(preload_path) = PRELOAD_PATH.get().unwrap() {
                        wasi_builder.env("RUVY_PRELOAD_PATH", preload_path.to_string_lossy());
                        add_preload_path_to_wasi_ctx(&mut wasi_builder, preload_path)
                            .expect("Should have failed earlier when tested earlier");
                    }
                    cx.wasi_ctx = Some(wasi_builder.build_p1());
                }
                cx.wasi_ctx.as_mut().unwrap()
            })?;
            Ok(linker)
        })))?;

    Ok(wizer)
}

fn add_preload_path_to_wasi_ctx(
    wasi_builder: &mut WasiCtxBuilder,
    preload_path: &Path,
) -> Result<()> {
    wasi_builder
        .preopened_dir(
            preload_path,
            preload_path.to_string_lossy(),
            DirPerms::READ,
            FilePerms::READ,
        )
        .map(|_| ())
}
