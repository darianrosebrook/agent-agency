#!/bin/bash
# Comprehensive Rust linting script for agent-agency project
# @darianrosebrook

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$PROJECT_ROOT"

echo -e "${BLUE} Running comprehensive Rust linting for agent-agency...${NC}"
echo ""

# Function to run command and show status
run_check() {
    local name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Running: $name${NC}"
    if eval "$command"; then
        echo -e "${GREEN} $name passed${NC}"
    else
        echo -e "${RED} $name failed${NC}"
        return 1
    fi
    echo ""
}

# Track overall success
OVERALL_SUCCESS=true

# 1. Basic compilation check
if ! run_check "Basic compilation check" "cargo check --workspace"; then
    OVERALL_SUCCESS=false
fi

# 2. Clippy linting
if ! run_check "Clippy linting" "cargo clippy --workspace --all-targets --all-features"; then
    OVERALL_SUCCESS=false
fi

# 3. Formatting check
if ! run_check "Formatting check" "cargo fmt --all -- --check"; then
    echo -e "${YELLOW} Run 'cargo fmt --all' to fix formatting issues${NC}"
    OVERALL_SUCCESS=false
fi

# 4. Run tests
if ! run_check "Unit tests" "cargo test --workspace"; then
    OVERALL_SUCCESS=false
fi

# 5. Check for security vulnerabilities
if command -v cargo-audit &> /dev/null; then
    if ! run_check "Security audit" "cargo audit"; then
        OVERALL_SUCCESS=false
    fi
else
    echo -e "${YELLOW}⚠️  cargo-audit not installed. Install with: cargo install cargo-audit${NC}"
fi

# 6. Check for outdated dependencies
if command -v cargo-outdated &> /dev/null; then
    if ! run_check "Dependency check" "cargo outdated --workspace"; then
        OVERALL_SUCCESS=false
    fi
else
    echo -e "${YELLOW}⚠️  cargo-outdated not installed. Install with: cargo install cargo-outdated${NC}"
fi

# Summary
echo "=========================================="
if [ "$OVERALL_SUCCESS" = true ]; then
    echo -e "${GREEN} All linting checks passed!${NC}"
    exit 0
else
    echo -e "${RED} Some linting checks failed. Please fix the issues above.${NC}"
    exit 1
fi
