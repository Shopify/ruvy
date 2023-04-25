.DEFAULT_GOAL := core

download-wasi-sdk:
	./install-wasi-sdk.sh

core:
		cd crates/core \
				&& cargo build --release --target=wasm32-wasi\
				&& cd - \
				&& wizer --allow-wasi target/wasm32-wasi/release/core.wasm --dir . -o crates/cli/engine.wasm
		
test-ruvy:
		cargo wasi test --package=core -- --nocapture

clean: clean-wasi-sdk clean-cargo

clean-cargo:
	cargo clean

clean-wasi-sdk:
	rm -r crates/quickjs-wasm-sys/wasi-sdk 2> /dev/null || true
