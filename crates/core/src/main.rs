mod runtime;

use once_cell::sync::OnceCell;
use runtime::cleanup_ruby;
use std::env;

static USER_CODE: OnceCell<String> = OnceCell::new();

fn main() {
    let code = USER_CODE.get().unwrap();
    runtime::eval(&code).unwrap();
    cleanup_ruby().unwrap();
}

#[export_name = "load_user_code"]
pub extern "C" fn load_user_code() {
    if let Ok(preload_path) = env::var("RUVY_PRELOAD_PATH") {
        runtime::preload_files(preload_path);
    }

    let contents = env::var("RUVY_USER_CODE").unwrap();
    USER_CODE.set(contents).unwrap();
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    runtime::init_ruby();
}
