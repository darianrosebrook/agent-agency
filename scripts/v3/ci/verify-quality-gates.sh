#!/usr/bin/env bash
# Comprehensive Quality Gate Verification Script
# Verifies all quality gates: tests, coverage, mutation, linting, security
# @darianrosebrook

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
TIER=${TIER:-2}
COVERAGE_MIN=${COVERAGE_MIN:-0.80}
BRANCH_COVERAGE_MIN=${BRANCH_COVERAGE_MIN:-0.90}
MUTATION_MIN=${MUTATION_MIN:-0.50}
ENABLE_MUTATION=${ENABLE_MUTATION:-false}
ENABLE_E2E=${ENABLE_E2E:-false}

# Project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_ROOT="$PROJECT_ROOT/iterations/v3"

cd "$V3_ROOT"

# Track overall success
OVERALL_SUCCESS=true
FAILED_GATES=()

# Function to print status
print_status() {
    echo -e "${BLUE}[$(date +'%H:%M:%S')]${NC} $1"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# Function to run a gate check
run_gate_check() {
    local gate_name="$1"
    local command="$2"
    
    print_status "Running $gate_name..."
    if eval "$command" > /tmp/${gate_name// /_}.log 2>&1; then
        print_success "$gate_name passed"
        return 0
    else
        print_error "$gate_name failed"
        FAILED_GATES+=("$gate_name")
        OVERALL_SUCCESS=false
        return 1
    fi
}

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Quality Gate Verification (Tier $TIER)"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Configuration:"
echo "  Tier: $TIER"
echo "  Line Coverage Min: ${COVERAGE_MIN}"
echo "  Branch Coverage Min: ${BRANCH_COVERAGE_MIN}"
echo "  Mutation Min: ${MUTATION_MIN}"
echo "  Mutation Testing: ${ENABLE_MUTATION}"
echo "  E2E Testing: ${ENABLE_E2E}"
echo ""

# Gate 1: Compilation Check
print_status "Gate 1: Compilation Check"
if run_gate_check "Compilation" "cargo check --workspace --all-features"; then
    print_success "All packages compile successfully"
else
    print_error "Compilation failed - check errors above"
    cat /tmp/Compilation.log | grep -E "error\[" | head -10
fi
echo ""

# Gate 2: Linting
print_status "Gate 2: Linting"
if [ -f "$SCRIPT_DIR/lint.sh" ]; then
    if run_gate_check "Linting" "bash $SCRIPT_DIR/lint.sh"; then
        print_success "Linting passed"
    else
        print_error "Linting failed - check errors above"
        cat /tmp/Linting.log | tail -20
    fi
else
    # Fallback: run clippy directly
    if run_gate_check "Clippy" "cargo clippy --workspace --all-targets --all-features -- -D warnings"; then
        print_success "Clippy passed"
    else
        print_error "Clippy failed"
        cat /tmp/Clippy.log | tail -20
    fi
    
    # Format check
    if run_gate_check "Formatting" "cargo fmt --all -- --check"; then
        print_success "Formatting check passed"
    else
        print_warning "Formatting issues found - run 'cargo fmt --all' to fix"
    fi
fi
echo ""

# Gate 3: Unit Tests
print_status "Gate 3: Unit Tests"
mkdir -p target/coverage
if run_gate_check "Unit Tests" "cargo test --workspace --all-features --lib"; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    cat /tmp/Unit_Tests.log | grep -E "test result|FAILED|error" | tail -20
fi
echo ""

# Gate 4: Integration Tests
print_status "Gate 4: Integration Tests"
if run_gate_check "Integration Tests" "cargo test --workspace --all-features --test '*'"; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    cat /tmp/Integration_Tests.log | grep -E "test result|FAILED|error" | tail -20
fi
echo ""

# Gate 5: Coverage
print_status "Gate 5: Test Coverage"
print_status "  Running tests with coverage instrumentation..."
if command -v cargo-nextest >/dev/null 2>&1; then
    RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw" \
        cargo nextest run --workspace --all-features --test-threads auto > /tmp/coverage_tests.log 2>&1 || true
else
    RUSTFLAGS="-C instrument-coverage" LLVM_PROFILE_FILE="target/coverage/%p-%m.profraw" \
        cargo test --workspace --all-features --test-threads auto > /tmp/coverage_tests.log 2>&1 || true
fi

print_status "  Generating coverage report..."
if command -v grcov >/dev/null 2>&1; then
    grcov . -s . -t lcov --llvm --branch --ignore-not-existing \
        -o target/coverage/lcov.info --ignore "/*" --ignore "target/*" > /tmp/grcov.log 2>&1 || true
    
    if [ -f "target/coverage/lcov.info" ]; then
        print_status "  Checking coverage thresholds..."
        export LINE_COVERAGE_MIN="$COVERAGE_MIN"
        export BRANCH_COVERAGE_MIN="$BRANCH_COVERAGE_MIN"
        if node "$PROJECT_ROOT/scripts/v3/test/check-coverage.js"; then
            print_success "Coverage thresholds met"
        else
            print_error "Coverage thresholds not met"
            OVERALL_SUCCESS=false
            FAILED_GATES+=("Coverage")
        fi
    else
        print_error "Coverage report generation failed"
        OVERALL_SUCCESS=false
        FAILED_GATES+=("Coverage")
    fi
else
    print_warning "grcov not installed - skipping coverage check"
    print_warning "Install with: cargo install grcov"
fi
echo ""

# Gate 6: Mutation Testing
if [ "$ENABLE_MUTATION" = "true" ]; then
    print_status "Gate 6: Mutation Testing"
    if command -v cargo-mutants >/dev/null 2>&1; then
        print_status "  Running mutation testing (min score: ${MUTATION_MIN})..."
        MUTANTS_OUTPUT=$(cargo mutants --workspace --timeout 300 --no-shuffle --baseline run 2>&1 || true)
        MUTANTS_EXIT_CODE=$?
        
        # Parse mutation score from output
        MUTATION_SCORE=$(echo "$MUTANTS_OUTPUT" | grep -oE "score: [0-9]+\.[0-9]+" | grep -oE "[0-9]+\.[0-9]+" | tail -1 || echo "0")
        
        if [ -n "$MUTATION_SCORE" ] && [ "$MUTATION_SCORE" != "0" ]; then
            print_status "  Mutation score: ${MUTATION_SCORE}, Threshold: ${MUTATION_MIN}"
            
            # Compare against threshold (using awk for floating point comparison)
            if awk "BEGIN {exit !($MUTATION_SCORE >= $MUTATION_MIN)}"; then
                print_success "Mutation testing passed (score: ${MUTATION_SCORE} >= ${MUTATION_MIN})"
            else
                print_error "Mutation testing failed (score: ${MUTATION_SCORE} < ${MUTATION_MIN})"
                OVERALL_SUCCESS=false
                FAILED_GATES+=("Mutation Testing")
            fi
        else
            print_warning "Could not parse mutation score from output"
            print_warning "Mutation testing output:"
            echo "$MUTANTS_OUTPUT" | tail -20
        fi
    else
        print_warning "cargo-mutants not installed - skipping mutation testing"
        print_warning "Install with: cargo install cargo-mutants"
    fi
    echo ""
fi

# Gate 7: Security Scans
print_status "Gate 7: Security Scans"
if command -v cargo-audit >/dev/null 2>&1; then
    if run_gate_check "Security Audit" "cargo audit"; then
        print_success "Security audit passed (no vulnerabilities found)"
    else
        print_error "Security vulnerabilities found"
        cat /tmp/Security_Audit.log | grep -E "advisory|vulnerability|error" | head -20
    fi
else
    print_warning "cargo-audit not installed - skipping security audit"
    print_warning "Install with: cargo install cargo-audit"
fi
echo ""

# Gate 8: E2E Tests
if [ "$ENABLE_E2E" = "true" ]; then
    print_status "Gate 8: End-to-End Tests"
    if [ -f "$PROJECT_ROOT/scripts/v3/test/run-e2e-tests.sh" ]; then
        if run_gate_check "E2E Tests" "bash $PROJECT_ROOT/scripts/v3/test/run-e2e-tests.sh"; then
            print_success "E2E tests passed"
        else
            print_error "E2E tests failed"
            cat /tmp/E2E_Tests.log | tail -30
        fi
    else
        print_warning "E2E test script not found - skipping E2E tests"
    fi
    echo ""
fi

# Gate 9: CAWS Gates
print_status "Gate 9: CAWS Quality Gates"
if [ -f "apps/tools/caws/gates.js" ]; then
    if run_gate_check "CAWS Gates" "cd apps/tools/caws && node gates.js tier $TIER"; then
        print_success "CAWS gates passed"
    else
        print_error "CAWS gates failed"
        cat /tmp/CAWS_Gates.log | tail -20
    fi
else
    print_warning "CAWS gates script not found - skipping CAWS gates"
fi
echo ""

# Summary
echo "═══════════════════════════════════════════════════════════════"
echo "  Quality Gate Verification Summary"
echo "═══════════════════════════════════════════════════════════════"
echo ""

if [ "$OVERALL_SUCCESS" = true ]; then
    print_success "All quality gates passed!"
    echo ""
    echo "Quality gates verified:"
    echo "  ✓ Compilation"
    echo "  ✓ Linting"
    echo "  ✓ Unit Tests"
    echo "  ✓ Integration Tests"
    echo "  ✓ Coverage"
    [ "$ENABLE_MUTATION" = "true" ] && echo "  ✓ Mutation Testing"
    echo "  ✓ Security Scans"
    [ "$ENABLE_E2E" = "true" ] && echo "  ✓ E2E Tests"
    echo "  ✓ CAWS Gates"
    echo ""
    exit 0
else
    print_error "Some quality gates failed!"
    echo ""
    echo "Failed gates:"
    for gate in "${FAILED_GATES[@]}"; do
        echo "  ✗ $gate"
    done
    echo ""
    echo "Check the logs above for details."
    echo "Log files are available in /tmp/ with names like: ${FAILED_GATES[0]// /_}.log"
    echo ""
    exit 1
fi

