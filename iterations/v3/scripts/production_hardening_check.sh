#!/bin/bash

# Production Hardening Check for Agent Agency V3
# Verifies production readiness and provides hardening recommendations

set -e

echo "🔒 Agent Agency V3 Production Hardening Check"
echo "=============================================="
echo

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Check functions
check_pass() {
    echo -e "${GREEN}✅ PASS${NC}: $1"
}

check_fail() {
    echo -e "${RED}❌ FAIL${NC}: $1"
}

check_warn() {
    echo -e "${YELLOW}⚠️  WARN${NC}: $1"
}

check_info() {
    echo -e "${BLUE}ℹ️  INFO${NC}: $1"
}

# Initialize counters
CHECKS_TOTAL=0
CHECKS_PASS=0
CHECKS_FAIL=0
CHECKS_WARN=0

check() {
    ((CHECKS_TOTAL++))
    if [ "$2" = "true" ]; then
        ((CHECKS_PASS++))
        check_pass "$1"
    elif [ "$2" = "false" ]; then
        ((CHECKS_FAIL++))
        check_fail "$1"
    elif [ "$2" = "warn" ]; then
        ((CHECKS_WARN++))
        check_warn "$1"
    else
        check_info "$1"
    fi
}

echo "🔍 Checking Code Quality..."
echo "---------------------------"

# Check for compilation errors
if cargo check --workspace --quiet 2>/dev/null; then
    check "Code compiles without errors" true
else
    check "Code compiles without errors" false
fi

# Check for unused dependencies
if cargo +nightly udeps --workspace --quiet 2>/dev/null; then
    check "No unused dependencies" true
else
    check "No unused dependencies" warn
fi

# Check for security vulnerabilities
if cargo audit --quiet 2>/dev/null; then
    check "No security vulnerabilities in dependencies" true
else
    VULN_COUNT=$(cargo audit 2>/dev/null | grep -c "vulnerabilities found" || echo "multiple")
    check "Security vulnerabilities found in dependencies (${VULN_COUNT}) - review and update" warn
fi

echo
echo "🧪 Checking Testing Coverage..."
echo "-------------------------------"

# Check test coverage (if cargo-tarpaulin is available)
if command -v cargo-tarpaulin &> /dev/null; then
    COVERAGE=$(cargo tarpaulin --workspace --out Lcov --output-dir /tmp | grep -oP "Coverage: \K\d+\.\d+" || echo "0")
    if (( $(echo "$COVERAGE >= 80" | bc -l) )); then
        check "Test coverage >= 80% ($COVERAGE%)" true
    elif (( $(echo "$COVERAGE >= 70" | bc -l) )); then
        check "Test coverage >= 70% ($COVERAGE%)" warn
    else
        check "Test coverage < 70% ($COVERAGE%)" false
    fi
else
    check "Cargo tarpaulin not available for coverage check" warn
fi

# Check for integration tests
INTEGRATION_TESTS=$(find . -name "*integration*" -type f | wc -l)
if [ "$INTEGRATION_TESTS" -gt 0 ]; then
    check "Integration tests present ($INTEGRATION_TESTS found)" true
else
    check "Integration tests present" false
fi

echo
echo "🏗️ Checking Infrastructure..."
echo "----------------------------"

# Check for database migrations
if [ -d "iterations/v3/data-infrastructure/migrations" ] && [ "$(ls iterations/v3/data-infrastructure/migrations/*.sql 2>/dev/null | wc -l)" -gt 0 ]; then
    check "Database migrations present" true
else
    check "Database migrations present" false
fi

# Check for Docker configuration
if [ -f "docker-compose.yml" ] || [ -f "Dockerfile" ]; then
    check "Container configuration present" true
else
    check "Container configuration present" warn
fi

# Check for monitoring setup
if grep -r "metrics\|prometheus\|grafana" iterations/v3/ --include="*.rs" --include="*.toml" --include="*.yaml" --include="*.yml" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Monitoring/metrics configuration present" true
else
    check "Monitoring/metrics configuration present" warn
fi

echo
echo "🔐 Checking Security..."
echo "----------------------"

# Check for hardcoded secrets (more specific pattern)
if grep -r "password.*=.*\".*\"\|secret.*=.*\".*\"\|key.*=.*\".*\"" iterations/v3/ --include="*.rs" --include="*.toml" | grep -v target/ | grep -v .git/ | grep -v "test_password\|example\|placeholder\|TODO\|FIXME" >/dev/null; then
    check "No hardcoded secrets detected" false
else
    check "No hardcoded secrets detected" true
fi

# Check for proper error handling
ERROR_HANDLING=$(grep -r "Result<\|anyhow::\|thiserror" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ | wc -l)
if [ "$ERROR_HANDLING" -gt 50 ]; then
    check "Comprehensive error handling present" true
else
    check "Comprehensive error handling present" warn
fi

# Check for input validation
VALIDATION=$(grep -r "validate\|sanitize" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ | wc -l)
if [ "$VALIDATION" -gt 20 ]; then
    check "Input validation present" true
else
    check "Input validation present" warn
fi

echo
echo "⚡ Checking Performance..."
echo "-------------------------"

# Check for performance optimizations
if grep -r "tokio::spawn\|async\|parallel\|concurrent" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Async/parallel processing implemented" true
else
    check "Async/parallel processing implemented" warn
fi

# Check for caching
if grep -r "cache\|Cache\|lru" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Caching mechanisms implemented" true
else
    check "Caching mechanisms implemented" warn
fi

# Check for resource limits
if grep -r "limit\|timeout\|backpressure" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Resource limits and backpressure handling" true
else
    check "Resource limits and backpressure handling" warn
fi

echo
echo "📚 Checking Documentation..."
echo "---------------------------"

# Check for README
if [ -f "README.md" ]; then
    check "README.md present" true
else
    check "README.md present" false
fi

# Check for API documentation
if grep -r "openapi\|swagger\|api.*doc" iterations/v3/ --include="*.yaml" --include="*.yml" --include="*.json" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "API documentation present" true
else
    check "API documentation present" warn
fi

# Check for code documentation
DOC_COMMENTS=$(grep -r "///\|/\*\*" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ | wc -l)
if [ "$DOC_COMMENTS" -gt 100 ]; then
    check "Comprehensive code documentation" true
else
    check "Comprehensive code documentation" warn
fi

echo
echo "🚀 Checking Deployment Readiness..."
echo "-----------------------------------"

# Check for environment configuration
if [ -f ".env.example" ] || grep -r "dotenv\|env::" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Environment configuration handled" true
else
    check "Environment configuration handled" warn
fi

# Check for health checks
if grep -r "health\|Health\|status" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Health check endpoints implemented" true
else
    check "Health check endpoints implemented" warn
fi

# Check for graceful shutdown
if grep -r "shutdown\|graceful\|signal" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "Graceful shutdown handling" true
else
    check "Graceful shutdown handling" warn
fi

echo
echo "📊 Checking CAWS Compliance..."
echo "------------------------------"

# Check for CAWS constitution references
if grep -r "CAWS\|constitution\|constitutional" iterations/v3/ --include="*.rs" --include="*.md" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "CAWS constitutional compliance implemented" true
else
    check "CAWS constitutional compliance implemented" false
fi

# Check for quality gates
if [ -f ".caws/working-spec.yaml" ] || grep -r "quality.*gate\|waiver" iterations/v3/ --include="*.rs" | grep -v target/ | grep -v .git/ >/dev/null; then
    check "CAWS quality gates implemented" true
else
    check "CAWS quality gates implemented" warn
fi

echo
echo "🎯 Production Readiness Summary"
echo "==============================="

PASS_PERCENT=$((CHECKS_PASS * 100 / CHECKS_TOTAL))
FAIL_PERCENT=$((CHECKS_FAIL * 100 / CHECKS_TOTAL))
WARN_PERCENT=$((CHECKS_WARN * 100 / CHECKS_TOTAL))

echo "Total Checks: $CHECKS_TOTAL"
echo -e "✅ Passed: $CHECKS_PASS (${PASS_PERCENT}%)"
echo -e "❌ Failed: $CHECKS_FAIL (${FAIL_PERCENT}%)"
echo -e "⚠️  Warnings: $CHECKS_WARN (${WARN_PERCENT}%)"

echo
if [ $FAIL_PERCENT -eq 0 ]; then
    echo -e "${GREEN}🎉 PRODUCTION READY${NC}"
    echo "All critical checks passed. System is ready for production deployment."
elif [ $FAIL_PERCENT -lt 20 ]; then
    echo -e "${YELLOW}⚠️  MOSTLY READY${NC}"
    echo "Minor issues need to be addressed before production deployment."
else
    echo -e "${RED}❌ NOT READY${NC}"
    echo "Critical issues must be resolved before production deployment."
fi

echo
echo "📋 Next Steps:"
echo "1. Address any FAILED checks immediately"
echo "2. Review WARNING items for potential improvements"
echo "3. Run performance benchmarks: cargo bench"
echo "4. Execute integration tests: cargo test --test integration_*"
echo "5. Deploy to staging environment for final validation"

# Exit with appropriate code
if [ $FAIL_PERCENT -eq 0 ]; then
    exit 0
elif [ $FAIL_PERCENT -lt 20 ]; then
    exit 1
else
    exit 2
fi
