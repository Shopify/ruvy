use std::{
    path::PathBuf,
    fs,
    process
};

use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(name = "ruvy_cli", about = "Compile ruby code into a Wasm module.")]
struct Opt {
    ruby_file: PathBuf
}

fn main() {
    let opt = Opt::from_args();
    let ruby_code = match fs::read_to_string(&opt.ruby_file) {
        Ok(code) => code,
        Err(err) => {
            eprintln!("Error reading Ruby file {}: {}", opt.ruby_file.display(), err);
            process::exit(1);
        }
    };
    println!("Ruby code: {}", ruby_code);
}
