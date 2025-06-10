use std::fs;

use anyhow::{anyhow, bail, Result};
use ruvy_wasm_sys::{
    rb_errinfo, rb_eval_string_protect, rb_obj_as_string, rb_set_errinfo, rb_string_value_ptr,
    ruby_init, ruby_init_loadpath, RUBY_Qnil, VALUE,
};
use std::{
    ffi::{CStr, CString},
    io::{self, Write},
    os::raw::c_char,
};

fn extract_ruby_error() -> String {
    unsafe {
        let error_obj = rb_errinfo();
        if error_obj == RUBY_Qnil {
            return "Unknown Ruby error".to_string();
        }

        let error_string = rb_obj_as_string(error_obj);
        let mut error_val = error_string;
        let error_ptr = rb_string_value_ptr(&mut error_val);
        if error_ptr.is_null() {
            return "Failed to extract error message".to_string();
        }

        let error_cstr = CStr::from_ptr(error_ptr);
        let error_msg = error_cstr.to_string_lossy().into_owned();

        // Clear the error info to prevent it from affecting subsequent operations
        rb_set_errinfo(RUBY_Qnil);

        error_msg
    }
}

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

    if state == 0 {
        Ok(result)
    } else {
        let error_msg = extract_ruby_error();
        let _ = writeln!(io::stderr(), "Ruby Error: {}", error_msg);
        Err(anyhow!(
            "Ruby evaluation failed with state {}: {}",
            state,
            error_msg
        ))
    }
}

pub fn preload_files(path: String) -> Result<()> {
    let entries = fs::read_dir(&path)
        .map_err(|e| anyhow!("Failed to read preload directory '{}': {}", path, e))?;

    for entry in entries {
        let entry = entry.map_err(|e| anyhow!("Failed to read directory entry: {}", e))?;
        let path = entry.path();

        if path.is_file() {
            let prelude_contents = fs::read_to_string(&path)
                .map_err(|e| anyhow!("Failed to read file '{}': {}", path.display(), e))?;

            if let Err(e) = eval(&prelude_contents) {
                let _ = writeln!(
                    io::stderr(),
                    "Error in preload file '{}': {}",
                    path.display(),
                    e
                );
                return Err(e.context(format!(
                    "Failed to evaluate preload file: {}",
                    path.display()
                )));
            }
        }
    }
    Ok(())
}

pub fn cleanup_ruby() -> Result<()> {
    const EXPECTED_SUCCESS_RET_VAL: i32 = 0;
    // ruby_cleanup expects an integer as an argument that will be returned if it ran successfully.
    let cleanup_status = unsafe { ruvy_wasm_sys::ruby_cleanup(EXPECTED_SUCCESS_RET_VAL) };
    if cleanup_status != EXPECTED_SUCCESS_RET_VAL {
        bail!("ruby_cleanup did not run successfully. Return value: {cleanup_status}");
    }
    Ok(())
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
