#!/bin/bash

echo "# Detailed Error Analysis Report"
echo "Generated: $(date)"
echo ""

crates_with_errors=(
    "agent-constitutional-council"
    "agent-data-processing" 
    "agent-evaluation"
    "agent-orchestration"
    "agent-research"
    "agent-workers"
    "data-infrastructure"
    "data-interfaces"
    "engine-coreml"
    "system-federated-ml"
    "testing-validation"
)

echo "## Error Details by Crate"
echo ""

for crate in "${crates_with_errors[@]}"; do
    echo "### $crate"
    echo ""
    
    # Get compilation output
    output=$(cd "$crate" 2>/dev/null && cargo check 2>&1)
    
    # Extract and display errors
    echo "$output" | grep "^error:" | while read -r line; do
        echo "- $line"
    done
    
    echo ""
    
    # Extract file locations for context
    echo "#### Files with errors:"
    echo "$output" | grep "^error:" -A 2 | grep -E " --> " | sed 's/ --> /- /' | head -5
    echo ""
    echo "---"
    echo ""
done

echo "## Error Pattern Analysis"
echo ""

# Count error types
error_patterns=$(for crate in "${crates_with_errors[@]}"; do
    cd "$crate" 2>/dev/null && cargo check 2>&1
done | grep "^error:" | sed 's/error: //' | sort | uniq -c | sort -nr)

echo "| Count | Error Pattern |"
echo "|-------|---------------|"
echo "$error_patterns" | while read count pattern; do
    printf "| %-5d | %-50s |\n" "$count" "$pattern"
done

echo ""
echo "## Prioritized Fix Order"
echo ""

# Group by error type for efficient fixing
echo "### Quick Syntax Fixes (High Impact)"
echo "- Unclosed delimiters, syntax errors"
echo ""

echo "### Missing Dependencies (Medium Impact)"  
echo "- Missing crate dependencies in Cargo.toml"
echo "- Import path issues"
echo ""

echo "### Type System Issues (Medium Impact)"
echo "- Type mismatches"
echo "- Missing trait implementations"
echo "- Generic parameter issues"
echo ""

echo "### API Contract Issues (Lower Impact)"
echo "- Function signature mismatches"
echo "- Missing method implementations"
echo ""
