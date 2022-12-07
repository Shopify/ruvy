use std::{os::raw::c_char, ffi::CString};
use ruvy_wasm_sys::{ruby_init, rb_eval_string_protect, rb_load_file};
use std::env;

const PRELUDE: &'static str = r#"
  Module X
    def x
	1 + 1
    end
  end
"#;

fn main() {
    unsafe {
	let prog = CString::new("puts 1").unwrap();
	let prog = prog.as_ptr() as *const c_char;
	let mut state = 0i32;


	rb_eval_string_protect(prog, state as *mut i32);

	println!("state: {}", state);
    }
}

// fn load_prelude() {
//     let mut state = 0i32;
//     let cwd = env::current_dir().unwrap();
//     let prelude_path = cwd.join("prelude").join("x.rb");
//     let prog = CString::new(prelude_path.to_str().unwrap()).unwrap();
//     let prog = prog.as_ptr() as *const c_char;


//     unsafe {
// 	rb_load_file(prog);
//     }
// }


#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
	ruby_init();
    }
}
