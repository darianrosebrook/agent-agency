#!/usr/bin/env bash
# Dashboard Readiness Check Module for V3 Readiness Framework
# Assesses dashboard integration readiness
# @author: @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_DIR="$ROOT_DIR/iterations/v3"
CONFIG_FILE="$SCRIPT_DIR/config.yaml"
OUTPUT_DIR="$ROOT_DIR/artifacts"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Load config values
get_config() {
    local key="$1"
    grep "^${key}:" "$CONFIG_FILE" | cut -d: -f2 | tr -d ' "'
}

get_config_list() {
    local section="$1"
    grep -A 10 "^${section}:" "$CONFIG_FILE" | grep "^-" | sed 's/^- //' | tr '\n' ' '
}

DASHBOARD_PATH=$(grep -A 10 "dashboard:" "$CONFIG_FILE" | grep "path:" | cut -d: -f2 | awk '{print $1}')
API_ENDPOINTS=$(grep -A 10 "dashboard:" "$CONFIG_FILE" | grep -A 6 "api_endpoints:" | grep "^-" | sed 's/^- //' | sed 's/"//g' | tr '\n' ' ')
CRITICAL_WORKFLOWS=$(grep -A 15 "dashboard:" "$CONFIG_FILE" | grep -A 6 "critical_workflows:" | grep "^-" | sed 's/^- //' | sed 's/"//g' | tr '\n' ' ')

cd "$ROOT_DIR"

echo -e "${BLUE}[dashboard-readiness] Starting dashboard readiness assessment...${NC}"

# Resolve dashboard path (relative to root)
DASHBOARD_ABS_PATH="$ROOT_DIR/$DASHBOARD_PATH"
if [ ! -d "$DASHBOARD_ABS_PATH" ]; then
    echo -e "${YELLOW}[dashboard-readiness] Dashboard path not found: $DASHBOARD_ABS_PATH${NC}"
    echo -e "${YELLOW}[dashboard-readiness] Attempting to find dashboard...${NC}"
    # Try alternative paths
    if [ -d "$ROOT_DIR/apps/agent_management_dashboard" ]; then
        DASHBOARD_ABS_PATH="$ROOT_DIR/apps/agent_management_dashboard"
        echo -e "${GREEN}[dashboard-readiness] Found dashboard at: $DASHBOARD_ABS_PATH${NC}"
    else
        echo -e "${RED}[dashboard-readiness] Dashboard not found${NC}"
        exit 1
    fi
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Initialize results JSON
RESULTS_FILE="$OUTPUT_DIR/dashboard-readiness.json"
cat > "$RESULTS_FILE" <<EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "dashboard_path": "$DASHBOARD_ABS_PATH",
  "build_status": {
    "compiles": false,
    "errors": [],
    "warnings": []
  },
  "api_connectivity": {
    "base_url": "http://localhost:8080",
    "endpoints": {}
  },
  "schema_alignment": {
    "aligned": false,
    "issues": []
  },
  "missing_apis": [],
  "critical_workflows": {},
  "overall_readiness": false
}
EOF

cd "$DASHBOARD_ABS_PATH"

# Check if package.json exists
if [ ! -f "package.json" ]; then
    echo -e "${RED}[dashboard-readiness] package.json not found in dashboard directory${NC}"
    exit 1
fi

# Check TypeScript compilation
echo -e "${BLUE}[dashboard-readiness] Checking TypeScript compilation...${NC}"
TSC_OUTPUT="$OUTPUT_DIR/typescript-compile.log"

if command -v npm &> /dev/null; then
    # Check if node_modules exists, if not suggest install
    if [ ! -d "node_modules" ]; then
        echo -e "${YELLOW}[dashboard-readiness] node_modules not found, skipping TypeScript check (run npm install first)${NC}"
        TSC_ERRORS=0
        TSC_WARNINGS=0
    else
        # Try to run type check
        if npm run typecheck 2>&1 | tee "$TSC_OUTPUT"; then
            TSC_EXIT_CODE=0
        else
            TSC_EXIT_CODE=$?
        fi
        
        # Parse TypeScript errors/warnings
        TSC_ERRORS=$(grep -c "error TS" "$TSC_OUTPUT" 2>/dev/null || echo "0")
        TSC_WARNINGS=$(grep -c "warning TS" "$TSC_OUTPUT" 2>/dev/null || echo "0")
        
        # Extract error messages
        TSC_ERROR_MSGS=$(grep "error TS" "$TSC_OUTPUT" 2>/dev/null | head -20 | jq -R . | jq -s . || echo "[]")
    fi
else
    echo -e "${YELLOW}[dashboard-readiness] npm not found, skipping TypeScript check${NC}"
    TSC_ERRORS=0
    TSC_WARNINGS=0
    TSC_ERROR_MSGS="[]"
    TSC_EXIT_CODE=0
fi

# Update build status
tmp_file=$(mktemp)
jq \
    --argjson compiles "$([ $TSC_EXIT_CODE -eq 0 ] && echo true || echo false)" \
    --argjson errors "$TSC_ERROR_MSGS" \
    --argjson error_count "$TSC_ERRORS" \
    --argjson warning_count "$TSC_WARNINGS" \
    '.build_status = {
        compiles: $compiles,
        error_count: $error_count,
        warning_count: $warning_count,
        errors: $errors
    }' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Check API connectivity (if API server is running)
echo -e "${BLUE}[dashboard-readiness] Checking API connectivity...${NC}"
API_BASE_URL="http://localhost:8080"

# Check if API server is running
API_RUNNING=false
if curl -s -f "$API_BASE_URL/api/health" > /dev/null 2>&1; then
    API_RUNNING=true
    echo -e "${GREEN}[dashboard-readiness] API server appears to be running${NC}"
else
    echo -e "${YELLOW}[dashboard-readiness] API server not running or not accessible${NC}"
fi

# Test each endpoint
ENDPOINT_RESULTS="{}"
for endpoint in $API_ENDPOINTS; do
    endpoint_status="unknown"
    endpoint_error=""
    
    if [ "$API_RUNNING" = true ]; then
        if curl -s -f -o /dev/null -w "%{http_code}" "$API_BASE_URL$endpoint" > /tmp/curl_status 2>&1; then
            http_code=$(cat /tmp/curl_status)
            if [ "$http_code" -ge 200 ] && [ "$http_code" -lt 400 ]; then
                endpoint_status="available"
            else
                endpoint_status="error"
                endpoint_error="HTTP $http_code"
            fi
        else
            endpoint_status="unavailable"
            endpoint_error="Connection failed"
        fi
    else
        endpoint_status="server_not_running"
    fi
    
    ENDPOINT_RESULTS=$(echo "$ENDPOINT_RESULTS" | jq \
        --arg endpoint "$endpoint" \
        --arg status "$endpoint_status" \
        --arg error "$endpoint_error" \
        '. + {($endpoint): {status: $status, error: $error}}')
done

# Update API connectivity
tmp_file=$(mktemp)
jq \
    --argjson running "$([ "$API_RUNNING" = true ] && echo true || echo false)" \
    --argjson endpoints "$ENDPOINT_RESULTS" \
    '.api_connectivity = {
        server_running: $running,
        base_url: "http://localhost:8080",
        endpoints: $endpoints
    }' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Check schema alignment (simplified - would need actual schema comparison)
echo -e "${BLUE}[dashboard-readiness] Checking schema alignment...${NC}"
SCHEMA_ISSUES=()

# Check if TypeScript types exist for API contracts
TS_TYPES_DIR="$DASHBOARD_ABS_PATH/src/lib/types"
if [ -d "$TS_TYPES_DIR" ]; then
    # Count type definition files
    TYPE_FILES=$(find "$TS_TYPES_DIR" -name "*.ts" -type f | wc -l)
    if [ "$TYPE_FILES" -eq 0 ]; then
        SCHEMA_ISSUES+=("No TypeScript type definitions found")
    fi
else
    SCHEMA_ISSUES+=("TypeScript types directory not found: $TS_TYPES_DIR")
fi

# Check for API client files
API_CLIENT_DIR="$DASHBOARD_ABS_PATH/src/lib/api"
if [ ! -d "$API_CLIENT_DIR" ]; then
    SCHEMA_ISSUES+=("API client directory not found: $API_CLIENT_DIR")
fi

# Update schema alignment
tmp_file=$(mktemp)
jq \
    --argjson aligned "$([ ${#SCHEMA_ISSUES[@]} -eq 0 ] && echo true || echo false)" \
    --argjson issues "$(printf '%s\n' "${SCHEMA_ISSUES[@]}" | jq -R . | jq -s .)" \
    '.schema_alignment = {
        aligned: $aligned,
        issues: $issues
    }' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Check for missing API implementations (simplified check)
echo -e "${BLUE}[dashboard-readiness] Checking for missing API implementations...${NC}"
MISSING_APIS=()

if [ -d "$API_CLIENT_DIR" ]; then
    for endpoint in $API_ENDPOINTS; do
        # Extract endpoint name (e.g., /api/agents -> agents)
        endpoint_name=$(echo "$endpoint" | sed 's|/api/||' | cut -d'/' -f1)
        
        # Check if corresponding API file exists
        api_file="$API_CLIENT_DIR/${endpoint_name}.ts"
        if [ ! -f "$api_file" ]; then
            MISSING_APIS+=("$endpoint (expected: $api_file)")
        fi
    done
else
    MISSING_APIS+=("API client directory not found")
fi

# Update missing APIs
tmp_file=$(mktemp)
jq \
    --argjson missing "$(printf '%s\n' "${MISSING_APIS[@]}" | jq -R . | jq -s .)" \
    '.missing_apis = $missing' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Check critical workflows (simplified - would need actual workflow tests)
echo -e "${BLUE}[dashboard-readiness] Checking critical workflows...${NC}"
WORKFLOW_RESULTS="{}"

for workflow in $CRITICAL_WORKFLOWS; do
    # Check if workflow-related components exist
    workflow_status="unknown"
    
    # Look for workflow-related files
    workflow_files=$(find "$DASHBOARD_ABS_PATH/src" -type f -name "*${workflow}*" 2>/dev/null | wc -l)
    
    if [ "$workflow_files" -gt 0 ]; then
        workflow_status="implemented"
    else
        workflow_status="missing"
    fi
    
    WORKFLOW_RESULTS=$(echo "$WORKFLOW_RESULTS" | jq \
        --arg workflow "$workflow" \
        --arg status "$workflow_status" \
        --argjson file_count "$workflow_files" \
        '. + {($workflow): {status: $status, file_count: $file_count}}')
done

# Update critical workflows
tmp_file=$(mktemp)
jq \
    --argjson workflows "$WORKFLOW_RESULTS" \
    '.critical_workflows = $workflows' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Calculate overall readiness
READY=true

# Check build status
BUILDS=$(jq -r '.build_status.compiles' "$RESULTS_FILE")
if [ "$BUILDS" != "true" ]; then
    READY=false
fi

# Check API connectivity (at least health endpoint should work)
HEALTH_STATUS=$(jq -r '.api_connectivity.endpoints."/api/health".status' "$RESULTS_FILE")
if [ "$HEALTH_STATUS" != "available" ]; then
    READY=false
fi

# Check schema alignment
SCHEMA_ALIGNED=$(jq -r '.schema_alignment.aligned' "$RESULTS_FILE")
if [ "$SCHEMA_ALIGNED" != "true" ]; then
    READY=false
fi

# Check missing APIs
MISSING_COUNT=$(jq -r '.missing_apis | length' "$RESULTS_FILE")
if [ "$MISSING_COUNT" -gt 0 ]; then
    READY=false
fi

# Update overall readiness
tmp_file=$(mktemp)
jq --argjson ready "$READY" '.overall_readiness = $ready' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Print summary
echo -e "${BLUE}[dashboard-readiness] Dashboard Readiness Summary:${NC}"
echo -e "  Build Status: $([ "$BUILDS" = "true" ] && echo -e "${GREEN}Compiles${NC}" || echo -e "${RED}Has Errors${NC}")"
echo -e "  TypeScript Errors: $TSC_ERRORS"
echo -e "  TypeScript Warnings: $TSC_WARNINGS"
echo -e "  API Server: $([ "$API_RUNNING" = true ] && echo -e "${GREEN}Running${NC}" || echo -e "${YELLOW}Not Running${NC}")"
echo -e "  Schema Alignment: $([ "$SCHEMA_ALIGNED" = "true" ] && echo -e "${GREEN}Aligned${NC}" || echo -e "${YELLOW}Issues Found${NC}")"
echo -e "  Missing APIs: $MISSING_COUNT"
echo -e "  Overall Readiness: $([ "$READY" = "true" ] && echo -e "${GREEN}Ready${NC}" || echo -e "${RED}Not Ready${NC}")"

if [ ${#SCHEMA_ISSUES[@]} -gt 0 ]; then
    echo -e "${YELLOW}  Schema Issues:${NC}"
    for issue in "${SCHEMA_ISSUES[@]}"; do
        echo -e "    - $issue"
    done
fi

if [ ${#MISSING_APIS[@]} -gt 0 ]; then
    echo -e "${YELLOW}  Missing API Implementations:${NC}"
    for api in "${MISSING_APIS[@]}"; do
        echo -e "    - $api"
    done
fi

echo -e "${GREEN}[dashboard-readiness] Results saved to $RESULTS_FILE${NC}"

