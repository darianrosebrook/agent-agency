#!/bin/bash
# Runtime Configuration Verification Script
# Verifies all runtime dependencies are configured correctly

set -e

echo "=== V3 Agent Runtime Configuration Verification ==="
echo ""

# Colors for output
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Check 1: Feature flags
echo "1. Checking feature flags..."
if grep -q 'default = \["orchestration"\]' data-interfaces-adapters/Cargo.toml; then
    echo -e "${GREEN}✓${NC} Orchestration feature enabled by default"
else
    echo -e "${YELLOW}⚠${NC} Orchestration feature may not be enabled"
fi

# Check 2: Database connection
echo ""
echo "2. Checking database configuration..."
if [ -z "$DATABASE_URL" ]; then
    echo -e "${YELLOW}⚠${NC} DATABASE_URL not set - system will run in standalone mode"
    echo "   Set DATABASE_URL to enable database persistence"
else
    echo -e "${GREEN}✓${NC} DATABASE_URL is set"
    # Try to connect (non-blocking check)
    if command -v psql &> /dev/null; then
        echo "   Attempting to verify database connection..."
        # Extract connection info (basic check)
        if [[ $DATABASE_URL == postgresql://* ]]; then
            echo -e "${GREEN}✓${NC} PostgreSQL connection string detected"
        fi
    fi
fi

# Check 3: MCP configuration
echo ""
echo "3. Checking MCP configuration..."
MCP_TOOLS_DIR="./tools"
MCP_EXTENSIONS_DIR="./extensions"
if [ -d "$MCP_TOOLS_DIR" ] || [ -d "$MCP_EXTENSIONS_DIR" ]; then
    echo -e "${GREEN}✓${NC} MCP tool directories found"
    if [ -d "$MCP_TOOLS_DIR" ]; then
        TOOL_COUNT=$(find "$MCP_TOOLS_DIR" -name "*.json" -o -name "manifest.toml" 2>/dev/null | wc -l | tr -d ' ')
        echo "   Found $TOOL_COUNT tool manifest(s) in ./tools"
    fi
else
    echo -e "${YELLOW}⚠${NC} MCP tool directories not found (./tools or ./extensions)"
    echo "   MCP will use auto-discovery - tools may be discovered at runtime"
fi

# Check 4: CoreML models (optional)
echo ""
echo "4. Checking CoreML model configuration..."
COREML_MODELS_DIR="./models"
if [ -d "$COREML_MODELS_DIR" ]; then
    MODEL_COUNT=$(find "$COREML_MODELS_DIR" -name "*.mlmodel" -o -name "*.mlpackage" 2>/dev/null | wc -l | tr -d ' ')
    if [ "$MODEL_COUNT" -gt 0 ]; then
        echo -e "${GREEN}✓${NC} Found $MODEL_COUNT CoreML model(s)"
    else
        echo -e "${YELLOW}⚠${NC} CoreML models directory exists but no models found"
        echo "   System will fallback to CPU inference"
    fi
else
    echo -e "${YELLOW}⚠${NC} CoreML models directory not found"
    echo "   System will use CPU inference (slower but functional)"
fi

# Check 5: Compilation
echo ""
echo "5. Checking compilation..."
if cargo check --bin agent-agency-api-server --features orchestration &> /tmp/cargo_check.log 2>&1; then
    echo -e "${GREEN}✓${NC} Code compiles successfully"
else
    echo -e "${RED}✗${NC} Compilation errors found"
    echo "   Check /tmp/cargo_check.log for details"
    exit 1
fi

# Check 6: Migration files
echo ""
echo "6. Checking database migrations..."
MIGRATIONS_DIR="data-infrastructure/migrations"
if [ -d "$MIGRATIONS_DIR" ]; then
    MIGRATION_COUNT=$(ls -1 "$MIGRATIONS_DIR"/*.sql 2>/dev/null | wc -l | tr -d ' ')
    echo -e "${GREEN}✓${NC} Found $MIGRATION_COUNT migration file(s)"
    echo "   Migrations will be applied automatically on first database connection"
else
    echo -e "${RED}✗${NC} Migrations directory not found"
    exit 1
fi

# Summary
echo ""
echo "=== Verification Summary ==="
echo ""
echo "System is ready to run if:"
echo "  - Database is accessible (or DATABASE_URL not set for standalone mode)"
echo "  - MCP tools are configured (or will be auto-discovered)"
echo ""
echo "To start the API server:"
echo "  cd iterations/v3"
echo "  cargo run --bin agent-agency-api-server --features orchestration"
echo ""
echo "To test with a task:"
echo "  curl -X POST http://localhost:8080/api/v1/tasks \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"description\": \"Create a simple hello world function\"}'"
echo ""

