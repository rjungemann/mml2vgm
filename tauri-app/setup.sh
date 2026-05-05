#!/bin/bash

# mml2vgm Tauri Desktop Setup Script
# This script sets up the Tauri app with all dependencies

set -e

echo "Setting up mml2vgm Tauri Desktop App..."
echo ""

# Check Node.js version
NODE_VERSION=$(node --version)
echo "Node.js version: $NODE_VERSION"

if [[ ! "$NODE_VERSION" =~ ^v18\. ]] && [[ ! "$NODE_VERSION" =~ ^v20\. ]]; then
    echo "Error: Node.js 18+ is required"
    echo "Please install Node.js 18 or later from https://nodejs.org/"
    exit 1
fi

# Check if we're in the tauri-app directory
if [ ! -f "package.json" ]; then
    echo "Error: Please run this script from the tauri-app directory"
    exit 1
fi

echo "Installing npm dependencies..."
npm install

echo ""
echo "Checking Rust installation..."
if ! command -v rustc &> /dev/null; then
    echo "Rust is not installed. Installing Rust 1.70+..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
else
    RUST_VERSION=$(rustc --version)
    echo "Rust version: $RUST_VERSION"
fi

echo ""
echo "Checking Tauri CLI..."
if ! command -v tauri &> /dev/null; then
    echo "Tauri CLI not found. Installing..."
    npm install -g @tauri-apps/cli
else
    TAURI_VERSION=$(tauri --version)
    echo "Tauri CLI version: $TAURI_VERSION"
fi

echo ""
echo "✅ Setup complete!"
echo ""
echo "To start development:"
echo "  npm run tauri:dev"
echo ""
echo "To build for production:"
echo "  npm run tauri:build"
echo ""
