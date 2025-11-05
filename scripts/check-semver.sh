#!/bin/bash
set -euo pipefail

# Install cargo-semver-checks if not present
if ! command -v cargo-semver-checks &> /dev/null; then
    echo "Installing cargo-semver-checks..."
    cargo install cargo-semver-checks
fi

# Check API compatibility
echo "Checking API compatibility for contracts..."
if [ -d ".caws/baseline-contracts" ]; then
    cargo semver-checks check-release \
        --manifest-path iterations/v3/agent-agency-contracts/Cargo.toml \
        --baseline-root .caws/baseline-contracts || {
        echo "ERROR: API compatibility check failed"
        echo "If this is intentional, update CHANGELOG.md and .caws/baseline-contracts"
        exit 1
    }
else
    echo "No baseline found - skipping semver check (first run)"
fi

