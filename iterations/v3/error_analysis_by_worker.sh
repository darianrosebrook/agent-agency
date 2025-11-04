#!/bin/bash

echo "# Comprehensive Error Analysis by Worker Assignment (3 Workers)"
echo "Generated: $(date)"
echo ""

# Define crates by category and priority
declare -a high_priority_crates=(
    "testing-validation"
    "agent-orchestration"
    "agent-research"
    "agent-workers"
)

declare -a medium_priority_crates=(
    "system-federated-ml"
    "system-observability"
    "data-infrastructure"
    "data-interfaces"
    "agent-data-processing"
)

declare -a low_priority_crates=(
    "system-configuration"
    "system-quality-security"
    "system-resilience"
    "system-resources"
    "system-acceleration"
    "system-common-interfaces"
    "engine-coreml"
    "development-tools"
    "agent-mcp"
    "agent-memory"
    "agent-model-management"
    "agent-evaluation"
    "agent-constitutional-council"
    "agent-agency-contracts"
)

# Function to analyze a single crate
analyze_crate() {
    local crate=$1
    echo "🔍 Analyzing $crate..."

    # Run cargo check and capture output
    local output_file="/tmp/cargo_check_${crate}.log"
    cd "$crate" && cargo check 2>&1 | tee "$output_file"
    local exit_code=${PIPESTATUS[0]}

    # Count different error types
    local total_errors=$(grep -c "^error\[" "$output_file")
    local compile_errors=$(grep "^error\[E" "$output_file" | wc -l)
    local type_errors=$(grep -E "^error\[(E04|E05|E06)" "$output_file" | wc -l)
    local trait_errors=$(grep -E "^error\[(E027|E059)" "$output_file" | wc -l)
    local import_errors=$(grep -E "^error\[(E043|E046)" "$output_file" | wc -l)
    local lifetime_errors=$(grep -E "^error\[(E049|E050|E051)" "$output_file" | wc -l)
    local warnings=$(grep -c "^warning:" "$output_file")

    # Categorize errors
    local schemars_errors=$(grep -c "schemars" "$output_file")
    local serde_errors=$(grep -c "serde" "$output_file")
    local missing_trait_errors=$(grep -c "trait.*not.*implemented" "$output_file")
    local type_mismatch_errors=$(grep -c "expected.*found" "$output_file")
    local import_resolution_errors=$(grep -c "cannot find" "$output_file")

    # Store results
    eval "errors_$crate=$total_errors"
    eval "warnings_$crate=$warnings"
    eval "compile_errors_$crate=$compile_errors"
    eval "type_errors_$crate=$type_errors"
    eval "trait_errors_$crate=$trait_errors"
    eval "import_errors_$crate=$import_errors"
    eval "lifetime_errors_$crate=$lifetime_errors"
    eval "schemars_errors_$crate=$schemars_errors"
    eval "serde_errors_$crate=$serde_errors"

    echo "  📊 $crate: $total_errors errors, $warnings warnings"
    if [ "$total_errors" -gt 0 ]; then
        echo "     🔧 Type errors: $type_errors, Trait errors: $trait_errors, Import errors: $import_errors"
        echo "     📦 Schemars: $schemars_errors, Serde: $serde_errors"
    fi

    # Clean up
    rm -f "$output_file"
}

# Analyze all crates
echo "Phase 1: Analyzing all crates..."
echo ""

for crate in "${high_priority_crates[@]}" "${medium_priority_crates[@]}" "${low_priority_crates[@]}"; do
    if [ -d "$crate" ]; then
        analyze_crate "$crate"
    else
        echo "⚠️  Directory $crate not found, skipping"
    fi
done

echo ""
echo "Phase 2: Worker Assignment Analysis"
echo "=================================="

# Calculate totals for each priority group
total_high_errors=0
total_medium_errors=0
total_low_errors=0
total_high_warnings=0
total_medium_warnings=0
total_low_warnings=0

for crate in "${high_priority_crates[@]}"; do
    eval "errors=\${errors_$crate:-0}"
    eval "warnings=\${warnings_$crate:-0}"
    ((total_high_errors += errors))
    ((total_high_warnings += warnings))
done

for crate in "${medium_priority_crates[@]}"; do
    eval "errors=\${errors_$crate:-0}"
    eval "warnings=\${warnings_$crate:-0}"
    ((total_medium_errors += errors))
    ((total_medium_warnings += warnings))
done

for crate in "${low_priority_crates[@]}"; do
    eval "errors=\${errors_$crate:-0}"
    eval "warnings=\${warnings_$crate:-0}"
    ((total_low_errors += errors))
    ((total_low_warnings += warnings))
done

# Calculate balanced worker assignments
total_errors=$((total_high_errors + total_medium_errors + total_low_errors))
total_warnings=$((total_high_warnings + total_medium_warnings + total_low_warnings))

echo ""
echo "## 📊 Overall Statistics"
echo ""
echo "| Category | Errors | Warnings | Total Issues |"
echo "|----------|--------|-----------|--------------|"
echo "| High Priority | $total_high_errors | $total_high_warnings | $((total_high_errors + total_high_warnings)) |"
echo "| Medium Priority | $total_medium_errors | $total_medium_warnings | $((total_medium_errors + total_medium_warnings)) |"
echo "| Low Priority | $total_low_errors | $total_low_warnings | $((total_low_errors + total_low_warnings)) |"
echo "| **TOTAL** | **$total_errors** | **$total_warnings** | **$((total_errors + total_warnings))** |"

echo ""
echo "## 🎯 3-Worker Distribution Strategy"
echo ""

# Strategy: Worker 1 gets high priority, Worker 2 gets medium, Worker 3 gets low + overflow
worker1_errors=$total_high_errors
worker1_warnings=$total_high_warnings
worker1_crates=(${high_priority_crates[@]})

worker2_errors=$total_medium_errors
worker2_warnings=$total_medium_warnings
worker2_crates=(${medium_priority_crates[@]})

worker3_errors=$total_low_errors
worker3_warnings=$total_low_warnings
worker3_crates=(${low_priority_crates[@]})

echo "### Worker 1 (High Priority - Critical Path)"
echo "**Focus:** Core business logic, testing, orchestration"
echo "**Workload:** $worker1_errors errors, $worker1_warnings warnings"
echo "**Crates:**"
for crate in "${worker1_crates[@]}"; do
    eval "errors=\${errors_$crate:-0}"
    eval "warnings=\${warnings_$crate:-0}"
    if [ "$errors" -gt 0 ] || [ "$warnings" -gt 0 ]; then
        echo "- $crate ($errors errors, $warnings warnings)"
    fi
done

echo ""
echo "### Worker 2 (Medium Priority - Infrastructure)"
echo "**Focus:** Federated learning, observability, data processing"
echo "**Workload:** $worker2_errors errors, $worker2_warnings warnings"
echo "**Crates:**"
for crate in "${worker2_crates[@]}"; do
    eval "errors=\${errors_$crate:-0}"
    eval "warnings=\${warnings_$crate:-0}"
    if [ "$errors" -gt 0 ] || [ "$warnings" -gt 0 ]; then
        echo "- $crate ($errors errors, $warnings warnings)"
    fi
done

echo ""
echo "### Worker 3 (Low Priority - Utilities & Tools)"
echo "**Focus:** Configuration, quality/security, tools, evaluation"
echo "**Workload:** $worker3_errors errors, $worker3_warnings warnings"
echo "**Crates:**"
for crate in "${worker3_crates[@]}"; do
    eval "errors=\${errors_$crate:-0}"
    eval "warnings=\${warnings_$crate:-0}"
    if [ "$errors" -gt 0 ] || [ "$warnings" -gt 0 ]; then
        echo "- $crate ($errors errors, $warnings warnings)"
    fi
done

echo ""
echo "## 🔍 Error Type Analysis"
echo ""

# Analyze error types across all crates
echo "### Error Categories Breakdown:"
echo ""

# Calculate error type totals
total_type_errors=0
total_trait_errors=0
total_import_errors=0
total_lifetime_errors=0
total_schemars_errors=0
total_serde_errors=0

for crate in "${high_priority_crates[@]}" "${medium_priority_crates[@]}" "${low_priority_crates[@]}"; do
    eval "total_type_errors=\$((total_type_errors + \${type_errors_$crate:-0}))"
    eval "total_trait_errors=\$((total_trait_errors + \${trait_errors_$crate:-0}))"
    eval "total_import_errors=\$((total_import_errors + \${import_errors_$crate:-0}))"
    eval "total_lifetime_errors=\$((total_lifetime_errors + \${lifetime_errors_$crate:-0}))"
    eval "total_schemars_errors=\$((total_schemars_errors + \${schemars_errors_$crate:-0}))"
    eval "total_serde_errors=\$((total_serde_errors + \${serde_errors_$crate:-0}))"
done

echo "| Error Type | Count | Percentage | Primary Issue |"
echo "|------------|-------|------------|---------------|"
echo "| Type Mismatches | $total_type_errors | $((total_type_errors * 100 / total_errors))% | Schema/API contract issues |"
echo "| Missing Traits | $total_trait_errors | $((total_trait_errors * 100 / total_errors))% | Implementation gaps |"
echo "| Import Resolution | $total_import_errors | $((total_import_errors * 100 / total_errors))% | Module/dependency issues |"
echo "| Lifetime Issues | $total_lifetime_errors | $((total_lifetime_errors * 100 / total_errors))% | Memory management |"
echo "| Schemars Issues | $total_schemars_errors | $((total_schemars_errors * 100 / total_errors))% | Schema generation |"
echo "| Serde Issues | $total_serde_errors | $((total_serde_errors * 100 / total_errors))% | Serialization |"

echo ""
echo "## 📋 Work Distribution Recommendations"
echo ""

# Calculate workload balance
avg_errors_per_worker=$((total_errors / 3))
echo "### Workload Balance:"
echo "- **Target per worker:** ~$avg_errors_per_worker errors"
echo "- **Worker 1:** $worker1_errors errors ($(( (worker1_errors - avg_errors_per_worker) * 100 / avg_errors_per_worker ))% from target)"
echo "- **Worker 2:** $worker2_errors errors ($(( (worker2_errors - avg_errors_per_worker) * 100 / avg_errors_per_worker ))% from target)"
echo "- **Worker 3:** $worker3_errors errors ($(( (worker3_errors - avg_errors_per_worker) * 100 / avg_errors_per_worker ))% from target)"

echo ""
echo "### Priority Order:"
echo "1. **Worker 1** - Fix critical path issues first (testing, orchestration, core agents)"
echo "2. **Worker 2** - Address infrastructure stability (federated learning, observability)"
echo "3. **Worker 3** - Clean up utilities and tools (configuration, quality checks)"

echo ""
echo "### Collaboration Points:"
echo "- **Schema Contracts:** Coordinate between Workers 1 & 2 for API consistency"
echo "- **Dependency Updates:** Worker 3 should coordinate major version bumps with others"
echo "- **Testing Integration:** Worker 1 should validate fixes don't break Worker 2's work"

echo ""
echo "## 🎯 Next Steps"
echo ""
echo "1. **Start with Worker 1** - Critical path fixes"
echo "2. **Parallel Work** - Workers 2 & 3 can start after Worker 1 clears initial blockers"
echo "3. **Daily Sync** - Check for cross-worker dependencies"
echo "4. **Rebalance** - Redistribute work if one worker finishes early"

echo ""
echo "---"
echo "*Analysis completed at $(date)*"
