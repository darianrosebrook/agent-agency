#!/bin/bash
# Scoped mutation testing - only test modules with adequate test coverage
# @author @darianrosebrook

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MUTATION_TIMEOUT="${MUTATION_TIMEOUT:-300}"
MUTATION_MIN_SCORE="${MUTATION_MIN_SCORE:-0.5}"

# Check if cargo-mutants is installed
if ! command -v cargo-mutants &> /dev/null; then
    echo -e "${YELLOW}[mutation-scoped] cargo-mutants not found, installing...${NC}"
    cargo install cargo-mutants
fi

# Function to check if a module has tests
has_tests() {
    local file="$1"
    local test_file="${file/src/tests}"
    test_file="${test_file%.rs}_test.rs"
    
    # Check if test file exists or if there are #[test] or #[tokio::test] in the file
    if [ -f "$test_file" ]; then
        return 0
    fi
    
    # Check for inline tests
    if grep -q "#\[test\]\|#\[tokio::test\]" "$file" 2>/dev/null; then
        return 0
    fi
    
    # Check for test module
    if grep -q "#\[cfg(test)\]" "$file" 2>/dev/null; then
        return 0
    fi
    
    return 1
}

# Function to find modules with tests
find_tested_modules() {
    local crate_dir="$1"
    local tested_modules=()
    
    cd "$crate_dir"
    
    # Find all Rust source files
    while IFS= read -r file; do
        if has_tests "$file"; then
            tested_modules+=("$file")
            echo -e "${GREEN}✓${NC} $file (has tests)"
        else
            echo -e "${YELLOW}⊘${NC} $file (no tests, skipping)"
        fi
    done < <(find src -name "*.rs" -type f | grep -v "/tests/" | sort)
    
    echo "${tested_modules[@]}"
}

# Main execution
main() {
    local crate_path="${1:-iterations/v3/system-federated-ml}"
    local crate_dir="$PROJECT_ROOT/$crate_path"
    
    if [ ! -d "$crate_dir" ]; then
        echo -e "${RED}[mutation-scoped] Error: Crate directory not found: $crate_dir${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}[mutation-scoped] Scanning for modules with tests...${NC}"
    echo ""
    
    # Find tested modules
    cd "$crate_dir"
    local tested_modules=($(find_tested_modules "$crate_dir"))
    
    if [ ${#tested_modules[@]} -eq 0 ]; then
        echo -e "${RED}[mutation-scoped] No modules with tests found!${NC}"
        echo -e "${YELLOW}[mutation-scoped] Add tests before running mutation testing.${NC}"
        exit 1
    fi
    
    echo ""
    echo -e "${BLUE}[mutation-scoped] Found ${#tested_modules[@]} modules with tests${NC}"
    echo ""
    
    # Build file filter for cargo-mutants
    local file_args=()
    for module in "${tested_modules[@]}"; do
        # Convert relative path to glob pattern
        local pattern="**/${module#src/}"
        file_args+=("--file" "$pattern")
    done
    
    echo -e "${BLUE}[mutation-scoped] Running mutation testing on tested modules only...${NC}"
    echo ""
    
    # Run mutation testing with file filters
    cd "$PROJECT_ROOT"
    cargo mutants \
        --workspace \
        "${file_args[@]}" \
        --timeout "$MUTATION_TIMEOUT" \
        --no-shuffle \
        --baseline run \
        2>&1 | tee /tmp/mutation_scoped.log
    
    # Parse results
    local mutation_score=$(grep -oE "score: [0-9]+\.[0-9]+" /tmp/mutation_scoped.log | grep -oE "[0-9]+\.[0-9]+" | tail -1)
    
    if [ -n "$mutation_score" ]; then
        echo ""
        echo -e "${BLUE}[mutation-scoped] Mutation score: ${mutation_score}${NC}"
        
        # Compare against threshold
        local score_float=$(echo "$mutation_score * 100" | bc -l 2>/dev/null || echo "0")
        local min_float=$(echo "$MUTATION_MIN_SCORE * 100" | bc -l 2>/dev/null || echo "0")
        
        if (( $(echo "$score_float >= $min_float" | bc -l) )); then
            echo -e "${GREEN}[mutation-scoped] Mutation score meets threshold (${score_float}% >= ${min_float}%)${NC}"
            exit 0
        else
            echo -e "${RED}[mutation-scoped] Mutation score below threshold (${score_float}% < ${min_float}%)${NC}"
            exit 1
        fi
    else
        echo -e "${YELLOW}[mutation-scoped] Could not parse mutation score${NC}"
        exit 0
    fi
}

# Run main function
main "$@"









