use ruvy::runtime;
use std::{fs, io::{self, Read} };

fn main() {
    let mut contents = String::new();
    io::stdin().read_to_string(&mut contents).unwrap();
    runtime::eval(&contents).unwrap();
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    runtime::init_ruby();

    let entries = fs::read_dir("./prelude").unwrap();

    entries
        .map(|r| r.map(|d| d.path()))
        .filter(|r| r.is_ok() && r.as_deref().unwrap().is_file())
        .for_each(|e| {
            let prelude_contents = fs::read_to_string(e.unwrap()).unwrap();
            runtime::eval(&prelude_contents).unwrap();
        });
}
