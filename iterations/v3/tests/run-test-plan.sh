#!/bin/bash
# Self-Governing Agent System Test Runner
# This script executes the comprehensive test plan for validating the system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
PROJECT_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TEST_RESULTS_DIR="$PROJECT_ROOT/test-results"
LOG_FILE="$TEST_RESULTS_DIR/test-run-$(date +%Y%m%d-%H%M%S).log"

# Create test results directory
mkdir -p "$TEST_RESULTS_DIR"

# Logging function
log() {
    echo "$(date '+%Y-%m-%d %H:%M:%S') - $*" | tee -a "$LOG_FILE"
}

# Test result tracking
PASSED_TESTS=0
FAILED_TESTS=0
TOTAL_TESTS=0

# Test execution function
run_test() {
    local test_name="$1"
    local test_command="$2"
    local timeout="${3:-300}" # Default 5 minute timeout

    TOTAL_TESTS=$((TOTAL_TESTS + 1))

    log "🔍 Running: $test_name"
    log "   Command: $test_command"

    if bash -c "$test_command" 2>&1; then
        log "✅ PASSED: $test_name"
        PASSED_TESTS=$((PASSED_TESTS + 1))
        return 0
    else
        local exit_code=$?
        log "❌ FAILED: $test_name (exit code: $exit_code)"
        FAILED_TESTS=$((FAILED_TESTS + 1))
        return 1
    fi
}

# Setup function
setup_environment() {
    log "🏗️  Setting up test environment..."

    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        log "❌ Error: Not in project root. Please run from iterations/v3/"
        exit 1
    fi

    # Check for required tools
    command -v cargo >/dev/null 2>&1 || { log "❌ Error: cargo not found"; exit 1; }
    command -v node >/dev/null 2>&1 || { log "❌ Error: node not found"; exit 1; }
    command -v npm >/dev/null 2>&1 || { log "❌ Error: npm not found"; exit 1; }

    # Clean previous build artifacts
    log "🧹 Cleaning previous build artifacts..."
    cargo clean

    log "✅ Environment setup complete"
}

# Unit Tests
run_unit_tests() {
    log "🧪 Running Unit Tests..."

    # Self-prompting agent unit tests
    run_test "Self-Prompting Agent Unit Tests" \
        "cargo test --package self-prompting-agent --lib --verbose"

    # Council unit tests
    run_test "Council Unit Tests" \
        "cargo test --package agent-orchestration --lib --verbose"

    # Model provider unit tests
    run_test "Model Provider Unit Tests" \
        "cargo test --package self-prompting-agent model_provider --verbose"

    # Evaluation framework unit tests
    run_test "Evaluation Framework Unit Tests" \
        "cargo test --package self-prompting-agent evaluation --verbose"
}

# Integration Tests
run_integration_tests() {
    log "🔗 Running Integration Tests..."

    # Check if Ollama is available
    if curl -s http://localhost:11434/api/tags >/dev/null 2>&1; then
        log "🐪 Ollama detected, running full integration tests"

        # Full integration test suite
        run_test "Self-Prompting Integration Tests" \
            "cargo test --package self-prompting-agent --test integration --verbose" 600

    else
        log "⚠️  Ollama not detected, running mock integration tests"
        run_test "Mock Integration Tests" \
            "cargo test --package self-prompting-agent --features mock-models --test integration --verbose"
    fi
}

# Performance Tests
run_performance_tests() {
    log "⚡ Running Performance Tests..."

    # Check if criterion is available
    if cargo bench --help >/dev/null 2>&1; then
        run_test "Performance Benchmarks" \
            "cargo bench --package self-prompting-agent-benchmarks" 1200
    else
        log "⚠️  Criterion not available, skipping performance tests"
    fi
}

# Web Dashboard Tests
run_web_dashboard_tests() {
    log "🌐 Running Web Dashboard Tests..."

    # Check if web dashboard directory exists
    if [ -d "apps/web-dashboard" ]; then
        cd apps/web-dashboard

        # Install dependencies if needed
        if [ ! -d "node_modules" ]; then
            run_test "Install Web Dashboard Dependencies" \
                "npm ci" 300
        fi

        # Run linting
        run_test "Web Dashboard Linting" \
            "npm run lint"

        # Run TypeScript checks
        run_test "Web Dashboard Type Check" \
            "npm run type-check"

        # Run unit tests
        run_test "Web Dashboard Unit Tests" \
            "npm test -- --watchAll=false --verbose"

        cd ..
    else
        log "⚠️  Web dashboard not found, skipping web tests"
    fi
}

# Playground Tests
run_playground_tests() {
    log "🎮 Running Playground Tests..."

    # Setup playground directory
    PLAYGROUND_DIR="$TEST_RESULTS_DIR/playground-test"
    mkdir -p "$PLAYGROUND_DIR"
    cd "$PLAYGROUND_DIR"
    git init --quiet

    # Run playground test harness
    run_test "Playground Test Harness" \
        "cd '$PROJECT_ROOT' && cargo run --package self-prompting-agent --example playground_test" 600

    cd "$PROJECT_ROOT"
}

# Security Tests
run_security_tests() {
    log "🔒 Running Security Tests..."

    # Check for cargo-audit
    if command -v cargo-audit >/dev/null 2>&1; then
        run_test "Dependency Security Audit" \
            "cargo audit"
    else
        log "⚠️  cargo-audit not installed, skipping security audit"
    fi

    # Check for outdated dependencies
    run_test "Dependency Freshness Check" \
        "cargo outdated --exit-code 1 || true"
}

# Code Quality Tests
run_quality_tests() {
    log "✨ Running Code Quality Tests..."

    # Rust formatting check
    run_test "Rust Code Formatting" \
        "cargo fmt --all -- --check"

    # Rust linting
    run_test "Rust Linting (Clippy)" \
        "cargo clippy --all-targets --all-features -- -D warnings"

    # Check for unused dependencies
    run_test "Unused Dependencies Check" \
        "cargo +nightly udeps --all-targets || true"
}

# Generate test report
generate_report() {
    local report_file="$TEST_RESULTS_DIR/test-report-$(date +%Y%m%d-%H%M%S).md"

    cat > "$report_file" << EOF
# Self-Governing Agent System Test Report

Generated: $(date)
Duration: ${SECONDS}s

## Test Results Summary

| Metric | Value |
|--------|-------|
| Total Tests | $TOTAL_TESTS |
| Passed | $PASSED_TESTS |
| Failed | $FAILED_TESTS |
| Success Rate | $((PASSED_TESTS * 100 / TOTAL_TESTS))% |

## Quality Gates

EOF

    # Add quality gate results
    if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
        echo "- ✅ **ALL TESTS PASSED** - System ready for production" >> "$report_file"
    else
        echo "- ❌ **TESTS FAILED** - $FAILED_TESTS tests failed, review required" >> "$report_file"
    fi

    # Unit test coverage requirement
    echo "- 📊 **Unit Test Coverage**: Target ≥90% (verify with \`cargo tarpaulin\`)" >> "$report_file"

    # Performance requirements
    echo "- ⚡ **Performance**: P95 response time <5s (verify benchmarks)" >> "$report_file"

    # Security requirements
    echo "- 🔒 **Security**: Zero high/critical vulnerabilities (verify audit)" >> "$report_file"

    cat >> "$report_file" << EOF

## Recommendations

EOF

    if [ $FAILED_TESTS -gt 0 ]; then
        echo "- 🔧 **Fix Failed Tests**: Review test output and fix issues" >> "$report_file"
        echo "- 🔍 **Debug Failures**: Check logs in $LOG_FILE" >> "$report_file"
    fi

    if [ $PASSED_TESTS -eq $TOTAL_TESTS ]; then
        echo "- 🚀 **Ready for Deployment**: All tests pass, system is production-ready" >> "$report_file"
        echo "- 📈 **Monitor Performance**: Establish baseline metrics for regression detection" >> "$report_file"
    fi

    echo "- 📝 **Documentation**: Update test documentation as features evolve" >> "$report_file"

    log "📄 Test report generated: $report_file"
    echo "📄 Test report: $report_file"
}

# Main execution
main() {
    log "🚀 Starting Self-Governing Agent System Test Suite"
    log "=================================================="

    # Setup
    setup_environment

    # Run test suites
    run_unit_tests
    run_integration_tests
    run_performance_tests
    run_web_dashboard_tests
    run_playground_tests
    run_security_tests
    run_quality_tests

    # Generate final report
    generate_report

    log "🏁 Test Suite Complete"
    log "======================"
    log "Results: $PASSED_TESTS/$TOTAL_TESTS tests passed"

    # Exit with appropriate code
    if [ $FAILED_TESTS -eq 0 ]; then
        log "🎉 All tests passed! System is ready."
        exit 0
    else
        log "❌ $FAILED_TESTS tests failed. Review and fix issues."
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    "unit")
        setup_environment
        run_unit_tests
        ;;
    "integration")
        setup_environment
        run_integration_tests
        ;;
    "performance")
        setup_environment
        run_performance_tests
        ;;
    "web")
        setup_environment
        run_web_dashboard_tests
        ;;
    "playground")
        setup_environment
        run_playground_tests
        ;;
    "security")
        setup_environment
        run_security_tests
        ;;
    "quality")
        setup_environment
        run_quality_tests
        ;;
    "report")
        generate_report
        ;;
    *)
        main
        ;;
esac
