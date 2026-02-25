#!/usr/bin/env bash
# Quick setup script for WSL environment

echo "Setting up build environment for WSL..."

# Install Rust
if ! command -v cargo &> /dev/null; then
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source "$HOME/.cargo/env"
    echo "✓ Rust installed"
else
    echo "✓ Rust already installed"
fi

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "⚠ Docker not found. Please install Docker Desktop for Windows with WSL2 integration"
    echo "  Download: https://www.docker.com/products/docker-desktop"
    exit 1
else
    echo "✓ Docker found"
fi

# Check if Docker daemon is running
if ! docker info &> /dev/null; then
    echo "⚠ Docker daemon not running. Please start Docker Desktop"
    exit 1
else
    echo "✓ Docker daemon running"
fi

echo ""
echo "Environment ready! You can now run:"
echo "  ./scripts/airgapped/build-single-binary.sh"
