# Justfile for mml2vgm project
# Run with: just <command>
# List commands: just --list

# ============ PROJECT-WIDE COMMANDS ============

# Show all available commands
list:
    just --list

# Clean all build artifacts
clean:
    #!/usr/bin/env bash
    set -e
    echo "Cleaning browser-ide..."
    cd browser-ide && rm -rf node_modules dist
    echo "Cleaning tauri-app..."
    cd tauri-app && rm -rf node_modules dist src-tauri/target
    echo "Cleaning Rust..."
    cd mml2vgm-rs && cargo clean
    cd mml2vgm-wasm && cargo clean
    echo "Cleaning test outputs..."
    rm -rf mml2vgm-rs/target mml2vgm-wasm/target
    echo "Done!"

# Install all dependencies
install:
    #!/usr/bin/env bash
    set -e
    echo "Installing browser-ide dependencies..."
    (cd browser-ide && npm install)
    echo "Installing tauri-app dependencies..."
    (cd tauri-app && npm install)
    echo "Done!"

# ============ BROWSER IDE COMMANDS ============

# Start browser IDE dev server
ide-dev:
    cd browser-ide && npm run dev

# Build browser IDE for production
ide-build:
    cd browser-ide && npm run build

# Run browser IDE tests
ide-test:
    cd browser-ide && npm run test

# Lint browser IDE
ide-lint:
    cd browser-ide && npm run lint

# ============ RUST CLI COMMANDS ============

# Build Rust CLI
rust-build:
    cd mml2vgm-rs && cargo build

# Build Rust CLI in release mode
rust-build-release:
    cd mml2vgm-rs && cargo build --release

# Run Rust tests
rust-test:
    cd mml2vgm-rs && cargo test

# Run Rust tests with coverage
rust-test-coverage:
    cd mml2vgm-rs && cargo tarpaulin --all-features

# Run Clippy linter on Rust code
rust-lint:
    cd mml2vgm-rs && cargo clippy --all-features -- -D warnings

# Build Rust docs
rust-docs:
    cd mml2vgm-rs && cargo doc --open

# Run Rust CLI help
rust-help:
    cd mml2vgm-rs && cargo run -- --help

# ============ WASM COMMANDS ============

# Build WASM module
wasm-build:
    cd mml2vgm-wasm && wasm-pack build

# Build WASM in release mode
wasm-build-release:
    cd mml2vgm-wasm && wasm-pack build --release

# ============ TAURI DESKTOP COMMANDS ============

# Start Tauri dev
tauri-dev:
    cd tauri-app && npm run tauri:dev

# Build Tauri app
tauri-build:
    cd tauri-app && npm run tauri:build

# Build Tauri app for release
tauri-build-release:
    cd tauri-app && npm run tauri:build -- --release

# ============ COMBINED COMMANDS ============

# Build everything (Rust + WASM + IDE + Tauri)
build-all:
    #!/usr/bin/env bash
    set -e
    just rust-build-release
    just wasm-build-release
    just ide-build
    just tauri-build

# Dev mode: Run IDE + Tauri watch (run in separate terminals)
dev:
    #!/usr/bin/env bash
    echo "Run in terminal 1: just ide-dev"
    echo "Run in terminal 2: just tauri-dev"

# Full build and test
ci:
    #!/usr/bin/env bash
    set -e
    just rust-lint
    just rust-test
    just ide-lint
    just ide-test
    just rust-build-release
    just wasm-build-release
    echo "All checks passed!"

# ============ UTILITY COMMANDS ============

# Format all code
format:
    #!/usr/bin/env bash
    set -e
    echo "Formatting Rust..."
    cargo fmt --manifest-path mml2vgm-rs/Cargo.toml
    cargo fmt --manifest-path mml2vgm-wasm/Cargo.toml
    echo "Formatting TypeScript..."
    cd browser-ide && npx prettier --write src/
    cd tauri-app && npx prettier --write src/

# Check for outdated dependencies
outdated:
    #!/usr/bin/env bash
    echo "Checking Rust dependencies..."
    cd mml2vgm-rs && cargo outdated --workspace
    echo "Checking npm dependencies..."
    cd browser-ide && npm outdated
    cd tauri-app && npm outdated
