#!/usr/bin/env bash
set -euo pipefail

# Dependency Gate Enforcement Script
# Ensures the Cargo dependency graph remains acyclic and follows architectural rules
#
# Usage: ./scripts/check-deps.sh
#
# This script:
# 1. Generates cargo metadata JSON
# 2. Calls Node.js script to validate dependency rules
# 3. Fails CI if forbidden edges are detected

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"

cd "$REPO_ROOT"

echo "🔍 Checking dependency graph for forbidden edges..."

# Generate metadata if not provided
METADATA_FILE="${1:-target/metadata.json}"
if [[ ! -f "$METADATA_FILE" ]]; then
    echo "📊 Generating cargo metadata..."
    cargo metadata --format-version=1 > "$METADATA_FILE"
fi

# Run dependency validation
node scripts/check-deps.mjs "$METADATA_FILE" <<'RULES'
# Core architectural rules - forbid upward dependencies from orchestration
FORBID: agent-orchestration -> agent-research
FORBID: agent-orchestration -> agent-workers
FORBID: agent-orchestration -> agent-data-processing
FORBID: agent-orchestration -> agent-memory
FORBID: agent-orchestration -> agent-model-management

# Contracts should not depend on application code
FORBID: agent-agency-contracts -> agent-*,system-*,apps-*,data-*

# Allow all downward dependencies to contracts (but don't require)
ALLOW: * -> agent-agency-contracts
RULES

echo "✅ All dependency rules satisfied!"
