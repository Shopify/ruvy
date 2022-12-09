use std::{os::raw::c_char, ffi::CString, io::Read};
use ruvy_wasm_sys::{ruby_init, rb_eval_string_protect, rb_load_file};
use std::{io, env};

const PRELUDE: &'static str = r#"
  module X
    def self.x
	1 + 1
    end
  end
"#;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
	panic!("expected exactly two arguments");
    }

    let input = &args[0];
    let program = &args[1];

    unsafe {
	let prog = CString::new(program.as_str()).unwrap();
	let prog = prog.as_ptr() as *const c_char;
	let state = 0i32;

	rb_eval_string_protect(prog, state as *mut i32);

	assert!(state == 0);
    }
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
	ruby_init();
	let prelude = CString::new(PRELUDE).unwrap();
	let prelude = prelude.as_ptr() as *const c_char;

	let mut state = 0i32;
	rb_eval_string_protect(prelude, state as *mut i32);
	assert!(state == 0);
    }
}
