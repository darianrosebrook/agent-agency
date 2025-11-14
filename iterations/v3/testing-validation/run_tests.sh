#!/bin/bash
# Test runner script with Swift library path configuration
# @author @darianrosebrook

set -e

# Set Swift library path
export DYLD_FALLBACK_LIBRARY_PATH="/usr/lib/swift:${DYLD_FALLBACK_LIBRARY_PATH:-}"

# Set database URL if not provided
export DATABASE_URL="${DATABASE_URL:-postgresql://test_user:test_password@localhost:5433/test_db}"

# Set API base URL if not provided
export API_BASE_URL="${API_BASE_URL:-http://localhost:8080}"

# Run tests
cd "$(dirname "$0")/.."
cargo test --lib "$@" --no-default-features

