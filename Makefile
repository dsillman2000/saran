# Saran CLI Wrapper Framework - Minimal Makefile
.PHONY: help build test lint fmt clean

help:
	@echo "Saran CLI Wrapper Framework"
	@echo ""
	@echo "Targets:"
	@echo "  build    - Build all crates"
	@echo "  test     - Run all tests (single-threaded for consistent output)"
	@echo "  lint     - Run clippy"
	@echo "  fmt      - Format code"
	@echo "  clean    - Clean build artifacts"

build:
	cargo build

# Tests run single-threaded (--test-threads=1) to ensure test tag output is not interleaved
test:
	cargo test --workspace -- --nocapture --test-threads=1

lint:
	cargo clippy --all -- -D warnings

fmt:
	cargo fmt --all

clean:
	cargo clean