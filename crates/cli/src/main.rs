use anyhow::Result;
use clap::Parser;
use std::{fs, path::PathBuf, process};
use wasmtime::{Config, Engine, Linker, Store};
use wasmtime_wasi::{
    p1::WasiP1Ctx, p2::pipe::MemoryInputPipe, DirPerms, FilePerms, WasiCtxBuilder,
};
use wasmtime_wizer::Wizer;

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

#[tokio::main]
async fn main() -> Result<()> {
    let opt = Opt::parse();
    let ruby_code = match fs::read_to_string(&opt.input) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error reading Ruby file {}: {}", opt.input.display(), err);
            process::exit(1);
        }
    };

    let ruby_engine = include_bytes!(concat!(env!("OUT_DIR"), "/engine.wasm"));
    let user_wasm = wizen(ruby_engine, &ruby_code, opt.preload).await?;

    fs::write(opt.output, user_wasm)?;
    Ok(())
}

async fn wizen(
    ruby_engine: &[u8],
    ruby_code: &str,
    preload_path: Option<PathBuf>,
) -> Result<Vec<u8>> {
    let mut cfg = Config::new();
    cfg.async_support(true);
    let engine = Engine::new(&cfg)?;
    let mut store = Store::new(&engine, wasi(ruby_code, preload_path)?);
    let user_wasm = Wizer::new()
        .run(&mut store, ruby_engine, async |store, module| {
            let engine = store.engine();
            let mut linker = Linker::new(engine);
            wasmtime_wasi::p1::add_to_linker_async(&mut linker, |cx| cx)?;
            let instance = linker.instantiate_async(store, module).await?;
            Ok(instance)
        })
        .await?;
    Ok(user_wasm)
}

fn wasi(ruby_code: &str, preload_path: Option<PathBuf>) -> Result<WasiP1Ctx> {
    let mut wasi_builder = WasiCtxBuilder::new();
    wasi_builder
        .stdin(MemoryInputPipe::new(ruby_code.as_bytes().to_owned()))
        .inherit_stdout()
        .inherit_stderr();
    if let Some(preload_path) = preload_path {
        let guest_preload_path = preload_path.to_string_lossy();
        wasi_builder
            .env("RUVY_PRELOAD_PATH", &guest_preload_path)
            .preopened_dir(
                &preload_path,
                &guest_preload_path,
                DirPerms::READ,
                FilePerms::READ,
            )
            .map(|_| ())?;
    }
    Ok(wasi_builder.build_p1())
}
