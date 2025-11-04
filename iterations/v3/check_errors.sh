#!/bin/bash
echo "Checking compilation errors per crate..."
echo "========================================"

# List of all crates (excluding workspace root)
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

for crate in "${crates[@]}"; do
    if [ -d "$crate" ]; then
        echo "Checking $crate..."
        # Run cargo check and capture output
        output=$(cargo check -p "$crate" 2>&1)
        errors=$(echo "$output" | grep -c "^error")
        warnings=$(echo "$output" | grep -c "^warning")
        
        if [ "$errors" -gt 0 ] || [ "$warnings" -gt 0 ]; then
            echo "  $crate: $errors errors, $warnings warnings"
            if [ "$errors" -gt 0 ]; then
                ((crates_with_errors++))
            fi
        else
            echo "  $crate: ✅ Clean"
        fi
        
        total_errors=$((total_errors + errors))
        total_warnings=$((total_warnings + warnings))
    fi
done

echo ""
echo "SUMMARY:"
echo "========"
echo "Total crates: ${#crates[@]}"
echo "Crates with errors: $crates_with_errors"
echo "Total errors: $total_errors"
echo "Total warnings: $total_warnings"
