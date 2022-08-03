.PHONY: lint
lint:
	@cargo clippy

.PHONY: build
build:
	@echo "Building geoip-filter and singleton-service"
	@make geoip-filter
	@make singleton-service

.PHONY: geoip-filter
geoip-filter:
	@cargo build --package geoip-filter --target wasm32-unknown-unknown --release

.PHONY: singleton-service
geoip-filter:
	@cargo build --package singleton-service --target wasm32-unknown-unknown --release
