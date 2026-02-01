# TopStats Rust SDK - Development Commands

# Default recipe: show available commands
default:
    @just --list

# Run all checks (format, lint, test)
check: fmt-check lint test

# Run tests for async mode (default)
test:
    cargo nextest run --lib

# Run tests for blocking mode
test-blocking:
    cargo nextest run --no-default-features --features "blocking,ureq-client" --lib

# Run tests for both modes
test-all: test test-blocking

# Run clippy for async mode
lint:
    cargo clippy -- -D warnings

# Run clippy for blocking mode
lint-blocking:
    cargo clippy --no-default-features --features "blocking,ureq-client" -- -D warnings

# Run clippy for both modes
lint-all: lint lint-blocking

# Check formatting
fmt-check:
    cargo fmt --check

# Format code
fmt:
    cargo fmt

# Build async mode
build:
    cargo build

# Build blocking mode
build-blocking:
    cargo build --no-default-features --features "blocking,ureq-client"

# Build both modes
build-all: build build-blocking

# Build release (async)
build-release:
    cargo build --release

# Run async example (requires TOPSTATS_TOKEN env var)
example:
    cargo run --manifest-path examples/async/Cargo.toml

# Run blocking example (requires TOPSTATS_TOKEN env var)
example-blocking:
    cargo run --manifest-path examples/blocking/Cargo.toml

# Generate documentation
doc:
    cargo doc --no-deps --open

# Clean build artifacts
clean:
    cargo clean

# Check compilation without building
check-async:
    cargo check

# Check blocking mode compilation
check-blocking:
    cargo check --no-default-features --features "blocking,ureq-client"

# Full CI check (what CI would run)
ci: fmt-check lint-all test-all
    @echo "All CI checks passed!"

# Run CI workflow locally with act
act *args:
    act {{args}}

# List available CI jobs
act-list:
    act -l
