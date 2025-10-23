#!/usr/bin/env bash
# Build environment setup script
# @darianrosebrook
#
# This script sets up the optimal build environment for Rust development
# with multiple agents working concurrently.

set -euo pipefail

echo " Setting up optimized Rust build environment..."

# Check if we're on macOS
if [[ "$OSTYPE" == "darwin"* ]]; then
    echo " Detected macOS - setting up Apple Silicon optimizations"
    
    # Install Homebrew if not present
    if ! command -v brew >/dev/null 2>&1; then
        echo " Installing Homebrew..."
        /bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"
    fi
    
    # Install LLVM for fast linker
    echo " Installing LLVM for fast linking..."
    brew install llvm || echo "LLVM may already be installed"
    
    # Install mold (alternative fast linker)
    echo " Installing mold (alternative fast linker)..."
    brew install mold || echo "Mold may already be installed"
fi

# Install Rust build optimization tools
echo " Installing Rust build optimization tools..."

# Install sccache for compiler caching
if ! command -v sccache >/dev/null 2>&1; then
    echo "   Installing sccache..."
    cargo install sccache
else
    echo "    sccache already installed"
fi

# Install cargo-nextest for faster testing
if ! command -v cargo-nextest >/dev/null 2>&1; then
    echo "   Installing cargo-nextest..."
    cargo install cargo-nextest
else
    echo "    cargo-nextest already installed"
fi

# Install cargo-machete for dependency cleanup
if ! command -v cargo-machete >/dev/null 2>&1; then
    echo "   Installing cargo-machete..."
    cargo install cargo-machete
else
    echo "    cargo-machete already installed"
fi

# Create cache directories
echo " Setting up cache directories..."
mkdir -p ~/.cache/sccache
mkdir -p ~/.cargo/registry/cache

# Set up environment variables for current session
echo " Setting up environment variables..."
export RUSTC_WRAPPER="$(command -v sccache 2>/dev/null || echo "")"
export SCCACHE_DIR="$HOME/.cache/sccache"
export SCCACHE_CACHE_SIZE="50G"
export SCCACHE_NAMESPACE="agent-agency"

# Create a .env file for the project
cat > .env.build << EOF
# Rust build optimization environment variables
# @darianrosebrook

# Compiler cache
export RUSTC_WRAPPER="$(command -v sccache 2>/dev/null || echo "")"
export SCCACHE_DIR="$HOME/.cache/sccache"
export SCCACHE_CACHE_SIZE="50G"
export SCCACHE_NAMESPACE="agent-agency"

# Fast linker configuration
export RUSTFLAGS="-Clink-arg=-fuse-ld=/usr/local/bin/ld64.lld"

# Parallel compilation
export CARGO_BUILD_JOBS=0

# Network optimization
export CARGO_NET_RETRY=2
export CARGO_NET_GIT_FETCH_WITH_CLI=true

# Registry optimization
export CARGO_REGISTRIES_CRATES_IO_PROTOCOL=sparse
EOF

echo " Build environment setup complete!"
echo ""
echo " Next steps:"
echo "   1. Source the environment: source .env.build"
echo "   2. Test the build wrapper: ./scripts/build-wrapper.sh check --workspace VERBOSE=true"
echo "   3. Run optimized builds: make build-dev"
echo ""
echo " Available tools:"
echo "   - sccache: Compiler caching"
echo "   - cargo-nextest: Fast parallel testing"
echo "   - cargo-machete: Dependency cleanup"
echo "   - ld64.lld: Fast linker for macOS"
echo ""
echo " Tips:"
echo "   - Each agent should use a unique AGENT_ID environment variable"
echo "   - Use VERBOSE=true to see detailed build information"
echo "   - The build wrapper automatically handles target directory isolation"
