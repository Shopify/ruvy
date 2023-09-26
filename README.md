# Ruvy: A Ruby to WebAssembly toolchain

## About this repo

Ruvy aims to initialize the ruby VM using wizer and execute ruby code passed into the wasm.

## Build

- [rustup](https://rustup.rs/)
- Latest Rust stable version
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On Mac it can be installed with `homebrew` via `brew
install cmake`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`

## Development

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`

### Using a different WASI SDK

The following environment variables allow you to experiment with different WASI SDKs:

- `RUVY_WASM_SYS_WASI_SDK_MAJOR_VERSION` sets the major version of the WASI SDK to use
- `RUVY_WASM_SYS_WASI_SDK_MINOR_VERSION` sets the minor version of the WASI SDK to use
- `RUVY_WASM_SYS_WASI_SDK_PATH` allows you to specify a path to WASI SDK to use

## Building

After all the dependencies are installed, run `make`

## Usage

A simple ruby program that prints "Hello world" to stdout

```
$ cargo run --package=cli ruby_examples/hello_world.rb
$ wasmtime index.wasm
Hello world
```

You can preload files by pointing to a directory of ruby files. At the moment, it just naively loads each file 1 by 1.

```
$ cargo run --package=cli -- --preload=prelude/ ruby_examples/use_preludes_and_stdin.rb
$ echo "this is my input" | wasmtime index.wasm
{:discount_input=>"this is my input", :value=>100.0}
```
