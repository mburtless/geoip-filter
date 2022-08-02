.PHONY: lint
lint:
	@cargo clippy

.PHONY: build
build:
	@cargo build --target wasm32-unknown-unknown --release

