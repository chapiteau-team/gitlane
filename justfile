# List available recipes
default:
    @just --list

# Build the workspace
build:
    cargo build --workspace

# Type-check the workspace
check:
    cargo check --workspace

# Format all code
fmt:
    cargo fmt --all

# Verify formatting without modifying files
fmt-check:
    cargo fmt --all -- --check

# Run clippy with warnings as errors
lint:
    cargo clippy --workspace --all-targets --all-features -- -D warnings

# Run cargo-deny (advisories, bans, licenses, sources)
deny:
    cargo deny check --all-features

# Run all workspace tests
test:
    cargo test --workspace

# Run the gitlane CLI; canonical form: `just run -- <args>`
run *args:
    cargo run -p gitlane-cli -- {{ args }}

# Run everything CI runs locally
ci: fmt-check lint deny test
