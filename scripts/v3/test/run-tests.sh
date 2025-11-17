#!/bin/bash
# Test Execution Script for V3 Agent Agency
# Provides comprehensive test execution with coverage reporting and database setup
#
# Usage:
#   ./scripts/v3/test/run-tests.sh [unit|integration|e2e|all] [--coverage] [--verbose]
#
# @author @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
cd "$PROJECT_ROOT"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_TYPE="${1:-all}"
COVERAGE="${2:-}"
VERBOSE="${3:-}"
DATABASE_URL="${DATABASE_URL:-postgresql://postgres@localhost:5432/postgres}"

# Test directories
V3_DIR="$PROJECT_ROOT/iterations/v3"
TESTING_VALIDATION_DIR="$V3_DIR/testing-validation"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  V3 Agent Agency Test Runner${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo "Test Type: $TEST_TYPE"
echo "Database URL: $DATABASE_URL"
echo ""

# Function to check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}Checking prerequisites...${NC}"
    
    # Check if PostgreSQL is accessible
    if ! psql "$DATABASE_URL" -c "SELECT 1" > /dev/null 2>&1; then
        echo -e "${RED}ERROR: Cannot connect to PostgreSQL database${NC}"
        echo "Please ensure PostgreSQL is running and DATABASE_URL is set correctly"
        exit 1
    fi
    
    # Check if Rust/Cargo is available
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}ERROR: cargo not found${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}Prerequisites check passed${NC}"
    echo ""
}

# Function to run unit tests
run_unit_tests() {
    echo -e "${BLUE}Running unit tests...${NC}"
    
    cd "$V3_DIR"
    
    local test_args="--lib --tests"
    if [[ "$VERBOSE" == "--verbose" ]]; then
        test_args="$test_args -- --nocapture"
    fi
    
    if [[ "$COVERAGE" == "--coverage" ]]; then
        echo "Running with coverage (requires cargo-llvm-cov)..."
        cargo llvm-cov --workspace --lib --tests --lcov --output-path coverage.lcov
    else
        cargo test --workspace $test_args
    fi
    
    echo -e "${GREEN}Unit tests completed${NC}"
    echo ""
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${BLUE}Running integration tests...${NC}"
    
    cd "$V3_DIR"
    
    # Ensure test database is available
    export DATABASE_URL
    
    local test_args="--test '*'"
    if [[ "$VERBOSE" == "--verbose" ]]; then
        test_args="$test_args -- --nocapture"
    fi
    
    if [[ "$COVERAGE" == "--coverage" ]]; then
        echo "Running integration tests with coverage..."
        cargo llvm-cov --workspace --test '*' --lcov --output-path coverage-integration.lcov
    else
        cargo test --workspace $test_args
    fi
    
    echo -e "${GREEN}Integration tests completed${NC}"
    echo ""
}

# Function to run E2E tests
run_e2e_tests() {
    echo -e "${BLUE}Running E2E tests...${NC}"
    
    cd "$TESTING_VALIDATION_DIR"
    
    export DATABASE_URL
    
    if [[ "$VERBOSE" == "--verbose" ]]; then
        cargo test --features "full" -- --nocapture
    else
        cargo test --features "full"
    fi
    
    echo -e "${GREEN}E2E tests completed${NC}"
    echo ""
}

# Function to generate coverage report
generate_coverage_report() {
    if [[ "$COVERAGE" == "--coverage" ]]; then
        echo -e "${BLUE}Generating coverage report...${NC}"
        
        if command -v genhtml &> /dev/null; then
            genhtml coverage.lcov -o coverage-report
            echo -e "${GREEN}Coverage report generated: coverage-report/index.html${NC}"
        else
            echo -e "${YELLOW}genhtml not found. Install lcov to generate HTML reports${NC}"
        fi
    fi
}

# Main execution
main() {
    check_prerequisites
    
    case "$TEST_TYPE" in
        unit)
            run_unit_tests
            ;;
        integration)
            run_integration_tests
            ;;
        e2e)
            run_e2e_tests
            ;;
        all)
            run_unit_tests
            run_integration_tests
            run_e2e_tests
            ;;
        *)
            echo -e "${RED}Unknown test type: $TEST_TYPE${NC}"
            echo "Usage: $0 [unit|integration|e2e|all] [--coverage] [--verbose]"
            exit 1
            ;;
    esac
    
    generate_coverage_report
    
    echo -e "${GREEN}========================================${NC}"
    echo -e "${GREEN}  All tests completed successfully!${NC}"
    echo -e "${GREEN}========================================${NC}"
}

main

