use ruvy::runtime;
use std::fs;

fn main() {
    let code = include_str!("../call_inspect.rb");
    runtime::eval(code).unwrap();
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
