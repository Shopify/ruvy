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

### Using a different ruby.wasm

Set the `RUVY_WASM_SYS_RUBY_PATH` environment variable to a path containing an extracted release asset from https://github.com/ruby/ruby.wasm. The directory the environment variable is set to must contain an `include` and `lib` directory.

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

## Ideas for contributions

Here are some ideas for welcome contributions!

### Improving compatibility with Shopify Functions

- Investigate and improve performance of simple Ruvy modules. At the present time, `puts "Hello world"` modules can take 45 milliseconds to execute. It should be substantially faster, ideally under a millisecond.
- Shrinking the size of modules by separating the interpreter into an engine Wasm module which exports memory and functions that can be imported by a Wasm module containing Ruby source code. To see an example of implementing this approach, take a look at https://github.com/bytecodealliance/javy, specifically the [core lib.rs](https://github.com/bytecodealliance/javy/blob/3b02858c4a68c830e8e82a1b15b4c3817ad1a64a/crates/core/src/lib.rs) and [the dynamic wasm generator](https://github.com/bytecodealliance/javy/blob/3b02858c4a68c830e8e82a1b15b4c3817ad1a64a/crates/cli/src/wasm_generator/dynamic.rs).
- Enable exports of named functions from Wasm that call into named functions in Ruby code so multiple functions can be exported.

### Misc

- Enable using `require` and Ruby gems. At the present time, using code in the preload directory is the only way to add dependencies and large parts of the standard library are not available. It should be possible to enable `require` to work and to load both code from the standard library and from third party gems that are not native gems. A good example of showing this is fixed would be adding a Ruby example that uses the standard library's `json` library to parse and dump JSON.
- Output any error messages from the Ruby VM on the standard error stream.
