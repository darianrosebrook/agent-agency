#!/usr/bin/env bash
# Main Readiness Assessment Orchestrator for V3
# Coordinates all assessment modules and generates unified report
# @author: @darianrosebrook

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "$SCRIPT_DIR/../../.." && pwd)"
V3_DIR="$ROOT_DIR/iterations/v3"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Parse command line arguments
TESTS_ONLY=false
COVERAGE_ONLY=false
TODOS_ONLY=false
COMPARE_BASELINE=false
SAVE_BASELINE=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --tests-only)
            TESTS_ONLY=true
            shift
            ;;
        --coverage-only)
            COVERAGE_ONLY=true
            shift
            ;;
        --todos-only)
            TODOS_ONLY=true
            shift
            ;;
        --compare-baseline)
            COMPARE_BASELINE=true
            shift
            ;;
        --save-baseline)
            SAVE_BASELINE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --tests-only        Run only test assessment"
            echo "  --coverage-only     Run only coverage assessment"
            echo "  --todos-only        Run only TODO assessment"
            echo "  --compare-baseline   Compare against previous baseline"
            echo "  --save-baseline      Save current assessment as baseline"
            echo "  --help, -h          Show this help message"
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

cd "$V3_DIR"

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${BLUE}  V3 Readiness Assessment Framework${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo ""

START_TIME=$(date +%s)

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo -e "${RED}Error: Cargo.toml not found. Please run from iterations/v3 directory.${NC}"
    exit 1
fi

# Create artifacts directory
mkdir -p "$ROOT_DIR/artifacts"

# Build verification
if [ "$TESTS_ONLY" = false ] && [ "$COVERAGE_ONLY" = false ] && [ "$TODOS_ONLY" = false ]; then
    echo -e "${BLUE}[readiness-assessment] Verifying build...${NC}"
    if cargo check --workspace --all-features > /dev/null 2>&1; then
        echo -e "${GREEN}[readiness-assessment] Build verification passed${NC}"
    else
        echo -e "${YELLOW}[readiness-assessment] Build has warnings, but continuing...${NC}"
    fi
    echo ""
fi

# Run test assessment
if [ "$COVERAGE_ONLY" = false ] && [ "$TODOS_ONLY" = false ]; then
    echo -e "${BLUE}[readiness-assessment] Running test assessment...${NC}"
    if "$SCRIPT_DIR/test-assessment.sh"; then
        echo -e "${GREEN}[readiness-assessment] Test assessment completed${NC}"
    else
        echo -e "${YELLOW}[readiness-assessment] Test assessment completed with warnings${NC}"
    fi
    echo ""
fi

# Run coverage assessment
if [ "$TESTS_ONLY" = false ] && [ "$TODOS_ONLY" = false ]; then
    echo -e "${BLUE}[readiness-assessment] Running coverage assessment...${NC}"
    if "$SCRIPT_DIR/coverage-assessment.sh"; then
        echo -e "${GREEN}[readiness-assessment] Coverage assessment completed${NC}"
    else
        echo -e "${YELLOW}[readiness-assessment] Coverage assessment completed with warnings${NC}"
    fi
    echo ""
fi

# Run TODO assessment
if [ "$TESTS_ONLY" = false ] && [ "$COVERAGE_ONLY" = false ]; then
    echo -e "${BLUE}[readiness-assessment] Running TODO assessment...${NC}"
    if "$SCRIPT_DIR/todo-assessment.sh"; then
        echo -e "${GREEN}[readiness-assessment] TODO assessment completed${NC}"
    else
        echo -e "${YELLOW}[readiness-assessment] TODO assessment completed with warnings${NC}"
    fi
    echo ""
fi

# Run dashboard readiness check
if [ "$TESTS_ONLY" = false ] && [ "$COVERAGE_ONLY" = false ] && [ "$TODOS_ONLY" = false ]; then
    echo -e "${BLUE}[readiness-assessment] Running dashboard readiness check...${NC}"
    if "$SCRIPT_DIR/dashboard-readiness.sh"; then
        echo -e "${GREEN}[readiness-assessment] Dashboard readiness check completed${NC}"
    else
        echo -e "${YELLOW}[readiness-assessment] Dashboard readiness check completed with warnings${NC}"
    fi
    echo ""
fi

# Generate unified report
if [ "$TESTS_ONLY" = false ] && [ "$COVERAGE_ONLY" = false ] && [ "$TODOS_ONLY" = false ]; then
    echo -e "${BLUE}[readiness-assessment] Generating unified report...${NC}"
    if node "$SCRIPT_DIR/generate-report.cjs"; then
        echo -e "${GREEN}[readiness-assessment] Report generation completed${NC}"
    else
        echo -e "${RED}[readiness-assessment] Report generation failed${NC}"
        exit 1
    fi
    echo ""
fi

# Compare against baseline if requested
if [ "$COMPARE_BASELINE" = true ]; then
    echo -e "${BLUE}[readiness-assessment] Comparing against baseline...${NC}"
    if node "$SCRIPT_DIR/compare-baseline.cjs"; then
        echo -e "${GREEN}[readiness-assessment] Baseline comparison completed${NC}"
    else
        echo -e "${YELLOW}[readiness-assessment] Baseline comparison completed with warnings${NC}"
    fi
    echo ""
fi

# Save baseline if requested
if [ "$SAVE_BASELINE" = true ]; then
    echo -e "${BLUE}[readiness-assessment] Saving current assessment as baseline...${NC}"
    BASELINE_FILE="$ROOT_DIR/artifacts/baseline.json"
    LATEST_ASSESSMENT=$(ls -t "$ROOT_DIR/artifacts"/readiness-assessment-*.json 2>/dev/null | head -1)
    
    if [ -n "$LATEST_ASSESSMENT" ]; then
        cp "$LATEST_ASSESSMENT" "$BASELINE_FILE"
        echo -e "${GREEN}[readiness-assessment] Baseline saved to $BASELINE_FILE${NC}"
    else
        echo -e "${RED}[readiness-assessment] No assessment file found to save as baseline${NC}"
    fi
    echo ""
fi

# Calculate duration
END_TIME=$(date +%s)
DURATION=$((END_TIME - START_TIME))
MINUTES=$((DURATION / 60))
SECONDS=$((DURATION % 60))

echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "${GREEN}  Readiness Assessment Complete!${NC}"
echo -e "${BLUE}═══════════════════════════════════════════════════════════${NC}"
echo -e "  Duration: ${MINUTES}m ${SECONDS}s"
echo -e "  Results: $ROOT_DIR/artifacts/"
echo ""

# Find and display latest report
LATEST_REPORT=$(ls -t "$ROOT_DIR/artifacts"/readiness-assessment-*.md 2>/dev/null | head -1)
if [ -n "$LATEST_REPORT" ]; then
    echo -e "${GREEN}  Latest Report: $LATEST_REPORT${NC}"
    echo ""
    
    # Display readiness score from JSON
    LATEST_JSON=$(ls -t "$ROOT_DIR/artifacts"/readiness-assessment-*.json 2>/dev/null | head -1)
    if [ -n "$LATEST_JSON" ] && command -v jq &> /dev/null; then
        SCORE=$(jq -r '.readiness_score.percentage' "$LATEST_JSON" 2>/dev/null || echo "N/A")
        echo -e "  Overall Readiness Score: ${SCORE}%"
    fi
fi

echo ""

