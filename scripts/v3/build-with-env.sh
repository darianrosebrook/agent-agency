#!/usr/bin/env bash
# Build wrapper that ensures M1 Max environment is configured
# Ensures CoreML and torch-sys compile correctly
# @author: @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_DIR="$ROOT_DIR/iterations/v3"

# Source the build environment if it exists
if [ -f "$ROOT_DIR/.env.build" ]; then
    source "$ROOT_DIR/.env.build"
fi

# Ensure critical environment variables are set
export LIBTORCH="${LIBTORCH:-$ROOT_DIR/libtorch-cpu}"
export LIBTORCH_CXX11_ABI="${LIBTORCH_CXX11_ABI:-0}"
export CMAKE_PREFIX_PATH="${CMAKE_PREFIX_PATH:-$LIBTORCH}"
export DYLD_LIBRARY_PATH="${DYLD_LIBRARY_PATH:-$LIBTORCH/lib:}"

# C++17 flags for torch-sys
export CXXFLAGS="${CXXFLAGS:--std=c++17 -stdlib=libc++}"
export CXX="${CXX:-clang++}"
export CC="${CC:-clang}"

# macOS deployment target
export MACOSX_DEPLOYMENT_TARGET="${MACOSX_DEPLOYMENT_TARGET:-11.0}"

# Change to v3 directory
cd "$V3_DIR"

# Run cargo command with all arguments
exec cargo "$@"

