#!/bin/bash
# Setup environment for running Agent Agency V3 tests on macOS
# Uses system Swift 6 runtime instead of Xcode toolchain Swift 5.5

export DYLD_LIBRARY_PATH="/usr/lib/swift:${DYLD_LIBRARY_PATH:-}"
export DATABASE_URL="${DATABASE_URL:-postgresql://test_user:test_password@localhost:5433/agent_agency_test}"

echo "Environment configured:"
echo "  DYLD_LIBRARY_PATH=$DYLD_LIBRARY_PATH"
echo "  DATABASE_URL=$DATABASE_URL"
echo ""
echo "Swift version: $(swift --version | head -1)"
echo ""
echo "To run tests, use:"
echo "  source $0"
echo "  cargo test --package <package-name>"

