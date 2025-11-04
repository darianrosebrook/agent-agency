#!/bin/bash

echo "# Accurate Crate Compilation Status Report"
echo "Generated: $(date)"
echo ""

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

echo "## Individual Crate Status"
echo ""

for crate in "${crates[@]}"; do
    echo "Checking $crate..."
    
    # Run cargo check and capture output
    output=$(cd "$crate" 2>/dev/null && cargo check 2>&1)
    exit_code=$?
    
    # Count errors and warnings more accurately
    errors=$(echo "$output" | grep -c "^error:")
    warnings=$(echo "$output" | grep -c "^warning:")
    
    if [ $exit_code -eq 0 ]; then
        status="✅ Compiles"
        error_count=0
    else
        status="❌ Errors"
        error_count=$errors
        ((crates_with_errors++))
    fi
    
    if [ $warnings -gt 0 ]; then
        ((crates_with_warnings++))
    fi
    
    printf "| %-30s | %-12s | %-6d | %-8d |\n" "$crate" "$status" "$error_count" "$warnings"
    
    ((total_errors += error_count))
    ((total_warnings += warnings))
done

echo ""
echo "## Summary"
echo ""
echo "**Total Errors:** $total_errors"
echo "**Total Warnings:** $total_warnings"
echo "**Crates with Errors:** $crates_with_errors"
echo "**Crates with Warnings:** $crates_with_warnings"

echo ""
echo "## Priority Recommendations"
echo ""

if [ $total_errors -eq 0 ]; then
    echo "🎉 **All crates compile successfully!**"
    echo "Next steps:"
    echo "- Address $total_warnings warnings"
    echo "- Add comprehensive tests"
    echo "- Performance optimization"
elif [ $crates_with_errors -le 3 ]; then
    echo "🔧 **Almost there!** Only $crates_with_errors crates have compilation errors."
    echo "Focus on fixing the remaining $total_errors errors."
else
    echo "⚠️ **Multiple compilation issues remain.**"
    echo "- $total_errors total errors across $crates_with_errors crates"
    echo "- $total_warnings warnings across $crates_with_warnings crates"
    echo ""
    echo "Need detailed analysis of error patterns."
fi
