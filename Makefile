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

clean: clean-wasi-sdk clean-cargo

clean-cargo:
	cargo clean

clean-wasi-sdk:
	rm -r crates/wasm-sys/wasi-sdk 2> /dev/null || true
