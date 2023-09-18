use anyhow::Result;
use clap::Parser;
use std::{env, fs, path::PathBuf, process};
use wizer::Wizer;

#[derive(Debug, Parser)]
#[clap(name = "ruvy_cli", about = "Compile ruby code into a Wasm module.")]
struct Opt {
    /// Path of the Ruby input file.
    input: PathBuf,

    /// Path of a directory containing Ruby files to preload to be used by the input file.
    #[clap(long, parse(from_os_str))]
    preload: Option<PathBuf>,

    #[clap(short, parse(from_os_str), default_value = "index.wasm")]
    /// Desired path of the WebAssembly output file.
    output: PathBuf,
}

fn main() -> Result<()> {
    let opt = Opt::from_args();
    let ruby_code = match fs::read_to_string(&opt.input) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error reading Ruby file {}: {}", opt.input.display(), err);
            process::exit(1);
        }
    };

    env::set_var("RUVY_USER_CODE", ruby_code);

    let engine = include_bytes!("../engine.wasm");
    let mut wizer = Wizer::new();
    wizer
        .allow_wasi(true)?
        .wasm_bulk_memory(true)
        .inherit_env(true)
        .init_func("load_user_code");

    if let Some(preload_path) = opt.preload {
        env::set_var("RUVY_PRELOAD_PATH", &preload_path);
        wizer.dir(preload_path);
    }

    let user_wasm = wizer.run(engine)?;
    fs::write(opt.output, user_wasm)?;

    env::remove_var("RUVY_USER_CODE");
    env::remove_var("RUVY_PRELOAD_PATH");

    Ok(())
}
