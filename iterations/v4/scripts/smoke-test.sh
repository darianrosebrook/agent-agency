#!/bin/bash
# V4 Smoke Test Script
#
# This script verifies the V4 API server works correctly:
# 1. Builds the project
# 2. Starts the server
# 3. Tests all endpoints
# 4. Reports results
#
# Usage: ./scripts/smoke-test.sh [--skip-build] [--port PORT]

set -e

# Configuration
PORT="${V4_PORT:-8080}"
HOST="${V4_HOST:-127.0.0.1}"
BASE_URL="http://${HOST}:${PORT}"
SKIP_BUILD=false
SERVER_PID=""
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
V4_DIR="$(dirname "$SCRIPT_DIR")"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --port)
            PORT="$2"
            BASE_URL="http://${HOST}:${PORT}"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [--skip-build] [--port PORT]"
            echo ""
            echo "Options:"
            echo "  --skip-build  Skip cargo build step"
            echo "  --port PORT   Use custom port (default: 8080)"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

# Cleanup function
cleanup() {
    if [[ -n "$SERVER_PID" ]]; then
        echo -e "${YELLOW}Stopping server (PID: $SERVER_PID)...${NC}"
        kill "$SERVER_PID" 2>/dev/null || true
        wait "$SERVER_PID" 2>/dev/null || true
    fi
}
trap cleanup EXIT

# Print header
print_header() {
    echo ""
    echo -e "${BLUE}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║              V4 End-to-End Smoke Test                     ║${NC}"
    echo -e "${BLUE}╚═══════════════════════════════════════════════════════════╝${NC}"
    echo ""
}

# Print test result
print_result() {
    local name="$1"
    local status="$2"
    local details="$3"

    if [[ "$status" == "PASS" ]]; then
        echo -e "  ${GREEN}✓${NC} $name"
    else
        echo -e "  ${RED}✗${NC} $name"
        if [[ -n "$details" ]]; then
            echo -e "    ${RED}$details${NC}"
        fi
    fi
}

# Check if jq is available
check_dependencies() {
    if ! command -v jq &> /dev/null; then
        echo -e "${YELLOW}Warning: jq not found. JSON output will not be pretty-printed.${NC}"
        JQ_CMD="cat"
    else
        JQ_CMD="jq"
    fi

    if ! command -v curl &> /dev/null; then
        echo -e "${RED}Error: curl is required but not found.${NC}"
        exit 1
    fi
}

# Build the project
build_project() {
    if [[ "$SKIP_BUILD" == true ]]; then
        echo -e "${YELLOW}Skipping build (--skip-build)${NC}"
        return 0
    fi

    echo -e "${BLUE}Building V4...${NC}"
    cd "$V4_DIR"
    if cargo build -p v4-api --bin v4-server 2>&1; then
        echo -e "${GREEN}Build successful${NC}"
    else
        echo -e "${RED}Build failed${NC}"
        exit 1
    fi
}

# Start the server
start_server() {
    echo -e "${BLUE}Starting V4 API server on port $PORT...${NC}"
    cd "$V4_DIR"
    V4_PORT="$PORT" V4_LOG_LEVEL=warn cargo run -p v4-api --bin v4-server &
    SERVER_PID=$!

    # Wait for server to be ready
    echo -n "Waiting for server to start"
    for i in {1..30}; do
        if curl -sf "$BASE_URL/health" > /dev/null 2>&1; then
            echo ""
            echo -e "${GREEN}Server started (PID: $SERVER_PID)${NC}"
            return 0
        fi
        echo -n "."
        sleep 0.5
    done

    echo ""
    echo -e "${RED}Server failed to start within 15 seconds${NC}"
    exit 1
}

# Test health endpoint
test_health() {
    echo -e "\n${BLUE}Testing Health Endpoint${NC}"

    local response
    response=$(curl -sf "$BASE_URL/health" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local health_status
        health_status=$(echo "$response" | $JQ_CMD -r '.status' 2>/dev/null || echo "unknown")
        if [[ "$health_status" == "healthy" ]]; then
            print_result "GET /health" "PASS"
            return 0
        else
            print_result "GET /health" "FAIL" "Status: $health_status"
            return 1
        fi
    else
        print_result "GET /health" "FAIL" "Request failed"
        return 1
    fi
}

# Test API info endpoint
test_api_info() {
    echo -e "\n${BLUE}Testing API Info Endpoint${NC}"

    local response
    response=$(curl -sf "$BASE_URL/api/v1" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local api_name
        api_name=$(echo "$response" | $JQ_CMD -r '.name' 2>/dev/null || echo "unknown")
        if [[ "$api_name" == *"V4"* ]]; then
            print_result "GET /api/v1" "PASS"
            return 0
        else
            print_result "GET /api/v1" "FAIL" "Unexpected response"
            return 1
        fi
    else
        print_result "GET /api/v1" "FAIL" "Request failed"
        return 1
    fi
}

# Test task submission
test_task_submission() {
    echo -e "\n${BLUE}Testing Task Submission${NC}"

    local response
    response=$(curl -sf -X POST "$BASE_URL/api/v1/tasks" \
        -H "Content-Type: application/json" \
        -d '{"title": "Smoke test task", "description": "Read the Cargo.toml file"}' 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        TASK_ID=$(echo "$response" | $JQ_CMD -r '.task_id' 2>/dev/null || echo "")
        if [[ -n "$TASK_ID" && "$TASK_ID" != "null" ]]; then
            print_result "POST /api/v1/tasks" "PASS"
            echo -e "    Task ID: $TASK_ID"
            return 0
        else
            print_result "POST /api/v1/tasks" "FAIL" "No task_id in response"
            return 1
        fi
    else
        print_result "POST /api/v1/tasks" "FAIL" "Request failed"
        return 1
    fi
}

# Test get task status
test_get_task() {
    echo -e "\n${BLUE}Testing Get Task Status${NC}"

    if [[ -z "$TASK_ID" ]]; then
        print_result "GET /api/v1/tasks/:id" "SKIP" "No task ID from previous test"
        return 1
    fi

    local response
    response=$(curl -sf "$BASE_URL/api/v1/tasks/$TASK_ID" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local returned_id
        returned_id=$(echo "$response" | $JQ_CMD -r '.task_id' 2>/dev/null || echo "")
        if [[ "$returned_id" == "$TASK_ID" ]]; then
            print_result "GET /api/v1/tasks/:id" "PASS"
            return 0
        else
            print_result "GET /api/v1/tasks/:id" "FAIL" "Task ID mismatch"
            return 1
        fi
    else
        print_result "GET /api/v1/tasks/:id" "FAIL" "Request failed"
        return 1
    fi
}

# Test chain-of-thought endpoint
test_chain_of_thought() {
    echo -e "\n${BLUE}Testing Chain-of-Thought Endpoint${NC}"

    if [[ -z "$TASK_ID" ]]; then
        print_result "GET /api/v1/tasks/:id/chain-of-thought" "SKIP" "No task ID"
        return 1
    fi

    local response
    response=$(curl -sf "$BASE_URL/api/v1/tasks/$TASK_ID/chain-of-thought" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local has_steps
        has_steps=$(echo "$response" | $JQ_CMD 'has("steps")' 2>/dev/null || echo "false")
        if [[ "$has_steps" == "true" ]]; then
            print_result "GET /api/v1/tasks/:id/chain-of-thought" "PASS"
            return 0
        else
            print_result "GET /api/v1/tasks/:id/chain-of-thought" "FAIL" "Missing 'steps' field"
            return 1
        fi
    else
        print_result "GET /api/v1/tasks/:id/chain-of-thought" "FAIL" "Request failed"
        return 1
    fi
}

# Test council-decisions endpoint
test_council_decisions() {
    echo -e "\n${BLUE}Testing Council Decisions Endpoint${NC}"

    if [[ -z "$TASK_ID" ]]; then
        print_result "GET /api/v1/tasks/:id/council-decisions" "SKIP" "No task ID"
        return 1
    fi

    local response
    response=$(curl -sf "$BASE_URL/api/v1/tasks/$TASK_ID/council-decisions" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local has_judges
        has_judges=$(echo "$response" | $JQ_CMD 'has("judges")' 2>/dev/null || echo "false")
        if [[ "$has_judges" == "true" ]]; then
            print_result "GET /api/v1/tasks/:id/council-decisions" "PASS"
            return 0
        else
            print_result "GET /api/v1/tasks/:id/council-decisions" "FAIL" "Missing 'judges' field"
            return 1
        fi
    else
        print_result "GET /api/v1/tasks/:id/council-decisions" "FAIL" "Request failed"
        return 1
    fi
}

# Test worker-actions endpoint
test_worker_actions() {
    echo -e "\n${BLUE}Testing Worker Actions Endpoint${NC}"

    if [[ -z "$TASK_ID" ]]; then
        print_result "GET /api/v1/tasks/:id/worker-actions" "SKIP" "No task ID"
        return 1
    fi

    local response
    response=$(curl -sf "$BASE_URL/api/v1/tasks/$TASK_ID/worker-actions" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local has_actions
        has_actions=$(echo "$response" | $JQ_CMD 'has("actions")' 2>/dev/null || echo "false")
        if [[ "$has_actions" == "true" ]]; then
            print_result "GET /api/v1/tasks/:id/worker-actions" "PASS"
            return 0
        else
            print_result "GET /api/v1/tasks/:id/worker-actions" "FAIL" "Missing 'actions' field"
            return 1
        fi
    else
        print_result "GET /api/v1/tasks/:id/worker-actions" "FAIL" "Request failed"
        return 1
    fi
}

# Test probe endpoint
test_probe() {
    echo -e "\n${BLUE}Testing LLM Probe Endpoint${NC}"

    local response
    response=$(curl -sf -X POST "$BASE_URL/api/v1/probe" \
        -H "Content-Type: application/json" \
        -d '{"prompt": "Explain recursion in one sentence", "max_tokens": 50}' 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local has_text
        has_text=$(echo "$response" | $JQ_CMD 'has("text")' 2>/dev/null || echo "false")
        if [[ "$has_text" == "true" ]]; then
            print_result "POST /api/v1/probe" "PASS"
            local text
            text=$(echo "$response" | $JQ_CMD -r '.text' 2>/dev/null | head -c 60)
            echo -e "    Response: ${text}..."
            return 0
        else
            print_result "POST /api/v1/probe" "FAIL" "Missing 'text' field"
            return 1
        fi
    else
        print_result "POST /api/v1/probe" "FAIL" "Request failed"
        return 1
    fi
}

# Test metrics endpoint
test_metrics() {
    echo -e "\n${BLUE}Testing Metrics Endpoint${NC}"

    local response
    response=$(curl -sf "$BASE_URL/metrics" 2>&1)
    local status=$?

    if [[ $status -eq 0 ]]; then
        local has_requests
        has_requests=$(echo "$response" | $JQ_CMD 'has("total_requests")' 2>/dev/null || echo "false")
        if [[ "$has_requests" == "true" ]]; then
            print_result "GET /metrics" "PASS"
            local count
            count=$(echo "$response" | $JQ_CMD '.total_requests' 2>/dev/null)
            echo -e "    Total requests: $count"
            return 0
        else
            print_result "GET /metrics" "FAIL" "Missing 'total_requests' field"
            return 1
        fi
    else
        print_result "GET /metrics" "FAIL" "Request failed"
        return 1
    fi
}

# Print summary
print_summary() {
    local passed=$1
    local total=$2

    echo ""
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    if [[ $passed -eq $total ]]; then
        echo -e "${GREEN}All $total tests passed!${NC}"
    else
        echo -e "${YELLOW}$passed/$total tests passed${NC}"
    fi
    echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
    echo ""
}

# Main execution
main() {
    print_header
    check_dependencies

    local passed=0
    local total=9

    # Build and start server
    build_project
    start_server

    # Run tests
    test_health && ((passed++))
    test_api_info && ((passed++))
    test_task_submission && ((passed++))
    test_get_task && ((passed++))
    test_chain_of_thought && ((passed++))
    test_council_decisions && ((passed++))
    test_worker_actions && ((passed++))
    test_probe && ((passed++))
    test_metrics && ((passed++))

    # Print summary
    print_summary $passed $total

    # Return appropriate exit code
    if [[ $passed -eq $total ]]; then
        exit 0
    else
        exit 1
    fi
}

# Run main
main
