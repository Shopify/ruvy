# Ruvy: A Ruby to WebAssembly toolchain

## About this repo

Ruvy aims to initialize the ruby VM using wizer and execute ruby code passed into the wasm.

## Build

- [rustup](https://rustup.rs/)
- Rust v1.60 (`rustup install 1.60 && rustup override set 1.60`)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On Mac it can be installed with `homebrew` via `brew
  install cmake`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`
- Install the `wasi-sdk` by running `make download-wasi-sdk`
- Wizer v1.6 (`cargo install wizer --all-features --version 1.6.0`)

## Development

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`

## Building

After all the dependencies are installed, run `make`

## Known limitations

The size of your main ruby program is limited by the max allowable limit for environment variables on your operating system. This is because we use a temporary env var to pass in the contents of the ruby program to the Wasm.

## Usage
A simple ruby program that prints "Hello world" to stdout
```
$ cargo run --package=cli ruby_examples/hello_world.rb
$ wasmtime index.wasm
Hello world
```

You can preload files by pointing to a directory of ruby files. At the moment, it just naively loads each file 1 by 1.
```
$ cargo run --package=cli --preload=prelude/ ruby_examples/use_preludes_and_stdin.rb
$ echo "this is my input" | wasmtime index.wasm
{:discount_input=>"this is my input", :value=>100.0}
```
