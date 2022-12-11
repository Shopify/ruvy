use ruvy_wasm_sys::{
    rb_eval_string_protect, rb_load_file, rb_string_value_cstr, ruby_init, ruby_set_argv, VALUE,
    rb_define_readonly_variable, rb_str_new_cstr, rb_define_global_const
};
use std::{env, fs, io};
use std::{
    ffi::{CStr, CString},
    io::Read,
    os::raw::c_char,
};

fn main() {
    // args = (program name, input, program)
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        panic!("expected exactly two arguments");
    }

    // // let mut argv = vec![args[1].clone()]
    // let mut argv = vec![args[1].as_str()]
    //     .iter()
    //     .map(|a| {
    // 	    let cstring = CString::new(*a).unwrap();
    // 	    cstring.as_ptr() as *mut _
    // 	})
    //     .collect::<Vec<_>>();

    let program = &args[2];
    let input = &args[1];

    unsafe {
	// Define INPUT constant
	let var_name = CString::new("SHOPIFY_INPUT").unwrap();
	let var_name = var_name.as_ptr() as *const c_char;
	let val = CString::new(input.as_str()).unwrap();
	let val = val.as_ptr() as *const c_char;
	let val = rb_str_new_cstr(val);
	rb_define_global_const(var_name, val);
	
	// Exec program
        let prog = CString::new(program.as_str()).unwrap();
        let prog = prog.as_ptr() as *const c_char;
        let state = 0i32;

        let mut val = rb_eval_string_protect(prog, state as *mut i32);
        let ptr = rb_string_value_cstr(&mut val);
        let cstr = CStr::from_ptr(ptr);

        println!("return: {}", cstr.to_str().unwrap());

        assert!(state == 0);
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
