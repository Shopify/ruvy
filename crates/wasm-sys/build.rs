use std::{env, path::PathBuf};

fn main() {
    let wasi_sdk_path = "/opt/wasi-sdk/wasi-sdk-12.0";
    let sysroot = format!("--sysroot={}/share/wasi-sysroot", &wasi_sdk_path);
    let lib_dir = env::current_dir().unwrap().join("lib");

    let include_dir = PathBuf::from("include/ruby-3.2.0+3");
    let include_config_dir = PathBuf::from("include/ruby-3.2.0+3/wasm32-wasi");

    env::set_var("CC", "/opt/wasi-sdk/wasi-sdk-12.0/bin/clang");
    env::set_var("AR", "/opt/wasi-sdk/wasi-sdk-12.0/bin/ar");
    env::set_var("CFLAGS", &sysroot);

    println!("cargo:rustc-link-search={}", lib_dir.display());
    println!("cargo:rustc-link-lib=static=ruby-static");

    let bindings = bindgen::Builder::default()
	.header("wrapper.h")
	.parse_callbacks(Box::new(bindgen::CargoCallbacks))
	.clang_args(&["-fvisibility=default", "--target=wasm32-wasi", &sysroot, &format!("-I{}", include_dir.display()), &format!("-I{}", include_config_dir.display())])
	.generate()
	.unwrap();

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
}
