#!/usr/bin/env bash
# Node/TypeScript Environment Setup
# @darianrosebrook

set -euo pipefail

setup_node() {
    if ! command -v node >/dev/null 2>&1; then
        echo "Node not found, skipping setup"
        return 0
    fi

    # Cache directories
    export NODE_CACHE_DIR=".cache/${BUILD_NAMESPACE:-unknown}"
    export TURBO_CACHE_DIR="${TURBO_CACHE_DIR:-$HOME/.cache/turbo}"
    export NX_CACHE_DIR="${NX_CACHE_DIR:-$HOME/.cache/nx}"

    # pnpm configuration (if available)
    if command -v pnpm >/dev/null 2>&1; then
        export PNPM_STORE_DIR="${PNPM_STORE_DIR:-$HOME/.cache/pnpm}"
        export PNPM_HOME="${PNPM_HOME:-$HOME/.cache/pnpm}"
    fi

    # Turbo configuration (if available)
    if command -v turbo >/dev/null 2>&1; then
        export TURBO_FORCE="false"
        export TURBO_REMOTE_CACHE_DISABLED="false"
    fi

    # Nx configuration (if available)
    if command -v nx >/dev/null 2>&1; then
        # Nx uses its own cache configuration
        export NX_CACHE_DIR="${NX_CACHE_DIR}"
    fi

    # Ensure cache directories exist
    mkdir -p "$NODE_CACHE_DIR" "$TURBO_CACHE_DIR" "$NX_CACHE_DIR"
}

# Run setup if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    setup_node
fi
