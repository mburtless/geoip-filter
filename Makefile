.PHONY: lint
lint:
	@cargo clippy

.PHONY: build
build:
	@echo "Building geoip-filter and singleton-service"
	@make geoip-filter
	@make singleton-service

.PHONY: run
run:
ifneq ("$(wildcard ./GeoLite2-Country.mmdb)","")
	@docker-compose up -d
else
	@echo "Waring: GeoLite2-Country.mmdb missing, this must be present in root dir to run locally!"
endif

.PHONY: geoip-filter
geoip-filter:
	@cargo build --package geoip-filter --target wasm32-unknown-unknown --release

.PHONY: singleton-service
singleton-service:
	@cargo build --package singleton-service --target wasm32-unknown-unknown --release
