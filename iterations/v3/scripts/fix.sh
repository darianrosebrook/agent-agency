#!/bin/bash
# Auto-fix script for common Rust issues
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

echo -e "${BLUE}🔧 Running auto-fix for common Rust issues...${NC}"
echo ""

# Function to run command and show status
run_fix() {
    local name="$1"
    local command="$2"
    
    echo -e "${YELLOW}Running: $name${NC}"
    if eval "$command"; then
        echo -e "${GREEN}✅ $name completed${NC}"
    else
        echo -e "${RED}❌ $name failed${NC}"
        return 1
    fi
    echo ""
}

# 1. Auto-fix with cargo fix
echo -e "${BLUE}Auto-fixing issues that can be automatically resolved...${NC}"
if ! run_fix "Auto-fix with cargo fix" "cargo fix --workspace --allow-dirty --allow-staged"; then
    echo -e "${YELLOW}⚠️  Some issues could not be auto-fixed${NC}"
fi

# 2. Format code
if ! run_fix "Format code" "cargo fmt --all"; then
    echo -e "${YELLOW}⚠️  Formatting failed${NC}"
fi

# 3. Clean unused imports
if ! run_fix "Clean unused imports" "cargo +nightly fix --workspace --allow-dirty --allow-staged --clippy"; then
    echo -e "${YELLOW}⚠️  Could not clean unused imports (nightly toolchain required)${NC}"
fi

echo -e "${GREEN}🎉 Auto-fix completed!${NC}"
echo -e "${YELLOW}💡 Run './scripts/lint.sh' to check remaining issues${NC}"
