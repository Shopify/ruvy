.DEFAULT_GOAL := cli

cli: core
	cargo build --package=cli

core:
	cargo build --package=core --release --target=wasm32-wasi
	wizer --allow-wasi --wasm-bulk-memory true target/wasm32-wasi/release/core.wasm -o crates/cli/engine.wasm

tests: test-cli test-core
		
test-cli: cli
	cargo test --package=cli -- --nocapture

test-core:
	cargo wasi test --package=core -- --nocapture

fmt: fmt-wasm-sys fmt-core fmt-cli

fmt-wasm-sys:
	cargo fmt --package=ruvy-wasm-sys -- --check
	cargo clippy --package=ruvy-wasm-sys --target=wasm32-wasi -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-core:
	cargo fmt --package=core -- --check
	cargo clippy --package=core --target=wasm32-wasi --all-targets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-cli:
	cargo fmt --package=cli -- --check
	cargo clippy --package=cli --all-targets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious
