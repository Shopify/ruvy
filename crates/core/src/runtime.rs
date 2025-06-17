use std::fs;

use anyhow::{anyhow, bail, Result};
use ruvy_wasm_sys::{
    rb_errinfo, rb_eval_string_protect, rb_obj_as_string, rb_string_value_ptr, ruby_init,
    ruby_init_loadpath, RUBY_Qnil, VALUE,
};
use std::{
    ffi::{CStr, CString},
    io::{self, Write},
    os::raw::c_char,
};

fn extract_ruby_error() -> Option<String> {
    unsafe {
        let error_obj = rb_errinfo();
        if error_obj == RUBY_Qnil {
            return None;
        }

        let error_string = rb_obj_as_string(error_obj);
        let mut error_val = error_string;
        let error_ptr = rb_string_value_ptr(&mut error_val);
        if error_ptr.is_null() {
            return None;
        }

        let error_cstr = CStr::from_ptr(error_ptr);
        let error_msg = error_cstr.to_string_lossy().into_owned();

        Some(error_msg)
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
        Err(anyhow!(
            "Error evaluating Ruby code. State: {}, message: {}",
            state,
            error_msg.as_deref().unwrap_or("None")
        ))
    }
}

pub fn preload_files(path: String) -> Result<()> {
    let entries = fs::read_dir(&path)
        .map_err(|e| anyhow!("Failed to read preload directory '{}': {}", path, e))?;

    for entry in entries {
        let path = entry?.path();
        if path.is_file() {
            let prelude_contents = fs::read_to_string(&path)
                .map_err(|e| anyhow!("Failed to read file '{}': {}", path.display(), e))?;

            if let Err(e) = eval(&prelude_contents) {
                return Err(e.context(format!(
                    "Failed to evaluate preload file '{}'",
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

    #[test]
    fn test_extract_ruby_error_with_syntax_error() {
        init_ruby();
        let result = eval("1 + + 1");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Ruby evaluation failed"));
    }

    #[test]
    fn test_extract_ruby_error_with_name_error() {
        init_ruby();
        let result = eval("undefined_variable");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Ruby evaluation failed"));
    }

    #[test]
    fn test_extract_ruby_error_with_runtime_error() {
        init_ruby();
        let result = eval("raise 'custom error message'");
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Ruby evaluation failed"));
        assert!(error_msg.contains("custom error message"));
    }

    #[test]
    fn test_preload_files_with_nonexistent_directory() {
        let result = preload_files("/nonexistent/directory".to_string());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to read preload directory"));
    }

    #[test]
    fn test_preload_files_with_invalid_ruby_file() {
        use std::fs;
        use std::io::Write;
        use tempfile::TempDir;

        init_ruby();

        let temp_dir = TempDir::new().unwrap();
        let invalid_file = temp_dir.path().join("invalid.rb");
        let mut file = fs::File::create(&invalid_file).unwrap();
        writeln!(file, "1 + + 1").unwrap();

        let result = preload_files(temp_dir.path().to_string_lossy().to_string());
        let error_msg = result.unwrap_err().to_string();
        assert!(error_msg.contains("Failed to evaluate preload file"));
    }

    #[test]
    fn test_preload_files_with_valid_ruby_file() {
        use std::fs;
        use std::io::Write;
        use tempfile::TempDir;

        init_ruby();

        let temp_dir = TempDir::new().unwrap();
        let valid_file = temp_dir.path().join("valid.rb");
        let mut file = fs::File::create(&valid_file).unwrap();
        writeln!(file, "$test_var = 42").unwrap();

        let result = preload_files(temp_dir.path().to_string_lossy().to_string());
        assert!(result.is_ok());

        let check_result = eval("$test_var");
        let value = unsafe { rb_num2int(check_result.unwrap()) };
        assert_eq!(value, 42);
    }
}
