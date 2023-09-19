mod runtime;

use once_cell::sync::OnceCell;
use std::env;

static USER_CODE: OnceCell<String> = OnceCell::new();

fn main() {
    let code = USER_CODE.get().unwrap();
    runtime::eval(&code).unwrap();

    const EXPECTED_SUCCESS_RET_VAL: i32 = 0;
    // ruby_cleanup expects an integer as an argument that will be returned if it ran successfully.
    let cleanup_status = unsafe { ruvy_wasm_sys::ruby_cleanup(EXPECTED_SUCCESS_RET_VAL) };
    if cleanup_status != EXPECTED_SUCCESS_RET_VAL {
        panic!("ruby_cleanup did not run successfully. Return value: {cleanup_status}");
    }
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
