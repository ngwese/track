# Track developer ergonomics (ADR-0001 phase 5)
.PHONY: build test run clean ci

build:
	cargo build -p track-cli --target wasm32-wasip2
	cargo build -p track-host

test:
	cargo test -p track-host
	cargo test -p track-host --test integration
	cargo test -p track-types

run: build
	cargo run -p track-host

clean:
	cargo clean

# Mirrors the GitHub Actions CI job locally
ci: build test
	cargo run -p track-host -- version
