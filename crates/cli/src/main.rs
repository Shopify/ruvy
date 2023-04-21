use std::{
    path::PathBuf,
    fs,
    process,
    env,
    io::Write
};
use structopt::StructOpt;
use wizer::Wizer;
use anyhow::{bail, Result};

#[derive(Debug, StructOpt)]
#[structopt(name = "ruvy_cli", about = "Compile ruby code into a Wasm module.")]
struct Opt {
    ruby_file: PathBuf
}

fn main() -> Result<()>{
    let opt = Opt::from_args();
    let ruby_code = match fs::read_to_string(&opt.ruby_file) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error reading Ruby file {}: {}", opt.ruby_file.display(), err);
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
        fs::write("index.wasm", user_wasm)?;
        env::remove_var("JAVY_WIZEN");
    } else {
        let self_cmd = env::args().next().unwrap();
        {
            env::set_var("RUVY_WIZEN", "1");
            let mut command = process::Command::new(self_cmd)
                .arg(&opt.ruby_file)
                .stdin(process::Stdio::piped())
                .spawn()?;
            command.stdin.take().unwrap().write_all(&ruby_code.as_bytes())?;
            let status = command.wait()?;
            if !status.success() {
                bail!("Couldn't create wasm from input");
            }
        }
    }
    Ok(())
}
