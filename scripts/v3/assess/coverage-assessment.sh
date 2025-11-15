#!/usr/bin/env bash
# Coverage Assessment Module for V3 Readiness Framework
# Generates coverage reports and analyzes against thresholds
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

LINE_THRESHOLD=$(grep -A 2 "coverage_thresholds:" "$CONFIG_FILE" | grep "line:" | cut -d: -f2 | tr -d ' ')
BRANCH_THRESHOLD=$(grep -A 2 "coverage_thresholds:" "$CONFIG_FILE" | grep "branch:" | cut -d: -f2 | tr -d ' ')
INSTRUMENTATION_FLAGS=$(grep -A 1 "coverage_generation:" "$CONFIG_FILE" | grep "instrumentation_flags:" | cut -d: -f2 | tr -d ' "')
PROFILE_PATTERN=$(grep -A 2 "coverage_generation:" "$CONFIG_FILE" | grep "profile_file_pattern:" | cut -d: -f2 | tr -d ' "')
GROCV_OUTPUT=$(grep -A 3 "coverage_generation:" "$CONFIG_FILE" | grep "grcov_output:" | cut -d: -f2 | tr -d ' "')

cd "$V3_DIR"

echo -e "${BLUE}[coverage-assessment] Starting coverage assessment...${NC}"

# Create output directory
mkdir -p "$OUTPUT_DIR"
mkdir -p "$(dirname "$GROCV_OUTPUT")"

# Initialize results JSON
RESULTS_FILE="$OUTPUT_DIR/coverage-results.json"
cat > "$RESULTS_FILE" <<EOF
{
  "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "thresholds": {
    "line": $LINE_THRESHOLD,
    "branch": $BRANCH_THRESHOLD
  },
  "overall": {
    "line_coverage": 0.0,
    "branch_coverage": 0.0,
    "lines_covered": 0,
    "lines_total": 0,
    "branches_covered": 0,
    "branches_total": 0
  },
  "crates": {},
  "below_threshold": [],
  "high_value_areas": []
}
EOF

# Check if grcov is available
if ! command -v grcov &> /dev/null; then
    echo -e "${YELLOW}[coverage-assessment] grcov not found. Installing...${NC}"
    cargo install grcov || {
        echo -e "${RED}[coverage-assessment] Failed to install grcov${NC}"
        exit 1
    }
fi

# Clean previous coverage data
echo -e "${BLUE}[coverage-assessment] Cleaning previous coverage data...${NC}"
rm -rf target/coverage/*.profraw 2>/dev/null || true

# Run tests with coverage instrumentation
echo -e "${BLUE}[coverage-assessment] Running tests with coverage instrumentation...${NC}"
COVERAGE_TEST_OUTPUT="$OUTPUT_DIR/coverage-tests.log"

export RUSTFLAGS="$INSTRUMENTATION_FLAGS"
export LLVM_PROFILE_FILE="$PROFILE_PATTERN"

if cargo test --workspace --all-features 2>&1 | tee "$COVERAGE_TEST_OUTPUT"; then
    TEST_EXIT_CODE=0
else
    TEST_EXIT_CODE=$?
fi

unset RUSTFLAGS
unset LLVM_PROFILE_FILE

if [ $TEST_EXIT_CODE -ne 0 ]; then
    echo -e "${YELLOW}[coverage-assessment] Some tests failed, but continuing with coverage analysis...${NC}"
fi

# Generate lcov report
echo -e "${BLUE}[coverage-assessment] Generating lcov coverage report...${NC}"
GROCV_IGNORE=$(get_config "grcov_ignore_patterns" | tr '\n' ' ' | sed 's/^/--ignore /g')

if grcov . -s . -t lcov --llvm --branch --ignore-not-existing \
    -o "$GROCV_OUTPUT" \
    --ignore "/*" \
    --ignore "target/*" \
    --ignore "tests/*" \
    --ignore "**/tests/*" 2>&1 | tee "$OUTPUT_DIR/grcov.log"; then
    echo -e "${GREEN}[coverage-assessment] Coverage report generated${NC}"
else
    echo -e "${RED}[coverage-assessment] Failed to generate coverage report${NC}"
    exit 1
fi

# Parse lcov file to extract coverage data
if [ ! -f "$GROCV_OUTPUT" ]; then
    echo -e "${RED}[coverage-assessment] Coverage file not found: $GROCV_OUTPUT${NC}"
    exit 1
fi

echo -e "${BLUE}[coverage-assessment] Parsing coverage data...${NC}"

# Parse lcov format
LINES_COVERED=0
LINES_TOTAL=0
BRANCHES_COVERED=0
BRANCHES_TOTAL=0

# Track crate-level coverage
declare -A CRATE_LINES_COVERED
declare -A CRATE_LINES_TOTAL
declare -A CRATE_BRANCHES_COVERED
declare -A CRATE_BRANCHES_TOTAL

CURRENT_FILE=""
CURRENT_CRATE=""

while IFS= read -r line; do
    # SF: Source file
    if [[ "$line" =~ ^SF:(.+) ]]; then
        CURRENT_FILE="${BASH_REMATCH[1]}"
        # Extract crate name from path
        if [[ "$CURRENT_FILE" =~ iterations/v3/([^/]+) ]]; then
            CURRENT_CRATE="${BASH_REMATCH[1]}"
        fi
    fi
    # DA: Line data (line_number,execution_count)
    if [[ "$line" =~ ^DA:([0-9]+),([0-9]+) ]]; then
        LINES_TOTAL=$((LINES_TOTAL + 1))
        if [ -n "$CURRENT_CRATE" ]; then
            CRATE_LINES_TOTAL["$CURRENT_CRATE"]=$((${CRATE_LINES_TOTAL["$CURRENT_CRATE"]:-0} + 1))
        fi
        if [ "${BASH_REMATCH[2]}" -gt 0 ]; then
            LINES_COVERED=$((LINES_COVERED + 1))
            if [ -n "$CURRENT_CRATE" ]; then
                CRATE_LINES_COVERED["$CURRENT_CRATE"]=$((${CRATE_LINES_COVERED["$CURRENT_CRATE"]:-0} + 1))
            fi
        fi
    fi
    # BRDA: Branch data (line_number,block_number,branch_number,taken)
    if [[ "$line" =~ ^BRDA:([0-9]+),([0-9]+),([0-9]+),(-|[0-9]+) ]]; then
        BRANCHES_TOTAL=$((BRANCHES_TOTAL + 1))
        if [ -n "$CURRENT_CRATE" ]; then
            CRATE_BRANCHES_TOTAL["$CURRENT_CRATE"]=$((${CRATE_BRANCHES_TOTAL["$CURRENT_CRATE"]:-0} + 1))
        fi
        if [ "${BASH_REMATCH[4]}" != "-" ] && [ "${BASH_REMATCH[4]}" -gt 0 ]; then
            BRANCHES_COVERED=$((BRANCHES_COVERED + 1))
            if [ -n "$CURRENT_CRATE" ]; then
                CRATE_BRANCHES_COVERED["$CURRENT_CRATE"]=$((${CRATE_BRANCHES_COVERED["$CURRENT_CRATE"]:-0} + 1))
            fi
        fi
    fi
done < "$GROCV_OUTPUT"

# Calculate overall coverage percentages
OVERALL_LINE_COV=0.0
OVERALL_BRANCH_COV=0.0

if [ $LINES_TOTAL -gt 0 ]; then
    OVERALL_LINE_COV=$(echo "scale=4; $LINES_COVERED / $LINES_TOTAL" | bc)
fi

if [ $BRANCHES_TOTAL -gt 0 ]; then
    OVERALL_BRANCH_COV=$(echo "scale=4; $BRANCHES_COVERED / $BRANCHES_TOTAL" | bc)
fi

# Update overall coverage in JSON
tmp_file=$(mktemp)
jq \
    --argjson line_cov "$OVERALL_LINE_COV" \
    --argjson branch_cov "$OVERALL_BRANCH_COV" \
    --argjson lines_covered "$LINES_COVERED" \
    --argjson lines_total "$LINES_TOTAL" \
    --argjson branches_covered "$BRANCHES_COVERED" \
    --argjson branches_total "$BRANCHES_TOTAL" \
    '.overall = {
        line_coverage: $line_cov,
        branch_coverage: $branch_cov,
        lines_covered: $lines_covered,
        lines_total: $lines_total,
        branches_covered: $branches_covered,
        branches_total: $branches_total
    }' "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Process crate-level coverage
BELOW_THRESHOLD=()
HIGH_VALUE_AREAS=()

for crate in "${!CRATE_LINES_TOTAL[@]}"; do
    lines_total="${CRATE_LINES_TOTAL[$crate]}"
    lines_covered="${CRATE_LINES_COVERED[$crate]:-0}"
    branches_total="${CRATE_BRANCHES_TOTAL[$crate]:-0}"
    branches_covered="${CRATE_BRANCHES_COVERED[$crate]:-0}"
    
    line_cov=0.0
    branch_cov=0.0
    
    if [ "$lines_total" -gt 0 ]; then
        line_cov=$(echo "scale=4; $lines_covered / $lines_total" | bc)
    fi
    
    if [ "$branches_total" -gt 0 ]; then
        branch_cov=$(echo "scale=4; $branches_covered / $branches_total" | bc)
    fi
    
    # Check if below threshold
    line_below=$(echo "$line_cov < $LINE_THRESHOLD" | bc)
    branch_below=$(echo "$branch_cov < $BRANCH_THRESHOLD" | bc)
    
    if [ "$line_below" -eq 1 ] || [ "$branch_below" -eq 1 ]; then
        BELOW_THRESHOLD+=("$crate")
        
        # Calculate gap
        line_gap=$(echo "scale=4; $LINE_THRESHOLD - $line_cov" | bc)
        branch_gap=$(echo "scale=4; $BRANCH_THRESHOLD - $branch_cov" | bc)
        
        # Estimate lines/branches needed
        lines_needed=$(echo "scale=0; ($line_gap * $lines_total) / 1" | bc)
        branches_needed=$(echo "scale=0; ($branch_gap * $branches_total) / 1" | bc)
    fi
    
    # Update JSON with crate data
    tmp_file=$(mktemp)
    jq \
        --arg crate "$crate" \
        --argjson line_cov "$line_cov" \
        --argjson branch_cov "$branch_cov" \
        --argjson lines_covered "$lines_covered" \
        --argjson lines_total "$lines_total" \
        --argjson branches_covered "$branches_covered" \
        --argjson branches_total "$branches_total" \
        --argjson line_below "$line_below" \
        --argjson branch_below "$branch_below" \
        '.crates[$crate] = {
            line_coverage: $line_cov,
            branch_coverage: $branch_cov,
            lines_covered: $lines_covered,
            lines_total: $lines_total,
            branches_covered: $branches_covered,
            branches_total: $branches_total,
            below_line_threshold: ($line_below == 1),
            below_branch_threshold: ($branch_below == 1)
        }' "$RESULTS_FILE" > "$tmp_file"
    mv "$tmp_file" "$RESULTS_FILE"
done

# Identify high-value areas (crates with high usage but low coverage)
# Load crate priorities from config
TIER_1_CRATES=$(grep -A 10 "tier_1:" "$CONFIG_FILE" | grep "^-" | sed 's/^- //' | tr '\n' ' ')

for crate in $TIER_1_CRATES; do
    crate_data=$(jq -r ".crates[\"$crate\"] // empty" "$RESULTS_FILE")
    if [ -n "$crate_data" ]; then
        line_cov=$(echo "$crate_data" | jq -r '.line_coverage')
        branch_cov=$(echo "$crate_data" | jq -r '.branch_coverage')
        
        line_below=$(echo "$line_cov < $LINE_THRESHOLD" | bc)
        branch_below=$(echo "$branch_cov < $BRANCHES_THRESHOLD" | bc)
        
        if [ "$line_below" -eq 1 ] || [ "$branch_below" -eq 1 ]; then
            HIGH_VALUE_AREAS+=("$crate")
        fi
    fi
done

# Update below_threshold and high_value_areas arrays
tmp_file=$(mktemp)
jq \
    --argjson below_threshold "$(printf '%s\n' "${BELOW_THRESHOLD[@]}" | jq -R . | jq -s .)" \
    --argjson high_value "$(printf '%s\n' "${HIGH_VALUE_AREAS[@]}" | jq -R . | jq -s .)" \
    '.below_threshold = $below_threshold | .high_value_areas = $high_value' \
    "$RESULTS_FILE" > "$tmp_file"
mv "$tmp_file" "$RESULTS_FILE"

# Print summary
echo -e "${BLUE}[coverage-assessment] Coverage Assessment Summary:${NC}"
echo -e "  Overall Line Coverage: $(printf "%.2f%%" $(echo "$OVERALL_LINE_COV * 100" | bc)) (threshold: $(printf "%.0f%%" $(echo "$LINE_THRESHOLD * 100" | bc)))"
echo -e "  Overall Branch Coverage: $(printf "%.2f%%" $(echo "$OVERALL_BRANCH_COV * 100" | bc)) (threshold: $(printf "%.0f%%" $(echo "$BRANCHES_THRESHOLD * 100" | bc)))"
echo -e "  Crates Below Threshold: ${#BELOW_THRESHOLD[@]}"
echo -e "  High-Value Areas Needing Coverage: ${#HIGH_VALUE_AREAS[@]}"

if [ ${#BELOW_THRESHOLD[@]} -gt 0 ]; then
    echo -e "${YELLOW}  Crates needing improvement:${NC}"
    for crate in "${BELOW_THRESHOLD[@]}"; do
        crate_data=$(jq -r ".crates[\"$crate\"]" "$RESULTS_FILE")
        line_cov=$(echo "$crate_data" | jq -r '.line_coverage')
        branch_cov=$(echo "$crate_data" | jq -r '.branch_coverage')
        echo -e "    - $crate: line=$(printf "%.2f%%" $(echo "$line_cov * 100" | bc)), branch=$(printf "%.2f%%" $(echo "$branch_cov * 100" | bc))"
    done
fi

echo -e "${GREEN}[coverage-assessment] Results saved to $RESULTS_FILE${NC}"
echo -e "${GREEN}[coverage-assessment] LCOV report saved to $GROCV_OUTPUT${NC}"

