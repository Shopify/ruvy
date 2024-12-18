.DEFAULT_GOAL := cli

cli: core
	cargo build --package=cli

core:
	cargo build --package=core --release --target=wasm32-wasip1

tests: test-cli test-core
		
test-cli: cli
	cargo test --package=cli -- --nocapture

test-core:
	cargo test --package=core --target=wasm32-wasip1 -- --nocapture

fmt: fmt-ruby-wasm-assets fmt-wasm-sys fmt-core fmt-cli

fmt-ruby-wasm-assets:
	cargo fmt --package=ruby-wasm-assets -- --check
	cargo clippy --package=ruby-wasm-assets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-wasm-sys:
	cargo fmt --package=ruvy-wasm-sys -- --check
	cargo clippy --package=ruvy-wasm-sys --target=wasm32-wasip1 -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-core:
	cargo fmt --package=core -- --check
	cargo clippy --package=core --target=wasm32-wasip1 --all-targets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

fmt-cli:
	cargo fmt --package=cli -- --check
	cargo clippy --package=cli --all-targets -- -D clippy::correctness -D clippy::perf -D clippy::suspicious

bench: core
	cargo bench --package=cli
