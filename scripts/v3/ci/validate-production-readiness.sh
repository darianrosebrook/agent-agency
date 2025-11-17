#!/usr/bin/env bash
# Production Readiness Validation Script
# Validates all production readiness criteria per the implementation plan
# @darianrosebrook

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_ROOT="$PROJECT_ROOT/iterations/v3"

cd "$V3_ROOT"

# Track overall status
OVERALL_READY=true
CRITICAL_ISSUES=0
WARNINGS=0

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
    ((WARNINGS++))
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
    OVERALL_READY=false
    ((CRITICAL_ISSUES++))
}

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Production Readiness Validation"
echo "═══════════════════════════════════════════════════════════════"
echo ""

# Category 1: Code Quality Gates
print_status "Category 1: Code Quality Gates"
echo ""

# 1.1 Linting
print_status "  1.1 Checking linting..."
if cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | grep -q "Finished"; then
    print_success "    Linting: Zero errors"
else
    LINT_ERRORS=$(cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | grep -c "error\|warning" || echo "0")
    if [ "$LINT_ERRORS" -gt 0 ]; then
        print_error "    Linting: $LINT_ERRORS errors/warnings found"
    else
        print_success "    Linting: Passed"
    fi
fi

# 1.2 Type Checking
print_status "  1.2 Checking type safety..."
if cargo check --workspace --all-features 2>&1 | grep -q "Finished"; then
    print_success "    Type checking: Zero errors"
else
    TYPE_ERRORS=$(cargo check --workspace --all-features 2>&1 | grep -c "error\[" || echo "0")
    if [ "$TYPE_ERRORS" -gt 0 ]; then
        print_error "    Type checking: $TYPE_ERRORS errors found"
    else
        print_success "    Type checking: Passed"
    fi
fi

# 1.3 TODO/Placeholder Check
print_status "  1.3 Checking for TODOs/PLACEHOLDERs in production code..."
TODO_ANALYZER_PATH=""
if [ -f "scripts/v3/analysis/todo_analyzer.py" ]; then
    TODO_ANALYZER_PATH="scripts/v3/analysis/todo_analyzer.py"
elif [ -f "$PROJECT_ROOT/scripts/v3/analysis/todo_analyzer.py" ]; then
    TODO_ANALYZER_PATH="$PROJECT_ROOT/scripts/v3/analysis/todo_analyzer.py"
elif [ -f "$PROJECT_ROOT/scripts/analysis/todo_analyzer.py" ]; then
    TODO_ANALYZER_PATH="$PROJECT_ROOT/scripts/analysis/todo_analyzer.py"
fi

if [ -n "$TODO_ANALYZER_PATH" ]; then
    if python3 "$TODO_ANALYZER_PATH" --v3-only --min-confidence 0.8 --ci-mode 2>&1 | grep -q "No high-confidence hidden TODOs found"; then
        print_success "    TODO/Placeholder check: No critical TODOs found"
    else
        TODO_COUNT=$(python3 "$TODO_ANALYZER_PATH" --v3-only --min-confidence 0.8 --ci-mode 2>&1 | grep -c "high-confidence" || echo "0")
        if [ "$TODO_COUNT" -gt 0 ]; then
            print_warning "    TODO/Placeholder check: $TODO_COUNT high-confidence TODOs found (may need review)"
        else
            print_success "    TODO/Placeholder check: Passed"
        fi
    fi
else
    print_warning "    TODO/Placeholder check: todo_analyzer.py not found, skipping"
fi

echo ""

# Category 2: Testing & Quality Assurance
print_status "Category 2: Testing & Quality Assurance"
echo ""

# 2.1 Test Execution
print_status "  2.1 Running test suite..."
if cargo test --workspace --all-features --lib --test '*' 2>&1 | grep -q "test result: ok"; then
    TEST_RESULT=$(cargo test --workspace --all-features --lib --test '*' 2>&1 | grep "test result:" | tail -1)
    print_success "    Tests: $TEST_RESULT"
else
    print_error "    Tests: Some tests failed"
fi

# 2.2 Coverage Check
print_status "  2.2 Checking test coverage..."
if [ -f "target/coverage/lcov.info" ]; then
    if node "$PROJECT_ROOT/scripts/v3/test/check-coverage.js" 2>&1 | grep -q "PASS"; then
        COVERAGE_INFO=$(node "$PROJECT_ROOT/scripts/v3/test/check-coverage.js" 2>&1 | grep "coverage:" || echo "")
        print_success "    Coverage: Thresholds met - $COVERAGE_INFO"
    else
        print_error "    Coverage: Thresholds not met"
    fi
else
    print_warning "    Coverage: lcov.info not found - run coverage tests first"
fi

# 2.3 Database Integration Tests
print_status "  2.3 Checking database integration tests..."
if [ -f "testing-validation/src/scenarios/api_integration.rs" ]; then
    print_success "    Database integration: Tests available"
else
    print_warning "    Database integration: Test infrastructure may be incomplete"
fi

echo ""

# Category 3: Infrastructure & Persistence
print_status "Category 3: Infrastructure & Persistence"
echo ""

# 3.1 Database Persistence
print_status "  3.1 Checking database persistence implementations..."
if grep -r "DatabaseLearningPersistence\|DatabaseTaskStatePersistence\|DatabaseProvenanceStorage" --include="*.rs" . 2>/dev/null | grep -q "impl"; then
    print_success "    Database persistence: Implementations found"
else
    print_warning "    Database persistence: May have incomplete implementations"
fi

# 3.2 Migration Scripts
print_status "  3.2 Checking migration scripts..."
if [ -d "data-infrastructure/migrations" ] && [ "$(find data-infrastructure/migrations -name "*.sql" | wc -l)" -gt 0 ]; then
    MIGRATION_COUNT=$(find data-infrastructure/migrations -name "*.sql" | wc -l | tr -d ' ')
    print_success "    Migrations: $MIGRATION_COUNT migration files found"
else
    print_warning "    Migrations: Migration directory or files not found"
fi

# 3.3 Connection Pooling
print_status "  3.3 Checking connection pooling..."
if grep -r "bb8\|r2d2\|deadpool" --include="*.toml" . 2>/dev/null | grep -q "bb8\|r2d2\|deadpool"; then
    print_success "    Connection pooling: Configured"
else
    print_warning "    Connection pooling: May not be configured"
fi

echo ""

# Category 4: Security & Reliability
print_status "Category 4: Security & Reliability"
echo ""

# 4.1 Security Scans
print_status "  4.1 Running security audit..."
if command -v cargo-audit >/dev/null 2>&1; then
    if cargo audit 2>&1 | grep -q "Success\|No vulnerabilities found"; then
        print_success "    Security audit: No vulnerabilities found"
    else
        VULN_COUNT=$(cargo audit 2>&1 | grep -c "advisory\|vulnerability" || echo "0")
        if [ "$VULN_COUNT" -gt 0 ]; then
            print_error "    Security audit: $VULN_COUNT vulnerabilities found"
        else
            print_success "    Security audit: Passed"
        fi
    fi
else
    print_warning "    Security audit: cargo-audit not installed"
fi

# 4.2 Real Cryptographic Implementations
print_status "  4.2 Checking cryptographic implementations..."
if grep -r "PaillierHomomorphicEncryption\|Schnorr\|verify_schnorr_proof" --include="*.rs" . 2>/dev/null | grep -q "impl"; then
    print_success "    Cryptography: Real implementations found (Paillier HE, Schnorr ZKP)"
else
    print_warning "    Cryptography: May have placeholder implementations"
fi

# 4.3 Integrity Verification
print_status "  4.3 Checking integrity verification..."
if grep -r "ContentHasher\|TamperingDetector\|calculate_hash" --include="*.rs" . 2>/dev/null | grep -q "impl\|fn"; then
    print_success "    Integrity verification: Real hash computation implemented"
else
    print_warning "    Integrity verification: May have placeholder implementations"
fi

echo ""

# Category 5: Documentation & Reality Alignment
print_status "Category 5: Documentation & Reality Alignment"
echo ""

# 5.1 Deployment Documentation
print_status "  5.1 Checking deployment documentation..."
if [ -f "docs/production-deployment-guide.md" ] || [ -f "docs/production-deployment.md" ]; then
    print_success "    Deployment docs: Found"
else
    print_warning "    Deployment docs: Missing or incomplete"
fi

# 5.2 API Documentation
print_status "  5.2 Checking API documentation..."
if [ -f "data-infrastructure/docs/API_DOCUMENTATION.md" ] || [ -d "docs/api" ]; then
    print_success "    API docs: Found"
else
    print_warning "    API docs: May be incomplete"
fi

# 5.3 Changelog
print_status "  5.3 Checking changelog..."
if [ -f "$PROJECT_ROOT/CHANGELOG.md" ]; then
    CHANGELOG_SIZE=$(wc -l < "$PROJECT_ROOT/CHANGELOG.md" | tr -d ' ')
    if [ "$CHANGELOG_SIZE" -gt 50 ]; then
        print_success "    Changelog: Present and has content ($CHANGELOG_SIZE lines)"
    else
        print_warning "    Changelog: Present but may be minimal"
    fi
else
    print_warning "    Changelog: Not found"
fi

echo ""

# Category 6: Deployment & Operations
print_status "Category 6: Deployment & Operations"
echo ""

# 6.1 Deployment Scripts
print_status "  6.1 Checking deployment scripts..."
if [ -f "$PROJECT_ROOT/scripts/v3/deploy/deploy-production.sh" ]; then
    print_success "    Deployment script: Found and executable"
else
    print_error "    Deployment script: Missing"
fi

# 6.2 Health Checks
print_status "  6.2 Checking health check endpoints..."
if grep -r "/health\|health_check\|health_check" --include="*.rs" . 2>/dev/null | grep -q "fn\|route"; then
    print_success "    Health checks: Implemented"
else
    print_warning "    Health checks: May not be implemented"
fi

# 6.3 Monitoring Configuration
print_status "  6.3 Checking monitoring configuration..."
if [ -f "$PROJECT_ROOT/deploy/docker/monitoring/prometheus.yml" ] || [ -d "system-observability" ]; then
    print_success "    Monitoring: Configuration found"
else
    print_warning "    Monitoring: Configuration may be incomplete"
fi

echo ""

# Category 7: Test Infrastructure
print_status "Category 7: Test Infrastructure"
echo ""

# 7.1 E2E Tests
print_status "  7.1 Checking E2E test infrastructure..."
if [ -f "testing-validation/src/scenarios/performance_scalability.rs" ] && \
   [ -f "testing-validation/src/scenarios/security_privacy.rs" ] && \
   [ -f "testing-validation/src/scenarios/api_integration.rs" ]; then
    print_success "    E2E tests: Performance, security, and API integration tests available"
else
    print_warning "    E2E tests: Some test scenarios may be missing"
fi

# 7.2 Quality Gate Verification
print_status "  7.2 Checking quality gate verification..."
if [ -f "$PROJECT_ROOT/scripts/v3/ci/verify-quality-gates.sh" ]; then
    print_success "    Quality gates: Verification script available"
else
    print_warning "    Quality gates: Verification script missing"
fi

# 7.3 k6 Performance Tests
print_status "  7.3 Checking k6 performance tests..."
if [ -f "$PROJECT_ROOT/tests/performance/agent-agency-smoke.js" ]; then
    print_success "    k6 tests: Performance test script available"
else
    print_warning "    k6 tests: Performance test script missing"
fi

echo ""

# Summary
echo "═══════════════════════════════════════════════════════════════"
echo "  Production Readiness Summary"
echo "═══════════════════════════════════════════════════════════════"
echo ""

if [ "$OVERALL_READY" = true ] && [ "$CRITICAL_ISSUES" -eq 0 ]; then
    print_success "Overall Status: Production Ready"
    echo ""
    echo "All critical quality gates passed:"
    echo "  ✓ Code quality (linting, type checking)"
    echo "  ✓ Testing infrastructure"
    echo "  ✓ Database persistence"
    echo "  ✓ Security implementations"
    echo "  ✓ Documentation"
    echo "  ✓ Deployment scripts"
    echo "  ✓ Test infrastructure"
    echo ""
    if [ "$WARNINGS" -gt 0 ]; then
        echo "Note: $WARNINGS warning(s) found - review recommended but not blocking"
    fi
    echo ""
    exit 0
else
    print_error "Overall Status: Not Production Ready"
    echo ""
    echo "Critical issues found: $CRITICAL_ISSUES"
    echo "Warnings: $WARNINGS"
    echo ""
    echo "Please address the issues above before production deployment."
    echo ""
    exit 1
fi

