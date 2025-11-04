#!/bin/bash

echo "# Crate Compilation Status Report"
echo "Generated: $(date)"
echo ""

# Use indexed arrays instead of associative arrays for better compatibility
crate_status=()
crate_errors=()
crate_warnings=()

crates=(
    "agent-agency-contracts"
    "agent-constitutional-council" 
    "agent-data-processing"
    "agent-evaluation"
    "agent-mcp"
    "agent-memory"
    "agent-model-management"
    "agent-orchestration"
    "agent-research"
    "agent-workers"
    "data-infrastructure"
    "data-interfaces"
    "development-tools"
    "engine-coreml"
    "system-acceleration"
    "system-common-interfaces"
    "system-configuration"
    "system-federated-ml"
    "system-observability"
    "system-quality-security"
    "system-resilience"
    "system-resources"
    "testing-validation"
)

total_errors=0
total_warnings=0
crates_with_errors=0
crates_with_warnings=0

# Create index mapping (associative array for string key lookup)
declare -A crate_indices
for i in "${!crates[@]}"; do
    crate_indices[${crates[$i]}]=$i
done

for crate in "${crates[@]}"; do
    echo "Checking $crate..."

    # Run cargo check and capture output
    output=$(cd "$crate" && cargo check 2>&1)
    exit_code=$?

    # Count errors and warnings more accurately
    errors=$(echo "$output" | grep -c "^error\[")
    warnings=$(echo "$output" | grep -c "^warning:")

    idx=${crate_indices[$crate]}
    if [ $exit_code -eq 0 ]; then
        crate_status[$idx]="✅ Compiles"
        crate_errors[$idx]=0
        crate_warnings[$idx]=$warnings
        if [ $warnings -gt 0 ]; then
            ((crates_with_warnings++))
        fi
    else
        crate_status[$idx]="❌ Errors"
        crate_errors[$idx]=$errors
        crate_warnings[$idx]=$warnings
        ((crates_with_errors++))
        if [ $warnings -gt 0 ]; then
            ((crates_with_warnings++))
        fi
    fi

    ((total_errors += errors))
    ((total_warnings += warnings))
done

echo "## Summary"
echo ""
echo "| Crate | Status | Errors | Warnings |"
echo "|-------|--------|--------|----------|"

for i in "${!crates[@]}"; do
    crate="${crates[$i]}"
    printf "| %-30s | %-12s | %-6d | %-8d |\n" \
        "$crate" "${crate_status[$i]}" "${crate_errors[$i]}" "${crate_warnings[$i]}"
done

echo ""
echo "**Total Errors:** $total_errors"
echo "**Total Warnings:** $total_warnings"
echo "**Crates with Errors:** $crates_with_errors"
echo "**Crates with Warnings:** $crates_with_warnings"

# Show top error crates
echo ""
echo "## Top Error Crates"
echo ""
echo "| Rank | Crate | Errors |"
echo "|------|-------|--------|"

# Sort crates by error count descending
sorted_crates=$(for i in "${!crates[@]}"; do
    echo "${crate_errors[$i]} ${crates[$i]}"
done | sort -nr | head -10)

rank=1
echo "$sorted_crates" | while read errors crate; do
    if [ "$errors" -gt 0 ]; then
        printf "| %-4d | %-25s | %-6d |\n" "$rank" "$crate" "$errors"
        ((rank++))
    fi
done

echo ""
echo "## Recommendations"
echo ""

if [ $total_errors -eq 0 ]; then
    echo "🎉 **All crates compile successfully!**"
    echo ""
    echo "Focus on:"
    echo "- Reducing warnings ($total_warnings total)"
    echo "- Adding tests and documentation" 
    echo "- Performance optimization"
elif [ $crates_with_errors -le 3 ]; then
    echo "🔧 **Nearly there!** Only $crates_with_errors crates have errors."
    echo ""
    echo "Priority order:"
    echo "1. Fix remaining $total_errors errors"
    echo "2. Address $total_warnings warnings"
    echo "3. Run integration tests"
else
    echo "⚠️ **Multiple crates still have errors.**"
    echo ""
    echo "Focus on:"
    echo "1. Fix top error crates first (see table above)"
    echo "2. Address $total_errors total errors"
    echo "3. Tackle $total_warnings warnings"
fi
