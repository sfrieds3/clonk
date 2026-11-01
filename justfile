# List available commands.
default:
    @just --list

# Launch CLONK.
run:
    cargo run

# Build the desktop app.
build:
    cargo build

# Run the simulation and parser tests.
test:
    cargo test

# Format Rust source.
fmt:
    cargo fmt

# Run strict Clippy checks.
lint:
    cargo clippy --all-targets -- -D warnings

# Run all validation without changing source files.
check:
    cargo fmt --check
    cargo test
    cargo clippy --all-targets -- -D warnings
