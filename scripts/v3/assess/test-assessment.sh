#!/usr/bin/env bash
# Test Assessment Module for V3 Readiness Framework
# Runs and analyzes all test suites (unit, integration, mutation)
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

# Load config values (simple YAML parsing)
get_config() {
    local key="$1"
    grep "^${key}:" "$CONFIG_FILE" | cut -d: -f2 | tr -d ' "'
}

WORKSPACE_TEST_CMD=$(grep -A 5 "test:" "$CONFIG_FILE" | grep "workspace_test_command:" | cut -d: -f2- | sed 's/^ *"//' | sed 's/" *$//')
MUTATION_ENABLED=$(grep -A 3 "test:" "$CONFIG_FILE" | grep "mutation_test_enabled:" | cut -d: -f2 | tr -d ' ')
MUTATION_TIMEOUT=$(grep -A 4 "test:" "$CONFIG_FILE" | grep "mutation_timeout:" | cut -d: -f2 | tr -d ' ')

cd "$V3_DIR"

echo -e "${BLUE}[test-assessment] Starting test assessment...${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"

# Initialize results JSON
RESULTS_FILE="$OUTPUT_DIR/test-results.json"
cat > "$RESULTS_FILE" <<EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "unit_tests": {
    "total": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "duration_seconds": 0,
    "failures": []
  },
  "integration_tests": {
    "total": 0,
    "passed": 0,
    "failed": 0,
    "ignored": 0,
    "duration_seconds": 0,
    "failures": []
  },
  "mutation_tests": {
    "enabled": false,
    "score": null,
    "mutants_killed": null,
    "mutants_survived": null,
    "duration_seconds": 0
  },
  "crates": {}
}
EOF

# Function to parse cargo test output
parse_test_output() {
    local output_file="$1"
    local test_type="$2"
    
    # Extract test counts
    local total=$(grep -oE "test result:.*" "$output_file" | grep -oE "[0-9]+ test" | grep -oE "[0-9]+" | head -1 || echo "0")
    local passed=$(grep -oE "test result:.*ok" "$output_file" | grep -oE "[0-9]+ passed" | grep -oE "[0-9]+" || echo "0")
    local failed=$(grep -oE "test result:.*" "$output_file" | grep -oE "[0-9]+ failed" | grep -oE "[0-9]+" || echo "0")
    local ignored=$(grep -oE "test result:.*" "$output_file" | grep -oE "[0-9]+ ignored" | grep -oE "[0-9]+" || echo "0")
    
    # Extract duration
    local duration=$(grep -oE "finished in [0-9.]+s" "$output_file" | grep -oE "[0-9.]+" || echo "0")
    
    # Extract failing test names
    local failures=()
    while IFS= read -r line; do
        if [[ "$line" =~ test\ ([a-zA-Z0-9_:]+)\ \.\.\.\ FAILED ]]; then
            failures+=("${BASH_REMATCH[1]}")
        fi
    done < "$output_file"
    
    # Update JSON
    local tmp_file=$(mktemp)
    jq \
        --arg type "$test_type" \
        --argjson total "${total:-0}" \
        --argjson passed "${passed:-0}" \
        --argjson failed "${failed:-0}" \
        --argjson ignored "${ignored:-0}" \
        --argjson duration "${duration:-0}" \
        --argjson failures "$(printf '%s\n' "${failures[@]}" | jq -R . | jq -s .)" \
        '. + {
            ($type): {
                total: $total,
                passed: $passed,
                failed: $failed,
                ignored: $ignored,
                duration_seconds: $duration,
                failures: $failures
            }
        }' "$RESULTS_FILE" > "$tmp_file"
    mv "$tmp_file" "$RESULTS_FILE"
}

# Run unit tests
echo -e "${BLUE}[test-assessment] Running unit tests...${NC}"
UNIT_TEST_OUTPUT="$OUTPUT_DIR/unit-tests.log"
START_TIME=$(date +%s)

if $WORKSPACE_TEST_CMD --lib 2>&1 | tee "$UNIT_TEST_OUTPUT"; then
    UNIT_EXIT_CODE=0
else
    UNIT_EXIT_CODE=$?
fi

END_TIME=$(date +%s)
UNIT_DURATION=$((END_TIME - START_TIME))

parse_test_output "$UNIT_TEST_OUTPUT" "unit_tests"

# Update duration in JSON
tmp_file=$(mktemp)
jq --argjson duration "$UNIT_DURATION" '.unit_tests.duration_seconds = $duration' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

if [ $UNIT_EXIT_CODE -ne 0 ]; then
    echo -e "${RED}[test-assessment] Unit tests failed${NC}"
else
    echo -e "${GREEN}[test-assessment] Unit tests completed${NC}"
fi

# Run integration tests
echo -e "${BLUE}[test-assessment] Running integration tests...${NC}"
INTEGRATION_TEST_OUTPUT="$OUTPUT_DIR/integration-tests.log"
START_TIME=$(date +%s)

if $WORKSPACE_TEST_CMD --test '*' 2>&1 | tee "$INTEGRATION_TEST_OUTPUT"; then
    INTEGRATION_EXIT_CODE=0
else
    INTEGRATION_EXIT_CODE=$?
fi

END_TIME=$(date +%s)
INTEGRATION_DURATION=$((END_TIME - START_TIME))

parse_test_output "$INTEGRATION_TEST_OUTPUT" "integration_tests"

# Update duration in JSON
tmp_file=$(mktemp)
jq --argjson duration "$INTEGRATION_DURATION" '.integration_tests.duration_seconds = $duration' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

if [ $INTEGRATION_EXIT_CODE -ne 0 ]; then
    echo -e "${RED}[test-assessment] Integration tests failed${NC}"
else
    echo -e "${GREEN}[test-assessment] Integration tests completed${NC}"
fi

# Parse crate-level test results
echo -e "${BLUE}[test-assessment] Analyzing crate-level test results...${NC}"
CRATE_RESULTS=$(mktemp)

# Extract crate test results from output
grep -E "^\s+Running.*target" "$UNIT_TEST_OUTPUT" "$INTEGRATION_TEST_OUTPUT" 2>/dev/null | while read -r line; do
    if [[ "$line" =~ Running\ (.*)\ target ]]; then
        crate="${BASH_REMATCH[1]}"
        # Extract crate name from path
        crate_name=$(basename "$crate" | sed 's/-/_/g')
        echo "$crate_name"
    fi
done | sort -u | while read -r crate_name; do
    # Count tests per crate (simplified - would need more sophisticated parsing)
    passed=$(grep -c "test $crate_name::.* ... ok" "$UNIT_TEST_OUTPUT" "$INTEGRATION_TEST_OUTPUT" 2>/dev/null || echo "0")
    failed=$(grep -c "test $crate_name::.* ... FAILED" "$UNIT_TEST_OUTPUT" "$INTEGRATION_TEST_OUTPUT" 2>/dev/null || echo "0")
    
    tmp_file=$(mktemp)
    jq \
        --arg crate "$crate_name" \
        --argjson passed "$passed" \
        --argjson failed "$failed" \
        '.crates[$crate] = {passed: $passed, failed: $failed}' \
        "$RESULTS_FILE" > "$tmp_file"
    mv "$tmp_file" "$RESULTS_FILE"
done

# Check mutation testing if enabled
if [ "$MUTATION_ENABLED" = "true" ]; then
    echo -e "${BLUE}[test-assessment] Running mutation tests...${NC}"
    
    if command -v cargo-mutants &> /dev/null; then
        MUTATION_OUTPUT="$OUTPUT_DIR/mutation-tests.log"
        START_TIME=$(date +%s)
        
        if timeout "$MUTATION_TIMEOUT" cargo-mutants --workspace --timeout "$MUTATION_TIMEOUT" --no-shuffle --baseline run 2>&1 | tee "$MUTATION_OUTPUT"; then
            MUTATION_EXIT_CODE=0
        else
            MUTATION_EXIT_CODE=$?
        fi
        
        END_TIME=$(date +%s)
        MUTATION_DURATION=$((END_TIME - START_TIME))
        
        # Extract mutation score
        MUTATION_SCORE=$(grep -oE "score: [0-9]+\.[0-9]+" "$MUTATION_OUTPUT" | grep -oE "[0-9]+\.[0-9]+" | tail -1 || echo "null")
        MUTANTS_KILLED=$(grep -oE "[0-9]+ mutants killed" "$MUTATION_OUTPUT" | grep -oE "[0-9]+" || echo "null")
        MUTANTS_SURVIVED=$(grep -oE "[0-9]+ mutants survived" "$MUTATION_OUTPUT" | grep -oE "[0-9]+" || echo "null")
        
        # Update JSON
        tmp_file=$(mktemp)
        jq \
            --argjson enabled true \
            --argjson score "${MUTATION_SCORE:-null}" \
            --argjson killed "${MUTANTS_KILLED:-null}" \
            --argjson survived "${MUTANTS_SURVIVED:-null}" \
            --argjson duration "$MUTATION_DURATION" \
            '.mutation_tests = {
                enabled: $enabled,
                score: $score,
                mutants_killed: $killed,
                mutants_survived: $survived,
                duration_seconds: $duration
            }' "$RESULTS_FILE" > "$tmp_file"
        mv "$tmp_file" "$RESULTS_FILE"
        
        if [ "$MUTATION_EXIT_CODE" -eq 0 ]; then
            echo -e "${GREEN}[test-assessment] Mutation tests completed (score: ${MUTATION_SCORE})${NC}"
        else
            echo -e "${YELLOW}[test-assessment] Mutation tests completed with warnings${NC}"
        fi
    else
        echo -e "${YELLOW}[test-assessment] cargo-mutants not available, skipping mutation tests${NC}"
        tmp_file=$(mktemp)
        jq '.mutation_tests.enabled = false' "$RESULTS_FILE" > "$tmp_file"
        mv "$tmp_file" "$RESULTS_FILE"
    fi
else
    echo -e "${BLUE}[test-assessment] Mutation testing disabled in config${NC}"
fi

# Print summary
echo -e "${BLUE}[test-assessment] Test Assessment Summary:${NC}"
jq -r '
    "Unit Tests: \(.unit_tests.passed) passed, \(.unit_tests.failed) failed, \(.unit_tests.ignored) ignored",
    "Integration Tests: \(.integration_tests.passed) passed, \(.integration_tests.failed) failed, \(.integration_tests.ignored) ignored",
    (.mutation_tests.enabled // false | if . then "Mutation Tests: score=\(.mutation_tests.score // "N/A")" else "Mutation Tests: disabled" end)
' "$RESULTS_FILE"

echo -e "${GREEN}[test-assessment] Results saved to $RESULTS_FILE${NC}"

