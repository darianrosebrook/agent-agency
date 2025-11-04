#!/usr/bin/env bash
# Cargo Concurrent Build Wrapper
# @darianrosebrook
#
# Wrapper that sets a unique target directory per process to avoid lockfile contention
# Usage: ./scripts/cargo-concurrent.sh build --workspace

set -euo pipefail

# Generate unique identifier for this build process
# Uses: PID + timestamp + random to ensure uniqueness
BUILD_ID="${BUILD_ID:-build-$$-$(date +%s)-$(shuf -i 1000-9999 -n 1)}"

# Set unique target directory
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-.target/${BUILD_ID}}"

# Ensure target directory exists
mkdir -p "$CARGO_TARGET_DIR"

# Use all CPU cores for parallel builds (set in .cargo/config.toml as jobs = 0)
# This allows Cargo to build multiple crates in parallel within this process

# Execute cargo with all arguments
if [[ "${VERBOSE:-false}" == "true" ]]; then
    echo "Running cargo with BUILD_ID=${BUILD_ID}"
    echo "Target directory: ${CARGO_TARGET_DIR}"
    echo "Command: cargo $*"
fi

exec cargo "$@"




