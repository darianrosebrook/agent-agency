#!/bin/bash
# Comprehensive Test Runner for Agent Agency V3
# 
# This script runs all tests with comprehensive reporting and analysis

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TEST_TIMEOUT=300  # 5 minutes timeout per test suite
COVERAGE_THRESHOLD=80
PERFORMANCE_THRESHOLD_MS=1000

# Create test results directory
mkdir -p test-results
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
RESULTS_DIR="test-results/test_run_$TIMESTAMP"
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}🚀 Starting Comprehensive Test Suite for Agent Agency V3${NC}"
echo -e "${BLUE}📁 Results will be saved to: $RESULTS_DIR${NC}"
echo ""

# Function to run tests with timeout and capture results
run_test_suite() {
    local suite_name="$1"
    local test_command="$2"
    local output_file="$RESULTS_DIR/${suite_name}.log"
    
    echo -e "${YELLOW}🧪 Running $suite_name...${NC}"
    
    if timeout $TEST_TIMEOUT bash -c "$test_command" > "$output_file" 2>&1; then
        echo -e "${GREEN}✅ $suite_name completed successfully${NC}"
        return 0
    else
        local exit_code=$?
        echo -e "${RED}❌ $suite_name failed (exit code: $exit_code)${NC}"
        return $exit_code
    fi
}

# Function to analyze test results
analyze_test_results() {
    local suite_name="$1"
    local log_file="$RESULTS_DIR/${suite_name}.log"
    
    if [[ -f "$log_file" ]]; then
        local passed=$(grep -c "test result: ok" "$log_file" 2>/dev/null || echo "0")
        local failed=$(grep -c "test result: FAILED" "$log_file" 2>/dev/null || echo "0")
        local total=$((passed + failed))
        
        if [[ $total -gt 0 ]]; then
            local success_rate=$((passed * 100 / total))
            echo "  📊 Results: $passed passed, $failed failed ($success_rate% success rate)"
            
            if [[ $failed -gt 0 ]]; then
                echo -e "  ${RED}❌ Failed tests detected${NC}"
                grep "test result: FAILED" "$log_file" | head -5
            fi
        else
            echo "  📊 No test results found"
        fi
    fi
}

# Function to check compilation status
check_compilation() {
    echo -e "${BLUE}🔧 Checking compilation status...${NC}"
    
    local compile_log="$RESULTS_DIR/compilation.log"
    if cargo check --workspace > "$compile_log" 2>&1; then
        echo -e "${GREEN}✅ All packages compile successfully${NC}"
        return 0
    else
        local error_count=$(grep -c "error\[" "$compile_log" 2>/dev/null || echo "0")
        echo -e "${RED}❌ Compilation failed with $error_count errors${NC}"
        
        # Show first few errors
        echo -e "${YELLOW}First few compilation errors:${NC}"
        grep "error\[" "$compile_log" | head -5
        
        return 1
    fi
}

# Function to run unit tests
run_unit_tests() {
    echo -e "${BLUE}🧪 Running Unit Tests...${NC}"
    
    local unit_test_suites=(
        "council:agent-agency-council"
        "claim-extraction:agent-agency-claim-extraction"
        "research:agent-agency-research"
        "orchestration:agent-agency-orchestration"
        "embedding-service:agent-agency-embedding-service"
    )
    
    local total_passed=0
    local total_failed=0
    
    for suite in "${unit_test_suites[@]}"; do
        local name=$(echo "$suite" | cut -d: -f1)
        local package=$(echo "$suite" | cut -d: -f2)
        
        if run_test_suite "unit_$name" "cargo test --package $package --lib"; then
            analyze_test_results "unit_$name"
        else
            echo -e "${RED}❌ Unit tests for $name failed${NC}"
            total_failed=$((total_failed + 1))
        fi
    done
    
    echo -e "${BLUE}📊 Unit Test Summary: $total_passed passed, $total_failed failed${NC}"
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${BLUE}🔗 Running Integration Tests...${NC}"
    
    local integration_suites=(
        "cross_component:Cross-component integration tests"
        "end_to_end:End-to-end workflow tests"
        "performance:Performance benchmarks"
        "database:Database integration tests"
    )
    
    for suite in "${integration_suites[@]}"; do
        local name=$(echo "$suite" | cut -d: -f1)
        local description=$(echo "$suite" | cut -d: -f2)
        
        if run_test_suite "integration_$name" "cargo test --package agent-agency-integration-tests --lib $name"; then
            analyze_test_results "integration_$name"
        else
            echo -e "${RED}❌ Integration tests for $name failed${NC}"
        fi
    done
}

# Function to run performance tests
run_performance_tests() {
    echo -e "${BLUE}⚡ Running Performance Tests...${NC}"
    
    if run_test_suite "performance" "cargo test --package agent-agency-integration-tests --lib performance_benchmarks"; then
        analyze_test_results "performance"
        
        # Extract performance metrics
        local perf_log="$RESULTS_DIR/performance.log"
        if [[ -f "$perf_log" ]]; then
            echo -e "${YELLOW}📈 Performance Metrics:${NC}"
            grep -E "(Average|Requests per second|Operations per second|Memory|CPU)" "$perf_log" | head -10
        fi
    else
        echo -e "${RED}❌ Performance tests failed${NC}"
    fi
}

# Function to generate coverage report
generate_coverage_report() {
    echo -e "${BLUE}📊 Generating Coverage Report...${NC}"
    
    if command -v grcov >/dev/null 2>&1; then
        echo -e "${YELLOW}Running tests with coverage instrumentation...${NC}"
        
        # Run tests with coverage
        RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw" \
        cargo test --workspace --all-features > "$RESULTS_DIR/coverage_tests.log" 2>&1
        
        # Generate coverage report
        grcov . -s . -t lcov --llvm --branch --ignore-not-existing \
            -o "$RESULTS_DIR/coverage.lcov" --ignore "/*" --ignore "target/*" > "$RESULTS_DIR/coverage_generation.log" 2>&1
        
        if [[ -f "$RESULTS_DIR/coverage.lcov" ]]; then
            echo -e "${GREEN}✅ Coverage report generated${NC}"
            
            # Check coverage threshold
            if command -v lcov >/dev/null 2>&1; then
                local coverage_percent=$(lcov --summary "$RESULTS_DIR/coverage.lcov" 2>/dev/null | grep "lines" | grep -o '[0-9.]*%' | head -1 | sed 's/%//')
                if [[ -n "$coverage_percent" ]]; then
                    echo -e "${BLUE}📊 Coverage: ${coverage_percent}%${NC}"
                    if (( $(echo "$coverage_percent >= $COVERAGE_THRESHOLD" | bc -l) )); then
                        echo -e "${GREEN}✅ Coverage meets threshold ($COVERAGE_THRESHOLD%)${NC}"
                    else
                        echo -e "${YELLOW}⚠️  Coverage below threshold ($COVERAGE_THRESHOLD%)${NC}"
                    fi
                fi
            fi
        else
            echo -e "${RED}❌ Failed to generate coverage report${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  grcov not found, skipping coverage report${NC}"
    fi
}

# Function to generate test summary
generate_test_summary() {
    echo -e "${BLUE}📋 Generating Test Summary...${NC}"
    
    local summary_file="$RESULTS_DIR/test_summary.md"
    
    cat > "$summary_file" << EOF
# Test Run Summary

**Date**: $(date)
**Duration**: $(date -d@$SECONDS -u +%H:%M:%S)
**Results Directory**: $RESULTS_DIR

## Compilation Status
EOF

    if [[ -f "$RESULTS_DIR/compilation.log" ]]; then
        local error_count=$(grep -c "error\[" "$RESULTS_DIR/compilation.log" 2>/dev/null || echo "0")
        if [[ $error_count -eq 0 ]]; then
            echo "✅ All packages compile successfully" >> "$summary_file"
        else
            echo "❌ Compilation failed with $error_count errors" >> "$summary_file"
        fi
    fi

    cat >> "$summary_file" << EOF

## Test Results

EOF

    # Add test results for each suite
    for log_file in "$RESULTS_DIR"/*.log; do
        if [[ -f "$log_file" ]]; then
            local suite_name=$(basename "$log_file" .log)
            local passed=$(grep -c "test result: ok" "$log_file" 2>/dev/null || echo "0")
            local failed=$(grep -c "test result: FAILED" "$log_file" 2>/dev/null || echo "0")
            local total=$((passed + failed))
            
            if [[ $total -gt 0 ]]; then
                local success_rate=$((passed * 100 / total))
                echo "### $suite_name" >> "$summary_file"
                echo "- Passed: $passed" >> "$summary_file"
                echo "- Failed: $failed" >> "$summary_file"
                echo "- Success Rate: $success_rate%" >> "$summary_file"
                echo "" >> "$summary_file"
            fi
        fi
    done

    cat >> "$summary_file" << EOF

## Performance Metrics

EOF

    if [[ -f "$RESULTS_DIR/performance.log" ]]; then
        grep -E "(Average|Requests per second|Operations per second|Memory|CPU)" "$RESULTS_DIR/performance.log" >> "$summary_file" 2>/dev/null || echo "No performance metrics available" >> "$summary_file"
    fi

    echo -e "${GREEN}✅ Test summary generated: $summary_file${NC}"
}

# Main execution
main() {
    local start_time=$(date +%s)
    
    echo -e "${BLUE}🎯 Agent Agency V3 Comprehensive Test Suite${NC}"
    echo -e "${BLUE}===========================================${NC}"
    echo ""
    
    # Check compilation first
    if ! check_compilation; then
        echo -e "${RED}❌ Compilation failed. Please fix compilation errors before running tests.${NC}"
        exit 1
    fi
    
    # Run test suites
    run_unit_tests
    echo ""
    run_integration_tests
    echo ""
    run_performance_tests
    echo ""
    
    # Generate reports
    generate_coverage_report
    echo ""
    generate_test_summary
    
    # Final summary
    local end_time=$(date +%s)
    local duration=$((end_time - start_time))
    
    echo -e "${BLUE}🎉 Test Suite Complete!${NC}"
    echo -e "${BLUE}⏱️  Total Duration: $(date -d@$duration -u +%H:%M:%S)${NC}"
    echo -e "${BLUE}📁 Results: $RESULTS_DIR${NC}"
    echo ""
    echo -e "${GREEN}✅ All tests completed. Check the results directory for detailed reports.${NC}"
}

# Run main function
main "$@"
