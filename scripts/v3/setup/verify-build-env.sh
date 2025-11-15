#!/usr/bin/env bash
# Verify M1 Max build environment is correctly configured
# Run this before building to ensure all requirements are met
# @author: @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

ERRORS=0

check_env_var() {
    local var="$1"
    local value="${!var:-}"
    
    if [ -z "$value" ]; then
        echo -e "${RED}❌ $var is not set${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    else
        echo -e "${GREEN}✅ $var=$value${NC}"
        return 0
    fi
}

echo -e "Verifying M1 Max build environment...\n"

# Check critical environment variables
echo "Environment Variables:"
check_env_var "LIBTORCH"
check_env_var "LIBTORCH_CXX11_ABI"
check_env_var "CXXFLAGS"
check_env_var "CXX"
check_env_var "CC"

# Verify LIBTORCH points to libtorch-cpu
if [ -n "${LIBTORCH:-}" ]; then
    if [[ "$LIBTORCH" != *"libtorch-cpu"* ]]; then
        echo -e "${YELLOW}⚠️  LIBTORCH doesn't point to libtorch-cpu: $LIBTORCH${NC}"
        echo -e "${YELLOW}   Should be: $ROOT_DIR/libtorch-cpu${NC}"
        ERRORS=$((ERRORS + 1))
    fi
fi

# Verify CXXFLAGS includes C++17
if [ -n "${CXXFLAGS:-}" ]; then
    if [[ "$CXXFLAGS" != *"c++17"* ]]; then
        echo -e "${YELLOW}⚠️  CXXFLAGS doesn't include C++17: $CXXFLAGS${NC}"
        echo -e "${YELLOW}   Should include: -std=c++17${NC}"
        ERRORS=$((ERRORS + 1))
    fi
fi

# Check Python architecture
echo ""
echo "Python Configuration:"
PYTHON_ARCH=$(python3 -c "import platform; print(platform.machine())" 2>/dev/null || echo "unknown")
if [ "$PYTHON_ARCH" = "arm64" ]; then
    echo -e "${GREEN}✅ Python is ARM64: $(which python3)${NC}"
else
    echo -e "${RED}❌ Python is not ARM64 (detected: $PYTHON_ARCH)${NC}"
    echo -e "${YELLOW}   Install ARM64 Python: brew install python@3.13${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Check libtorch-cpu exists
echo ""
echo "LibTorch Installation:"
if [ -d "$ROOT_DIR/libtorch-cpu" ]; then
    echo -e "${GREEN}✅ libtorch-cpu directory exists${NC}"
    if [ -f "$ROOT_DIR/libtorch-cpu/lib/libtorch.dylib" ]; then
        echo -e "${GREEN}✅ libtorch.dylib found${NC}"
    else
        echo -e "${RED}❌ libtorch.dylib not found${NC}"
        ERRORS=$((ERRORS + 1))
    fi
else
    echo -e "${RED}❌ libtorch-cpu directory not found${NC}"
    ERRORS=$((ERRORS + 1))
fi

# Summary
echo ""
if [ $ERRORS -eq 0 ]; then
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${GREEN}  Environment verification passed!${NC}"
    echo -e "${GREEN}═══════════════════════════════════════════════════════════${NC}"
    exit 0
else
    echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
    echo -e "${RED}  Environment verification failed ($ERRORS errors)${NC}"
    echo -e "${RED}═══════════════════════════════════════════════════════════${NC}"
    echo ""
    echo "Run the setup script to fix:"
    echo "  bash scripts/v3/setup/setup-m1-build-env.sh"
    echo ""
    echo "Then source the environment:"
    echo "  source .env.build"
    exit 1
fi

