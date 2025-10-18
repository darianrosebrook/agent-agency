#!/usr/bin/env bash
set -euo pipefail

# Setup sccache for Rust compiler caching
# This dramatically speeds up builds by caching compilation results

echo "Setting up sccache for Rust compiler caching..."

# Check if sccache is already installed
if command -v sccache >/dev/null 2>&1; then
    echo "sccache is already installed: $(sccache --version)"
else
    echo "Installing sccache..."
    
    # Install sccache based on platform
    case "$(uname -s)" in
        Darwin)
            if command -v brew >/dev/null 2>&1; then
                brew install sccache
            else
                echo "Please install Homebrew first: https://brew.sh/"
                exit 1
            fi
            ;;
        Linux)
            # Try to install via cargo first
            if command -v cargo >/dev/null 2>&1; then
                cargo install sccache
            else
                echo "Please install Rust/Cargo first: https://rustup.rs/"
                exit 1
            fi
            ;;
        *)
            echo "Unsupported platform. Please install sccache manually: https://github.com/mozilla/sccache"
            exit 1
            ;;
    esac
fi

# Configure sccache
SCCACHE_DIR="${SCCACHE_DIR:-$HOME/.cache/sccache}"
mkdir -p "$SCCACHE_DIR"

# Set up environment variables
cat > "$HOME/.sccache-env" << EOF
# sccache configuration
export RUSTC_WRAPPER="\$(command -v sccache)"
export SCCACHE_DIR="$SCCACHE_DIR"
export SCCACHE_CACHE_SIZE="50G"
export SCCACHE_NAMESPACE="agent-agency-v3"
export SCCACHE_LOG_LEVEL="info"
EOF

echo "sccache configuration written to $HOME/.sccache-env"
echo ""
echo "To activate sccache, run:"
echo "  source $HOME/.sccache-env"
echo ""
echo "Or add this to your shell profile:"
echo "  source $HOME/.sccache-env"

# Test sccache
if command -v sccache >/dev/null 2>&1; then
    echo ""
    echo "Testing sccache..."
    sccache --show-stats
    echo ""
    echo "sccache setup complete!"
else
    echo "Error: sccache installation failed"
    exit 1
fi
