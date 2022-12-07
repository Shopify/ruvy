.DEFAULT_GOAL := ruvy

download-wasi-sdk:
	./install-wasi-sdk.sh

ruvy:
		cd crates/ruvy \
				&& cargo build --release --target=wasm32-wasi\
				&& cd - \
				&& wizer --allow-wasi target/wasm32-wasi/release/ruvy.wasm --dir . -o ruvy.wizened.wasm

clean: clean-wasi-sdk clean-cargo

clean-cargo: cargo clean
