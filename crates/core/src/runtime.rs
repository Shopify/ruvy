use std::fs;

use anyhow::{bail, Result};
use ruvy_wasm_sys::{rb_eval_string_protect, ruby_init, ruby_init_loadpath, VALUE};
use std::{ffi::CString, os::raw::c_char};

pub fn init_ruby() {
    unsafe {
        ruby_init();
        ruby_init_loadpath();
    }
}

pub fn eval(code: &str) -> Result<VALUE> {
    let c_code = CString::new(code)?;
    let mut state: i32 = 0;
    let result =
        unsafe { rb_eval_string_protect(c_code.as_ptr() as *const c_char, &mut state as *mut i32) };

    if state != 0 {
        bail!("Error evaluating Ruby code. State: {state}");
    }
    Ok(result)
}

pub fn preload_files(path: String) {
    let entries = fs::read_dir(path).unwrap();

    entries
        .map(|r| r.map(|d| d.path()))
        .filter(|r| r.is_ok() && r.as_deref().unwrap().is_file())
        .for_each(|e| {
            let prelude_contents = fs::read_to_string(e.unwrap()).unwrap();
            eval(&prelude_contents).unwrap();
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruvy_wasm_sys::rb_num2int;

    #[test]
    fn test_int() {
        init_ruby();
        let result = unsafe { rb_num2int(eval("1 + 1").unwrap()) };
        assert_eq!(result, 2);
    }
}
