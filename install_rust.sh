#!/bin/bash
set -e

echo "Installing Rust toolchain..."
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y

echo "Sourcing Rust environment..."
source $HOME/.cargo/env

echo "Verifying Rust installation..."
rustc --version
cargo --version

echo "Rust installation complete!"
