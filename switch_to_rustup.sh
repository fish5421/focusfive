#!/bin/bash

# Script to switch from Homebrew Rust to rustup

echo "Step 1: Uninstalling Homebrew Rust..."
brew uninstall rust rust-analyzer

echo "Step 2: Installing rustup..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

echo "Step 3: Sourcing cargo environment..."
source "$HOME/.cargo/env"

echo "Step 4: Installing rust-src component..."
rustup component add rust-src rust-analyzer

echo "Step 5: Verifying installation..."
which rustc
rustc --version
which rust-analyzer
rust-analyzer --version

echo "Done! Please restart your terminal or run: source ~/.cargo/env"
