#!/usr/bin/env bash
set -euo pipefail

# Duplicate Type Detection Script
#
# Detects duplicate type definitions that should be in agent-agency-contracts.
# Fails CI if types that belong in contracts are defined elsewhere.
#
# Usage: ./scripts/check-duplicate-types.sh
#
# @author @darianrosebrook

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(dirname "$SCRIPT_DIR")"
V3_DIR="${REPO_ROOT}/iterations/v3"

cd "$REPO_ROOT"

echo "🔍 Checking for duplicate type definitions..."

# Types that MUST be in contracts, not elsewhere
FORBIDDEN_PATTERNS=(
    "struct TaskDescriptor"
    "enum TaskPriority"
    "enum ExecutionMode"
    "enum RiskTier"
    "struct BlastRadius"
    "struct ExecutionContext"
    "struct Milestone"
    "struct AcceptanceCriterion"
    "enum CouncilVerdict"
    "struct FinalDecision"
    "enum ContentType"
    "struct ProcessedContent"
)

# Exclude contracts crate and generated files
EXCLUDE_PATHS=(
    "agent-agency-contracts"
    "target"
    "node_modules"
    ".git"
)

VIOLATIONS=0

for pattern in "${FORBIDDEN_PATTERNS[@]}"; do
    echo "  Checking for: $pattern"
    
    # Search in Rust files, excluding contracts crate
    while IFS= read -r file; do
        # Skip if in excluded paths
        skip=false
        for excl in "${EXCLUDE_PATHS[@]}"; do
            if [[ "$file" == *"$excl"* ]]; then
                skip=true
                break
            fi
        done
        
        if [ "$skip" = false ]; then
            # Check if this is a type definition (not just a reference)
            if grep -q "^pub\|^struct\|^enum" "$file" | grep -q "$pattern"; then
                echo "    ❌ Found in: $file"
                echo "       This type should be in agent-agency-contracts, not duplicated here"
                ((VIOLATIONS++))
            fi
        fi
    done < <(grep -r -l "$pattern" "$V3_DIR" --include="*.rs" 2>/dev/null || true)
done

if [ $VIOLATIONS -gt 0 ]; then
    echo ""
    echo "❌ Found $VIOLATIONS duplicate type definition(s)"
    echo ""
    echo "All shared types must be defined in agent-agency-contracts."
    echo "Local crates should import from contracts, not define their own."
    echo ""
    echo "To fix:"
    echo "  1. Remove the duplicate definition"
    echo "  2. Add 'use agent_agency_contracts::types::prelude::*;'"
    echo "  3. Update imports to use contracts types"
    exit 1
fi

echo "✅ No duplicate type definitions found"

