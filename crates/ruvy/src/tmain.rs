use ruvy_wasm_sys::{
    rb_eval_string_protect, rb_load_file, rb_string_value_cstr, ruby_init, ruby_set_argv, VALUE,
    rb_define_readonly_variable, rb_str_new_cstr, rb_define_global_const, ruby_init_loadpath
};
use std::{env, fs, io};
use std::{
    ffi::{CStr, CString},
    io::Read,
    os::raw::c_char,
};


fn main() {
    unsafe {
        // Initialize the Ruby interpreter
        ruby_init();
        ruby_init_loadpath();

        // Load and execute the Ruby script
        let ruby_script = include_str!("../call_inspect.rb");
        let c_ruby_script = CString::new(ruby_script).unwrap();
        let mut state: i32 = 0;
        let result = rb_eval_string_protect(c_ruby_script.as_ptr() as *const c_char, &mut state as *mut i32);

        if state != 0 {
            eprintln!("Error executing Ruby script: {}", state);
        } else {
            println!("Ruby script executed successfully.");
        }
    }
}

#[export_name = "wizer.initialize"]
pub extern "C" fn init() {
    unsafe {
        ruby_init();

        let entries = fs::read_dir("./prelude").unwrap();

        entries
            .map(|r| r.map(|d| d.path()))
            .filter(|r| r.is_ok() && r.as_deref().unwrap().is_file())
            .for_each(|e| {
                let prelude_contents = fs::read_to_string(e.unwrap()).unwrap();
                let prelude_item = CString::new(prelude_contents.as_str()).unwrap();
                let ptr = prelude_item.as_ptr() as *const c_char;
                let mut state = 0i32;
                rb_eval_string_protect(ptr, state as *mut i32);
                assert!(state == 0);
            });
    }
}
