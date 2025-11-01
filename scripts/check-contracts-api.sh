#!/usr/bin/env bash
set -euo pipefail

# Public API Regression Check for agent-agency-contracts
#
# Ensures the contracts crate maintains API stability by detecting:
# - Removed public items
# - Changed public API signatures
# - Breaking changes to public types
#
# Usage: ./scripts/check-contracts-api.sh
#
# Requires: cargo-public-api, cargo-semver-checks
#
# @author @darianrosebrook

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
CONTRACTS_PATH="${REPO_ROOT}/iterations/v3/agent-agency-contracts"

cd "$REPO_ROOT"

echo "🔍 Checking public API stability for agent-agency-contracts..."

# Check if cargo-public-api is installed
if ! command -v cargo-public-api &> /dev/null; then
    echo "⚠️  cargo-public-api not found. Installing..."
    cargo install cargo-public-api --quiet || {
        echo "❌ Failed to install cargo-public-api"
        echo "   Install manually: cargo install cargo-public-api"
        exit 1
    }
fi

# Check if cargo-semver-checks is installed
if ! command -v cargo-semver-checks &> /dev/null; then
    echo "⚠️  cargo-semver-checks not found. Installing..."
    cargo install cargo-semver-checks --quiet || {
        echo "❌ Failed to install cargo-semver-checks"
        echo "   Install manually: cargo install cargo-semver-checks"
        exit 1
    }
fi

cd "$CONTRACTS_PATH"

# Generate current public API
echo "📊 Generating current public API..."
cargo public-api \
    --manifest-path Cargo.toml \
    --output-format stdout \
    > target/public-api-current.txt 2>&1 || {
    echo "⚠️  cargo-public-api generated warnings (this is usually OK)"
}

# Store snapshot for comparison (first run only)
if [ ! -f "target/public-api-snapshot.txt" ]; then
    echo "📸 Creating initial API snapshot..."
    cp target/public-api-current.txt target/public-api-snapshot.txt
    echo "✅ Initial snapshot created at target/public-api-snapshot.txt"
    echo "   Review and commit this snapshot as the baseline."
    exit 0
fi

# Compare with snapshot
echo "🔬 Comparing with API snapshot..."
if ! diff -u target/public-api-snapshot.txt target/public-api-current.txt > target/public-api-diff.txt; then
    echo "❌ Public API changes detected!"
    echo ""
    echo "Changes:"
    cat target/public-api-diff.txt
    echo ""
    echo "If these changes are intentional:"
    echo "  1. Review the changes above"
    echo "  2. Update snapshot: cp target/public-api-current.txt target/public-api-snapshot.txt"
    echo "  3. Commit the updated snapshot"
    echo ""
    echo "If these changes are accidental:"
    echo "  - Revert your changes"
    echo "  - Or mark items as #[doc(hidden)] if they shouldn't be public"
    exit 1
fi

echo "✅ Public API matches snapshot"

# Run semver checks (requires baseline version)
if git rev-parse --verify HEAD~1 &> /dev/null; then
    echo "🔬 Running semver compatibility checks..."
    cargo semver-checks check-release \
        --manifest-path Cargo.toml \
        --baseline-rev HEAD~1 || {
        echo "⚠️  Semver checks found potential issues"
        echo "   Review the output above for breaking changes"
    }
fi

echo "✅ Public API checks complete"

