# Ruvy: A Ruby to WebAssembly toolchain

## About this repo

Ruvy aims to initialize the ruby VM using wizer and execute ruby code passed into the wasm.

## Build

- [rustup](https://rustup.rs/)
- Stable Rust (`rustup install stable && rustup default stable`)
- wasm32-wasi, can be installed via `rustup target add wasm32-wasi`
- cmake, depending on your operating system and architecture, it might not be
  installed by default. On Mac it can be installed with `homebrew` via `brew
  install cmake`
- Rosetta 2 if running MacOS on Apple Silicon, can be installed via
  `softwareupdate --install-rosetta`
- Install the `wasi-sdk` by running `make download-wasi-sdk`

## Development

- wasmtime-cli, can be installed via `cargo install wasmtime-cli` (required for
  `cargo-wasi`)
- cargo-wasi, can be installed via `cargo install cargo-wasi`

## Building

After all the dependencies are installed, run `make`. You
should now have access to the executable in `target/release/ruvy`

## Example

```
$ make
$ cat example.rb | wizer crates/cli/engine.wasm --allow-wasi -f load_user_code -o index.wasm
$ echo "this is a discount input" | wasmtime index.wasm
{:discount_input=>"this is a discount input", :value=>100.0}
```
