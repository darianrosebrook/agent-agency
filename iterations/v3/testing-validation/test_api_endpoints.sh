#!/bin/bash
# Quick API endpoint validation test
# Tests that the API server is responding correctly

set -e

API_URL="${API_URL:-http://localhost:8080}"
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

log_info() {
    echo -e "${YELLOW}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

test_endpoint() {
    local method=$1
    local endpoint=$2
    local expected_status=${3:-200}
    
    log_info "Testing $method $endpoint..."
    
    if [ "$method" = "GET" ]; then
        response=$(curl -s -w "\n%{http_code}" "$API_URL$endpoint" || echo -e "\n000")
    else
        response=$(curl -s -w "\n%{http_code}" -X "$method" "$API_URL$endpoint" || echo -e "\n000")
    fi
    
    http_code=$(echo "$response" | tail -n1)
    body=$(echo "$response" | sed '$d')
    
    if [ "$http_code" = "$expected_status" ]; then
        log_success "$method $endpoint returned $http_code"
        return 0
    else
        log_error "$method $endpoint returned $http_code (expected $expected_status)"
        echo "Response body: $body"
        return 1
    fi
}

log_info "Starting API endpoint validation tests..."
log_info "API URL: $API_URL"

# Test health endpoint
test_endpoint "GET" "/health" 200

# Test API health endpoint
test_endpoint "GET" "/api/v1/health" 200

# Test root endpoint
test_endpoint "GET" "/" 200

# Test system health endpoint
test_endpoint "GET" "/api/v1/system/health" 200

# Test system metrics endpoint
test_endpoint "GET" "/api/v1/system/metrics" 200

# Test list tasks endpoint (should return empty array if no tasks)
test_endpoint "GET" "/api/v1/tasks" 200

# Test list queries endpoint
test_endpoint "GET" "/api/v1/queries" 200

# Test list waivers endpoint
test_endpoint "GET" "/api/v1/waivers" 200

# Test provenance endpoint
test_endpoint "GET" "/api/v1/provenance" 200

log_success "All API endpoint tests passed!"








