.DEFAULT_GOAL := cli

cli: core
	cargo build --package=cli

core:
	cargo build --package=core --release --target=wasm32-wasi

tests: test-cli test-core
		
test-cli: cli
	cargo test --package=cli -- --nocapture

test-core:
	cargo wasi test --package=core -- --nocapture

fmt: fmt-github-asset-download fmt-wasm-sys fmt-core fmt-cli

fmt-github-asset-download:
	cargo fmt --package=github-asset-download -- --check
	cargo clippy --package=github-asset-download -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-wasm-sys:
	cargo fmt --package=ruvy-wasm-sys -- --check
	cargo clippy --package=ruvy-wasm-sys --target=wasm32-wasi -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-core:
	cargo fmt --package=core -- --check
	cargo clippy --package=core --target=wasm32-wasi --all-targets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-cli:
	cargo fmt --package=cli -- --check
	cargo clippy --package=cli --all-targets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

bench: core
	cargo bench --package=cli
