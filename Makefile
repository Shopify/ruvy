.DEFAULT_GOAL := cli

download-wasi-sdk:
	./install-wasi-sdk.sh

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
	cargo clippy --package=ruvy-wasm-sys --target=wasm32-wasi

fmt-core:
	cargo fmt --package=core -- --check
	cargo clippy --package=core --target=wasm32-wasi --all-targets -- -D warnings

fmt-cli:
	cargo fmt --package=cli -- --check
	cargo clippy --package=cli --all-targets -- -D warnings

clean: clean-wasi-sdk clean-cargo

clean-cargo:
	cargo clean

clean-wasi-sdk:
	rm -r crates/wasm-sys/wasi-sdk 2> /dev/null || true
