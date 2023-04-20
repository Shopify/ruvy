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
        let ruby_script = include_str!("../hello_world.rb");
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

