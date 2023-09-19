use std::{env, path::PathBuf};

fn main() {
    let this_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let wasi_sdk_path = format!("{}/wasi-sdk", this_dir);
    if !std::path::Path::exists(std::path::Path::new(&wasi_sdk_path)) {
        panic!(
            "wasi-sdk not installed in specified path of {}",
            &wasi_sdk_path
        );
    }
    let sysroot = format!("--sysroot={}/share/wasi-sysroot", &wasi_sdk_path);
    let sysroot_lib = format!("{}/share/wasi-sysroot/lib/wasm32-wasi", &wasi_sdk_path);
    let lib_dir = env::current_dir().unwrap().join("lib");

    let include_dir = PathBuf::from("include/ruby-3.2.0+3");
    let include_config_dir = PathBuf::from("include/ruby-3.2.0+3/wasm32-wasi");

    env::set_var("CC", format!("{}/bin/clang", &wasi_sdk_path));
    env::set_var("LD", format!("{}/bin/clang", &wasi_sdk_path));
    env::set_var("AR", format!("{}/bin/ar", &wasi_sdk_path));
    env::set_var("CFLAGS", format!("{} -D_WASI_EMULATED_SIGNAL", &sysroot));

    // Ruby lib directory
    println!("cargo:rustc-link-search={}", lib_dir.display());
    // WASI Sysroot directory
    println!("cargo:rustc-link-search={}", sysroot_lib);

    cc::Build::new()
        .file("foo.c")
        .flag_if_supported("-fdeclspec")
        .cargo_metadata(true)
        .include(&include_dir)
        .include(&include_config_dir)
        .target("wasm32-wasi")
        .compile("ruvy");

    let bindings = bindgen::Builder::default()
        .header("wrapper.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks))
        .clang_args(&[
            "-fvisibility=default",
            "--target=wasm32-wasi",
            &sysroot,
            &format!("-I{}", include_dir.display()),
            &format!("-I{}", include_config_dir.display()),
        ])
        .generate()
        .unwrap();

    println!("cargo:rustc-link-lib=static=ruby-static");
    println!("cargo:rustc-link-lib=static=m");
    println!("cargo:rustc-link-lib=static=wasi-emulated-signal");
    println!("cargo:rustc-link-lib=static=wasi-emulated-mman");
    println!("cargo:rustc-link-lib=static=wasi-emulated-process-clocks");
    println!("cargo:rustc-link-lib=static=c");
    println!("cargo:rustc-link-lib=static=crypt");
    println!("cargo:rustc-link-lib=static=pthread");
    println!("cargo:rustc-link-lib=static=rt");
    println!("cargo:rustc-link-lib=static=dl");
    println!("cargo:rustc-link-lib=static=resolv");
    println!("cargo:rustc-link-lib=static=util");

    println!("cargo:rerun-if-changed=build.rs");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings.write_to_file(out_dir.join("bindings.rs")).unwrap();
}
