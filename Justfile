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
    (cd browser-ide && rm -rf node_modules dist)
    echo "Cleaning tauri-app..."
    (cd tauri-app && rm -rf node_modules dist src-tauri/target)
    echo "Cleaning Rust..."
    (cd mml2vgm-rs && cargo clean)
    (cd mml2vgm-wasm && cargo clean)
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
    cd mml2vgm-wasm && wasm-pack build --release

# Build WASM module in debug mode (slower runtime, faster iterative compile)
wasm-build-debug:
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

# ============ DEPLOY COMMANDS ============

# Deploy browser IDE to Cloudflare Pages via Wrangler
# Setup: npm install -g wrangler && wrangler login
# Or set env vars: CLOUDFLARE_API_TOKEN, CLOUDFLARE_ACCOUNT_ID
deploy:
    #!/usr/bin/env bash
    set -e
    echo "Building WASM module..."
    just wasm-build
    echo "Building browser IDE..."
    (cd browser-ide && npm run build)
    echo "Deploying to Cloudflare Pages..."
    (cd browser-ide && npx wrangler pages deployment create dist --project-name mml2vgm-browser-ide)
    echo "Deployment complete!"

# ============ CROSS-PLATFORM BUILD COMMANDS ============

# Build Tauri for Linux
tauri-build-linux:
    (cd tauri-app && npm run tauri:build -- --release --target x86_64-unknown-linux-gnu)

# Build Tauri for Windows
tauri-build-windows:
    (cd tauri-app && npm run tauri:build -- --release --target x86_64-pc-windows-msvc)

# Build Tauri for macOS
tauri-build-macos:
    (cd tauri-app && npm run tauri:build -- --release --target x86_64-apple-darwin)

# Build Tauri for all platforms
tauri-build-all:
    #!/usr/bin/env bash
    set -e
    echo "Building Tauri for Linux..."
    just tauri-build-linux
    echo "Building Tauri for Windows..."
    just tauri-build-windows
    echo "Building Tauri for macOS..."
    just tauri-build-macos
    echo "Tauri builds complete in tauri-app/src-tauri/target/release/"

# Build Rust CLI for all platforms
rust-build-all:
    #!/usr/bin/env bash
    set -e
    echo "Building Rust CLI for Linux..."
    (cd mml2vgm-rs && cargo build --release --target x86_64-unknown-linux-gnu)
    echo "Building Rust CLI for Windows..."
    (cd mml2vgm-rs && cargo build --release --target x86_64-pc-windows-msvc)
    echo "Building Rust CLI for macOS..."
    (cd mml2vgm-rs && cargo build --release --target x86_64-apple-darwin)
    echo "Rust CLI builds complete in mml2vgm-rs/target/"

# Build everything for all platforms
build-all-release:
    #!/usr/bin/env bash
    set -e
    echo "Building Rust CLI..."
    just rust-build-release
    echo "Building WASM module..."
    just wasm-build-release
    echo "Building browser IDE..."
    just ide-build
    echo "Building Tauri desktop app..."
    just tauri-build-release
    echo "All builds complete!"
