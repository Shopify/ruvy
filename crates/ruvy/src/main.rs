use std::{os::raw::c_char, ffi::CString};
use ruvy_wasm_sys::{ruby_init, rb_eval_string_protect};
fn main() {
    unsafe {
	ruby_init();

	let prog = CString::new("1 + 1").unwrap();
	let prog = prog.as_ptr() as *const c_char;
	let mut state = 0i32;


	let ret = rb_eval_string_protect(prog, state as *mut i32);

	println!("{}", ret);
    }
}


#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
	ruby_init();
    }
}
