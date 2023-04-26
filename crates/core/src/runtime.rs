use anyhow::{anyhow, Result};
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

    if state == 0 {
        Ok(result)
    } else {
       Err(anyhow!("Error evaluating Ruby code. State: {}", state))
    }
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
