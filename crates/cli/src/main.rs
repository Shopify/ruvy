use anyhow::{bail, Result};
use clap::Parser;
use std::{env, fs, io::Write, path::PathBuf, process};
use wizer::Wizer;

#[derive(Debug, Parser)]
#[clap(name = "ruvy_cli", about = "Compile ruby code into a Wasm module.")]
struct Opt {
    /// Path of the Ruby input file.
    input: PathBuf,

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
    let wizen = env::var("RUVY_WIZEN");

    if wizen.eq(&Ok("1".into())) {
        let engine = include_bytes!("../engine.wasm");
        let user_wasm = Wizer::new()
            .allow_wasi(true)?
            .init_func("load_user_code")
            .run(engine)?;
        fs::write(opt.output, user_wasm)?;
        env::remove_var("RUVY_WIZEN");
    } else {
        let self_cmd = env::args().next().unwrap();
        {
            env::set_var("RUVY_WIZEN", "1");
            let mut command = process::Command::new(self_cmd)
                .arg(&opt.input)
                .arg("-o")
                .arg(&opt.output)
                .stdin(process::Stdio::piped())
                .spawn()?;
            command
                .stdin
                .take()
                .unwrap()
                .write_all(&ruby_code.as_bytes())?;
            let status = command.wait()?;
            if !status.success() {
                bail!("Couldn't create wasm from input");
            }
        }
    }
    Ok(())
}
