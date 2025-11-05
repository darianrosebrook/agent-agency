#!/bin/bash
set -euo pipefail

# Install guppy if not present
if ! command -v cargo-guppy &> /dev/null; then
    echo "Installing cargo-guppy..."
    cargo install cargo-guppy
fi

# Check for cycles
echo "Checking for dependency cycles..."
if cargo guppy cycles --workspace-root .; then
    echo "ERROR: Dependency cycle detected!"
    exit 1
fi

echo "No cycles detected"

