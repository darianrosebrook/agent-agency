#!/bin/bash
# V3 Agent Task Evaluation Script
# Evaluates completed tasks against theory.md requirements

API_URL="http://localhost:8080"
REPORT_FILE="task_evaluation_report.json"

echo "=================================================================================="
echo "V3 Agent Task Evaluation Report"
echo "=================================================================================="
echo ""

# Get all tasks
echo "Fetching all tasks..."
TASKS_JSON=$(curl -s "${API_URL}/api/v1/tasks")
COMPLETED_COUNT=$(echo "$TASKS_JSON" | python3 -c "import sys, json; tasks = json.load(sys.stdin); print(len([t for t in tasks if t.get('status') == 'completed']))" 2>/dev/null || echo "0")
TOTAL_COUNT=$(echo "$TASKS_JSON" | python3 -c "import sys, json; tasks = json.load(sys.stdin); print(len(tasks))" 2>/dev/null || echo "0")

echo "Total tasks: ${TOTAL_COUNT}"
echo "Completed tasks: ${COMPLETED_COUNT}"
echo ""

# Get completed task IDs
COMPLETED_IDS=$(echo "$TASKS_JSON" | python3 -c "import sys, json; tasks = json.load(sys.stdin); print('\n'.join([t['task_id'] for t in tasks if t.get('status') == 'completed'][:7]))" 2>/dev/null)

if [ -z "$COMPLETED_IDS" ]; then
    echo "No completed tasks found for evaluation"
    exit 1
fi

# Evaluate each task
EVALUATION_COUNT=0
TOTAL_QUALITY=0
TOTAL_COMPLIANCE=0

echo "=================================================================================="
echo "Evaluating Tasks"
echo "=================================================================================="
echo ""

for TASK_ID in $COMPLETED_IDS; do
    echo "--------------------------------------------------------------------------------"
    echo "Task ID: ${TASK_ID}"
    echo "--------------------------------------------------------------------------------"
    
    # Get task result
    TASK_RESULT=$(curl -s "${API_URL}/api/v1/tasks/${TASK_ID}/result")
    TASK_STATUS=$(curl -s "${API_URL}/api/v1/tasks/${TASK_ID}/status")
    
    # Extract key metrics using Python
    METRICS=$(echo "$TASK_RESULT" | python3 << 'PYTHON_SCRIPT'
import sys
import json

try:
    data = json.load(sys.stdin)
    artifacts = data.get("artifacts", {})
    
    # Code changes
    code_stats = artifacts.get("code_changes", {}).get("statistics", {})
    files_modified = code_stats.get("files_modified", 0)
    lines_added = code_stats.get("lines_added", 0)
    
    # Tests
    tests = artifacts.get("tests", {})
    unit_total = tests.get("unit_tests", {}).get("total", 0)
    unit_passed = tests.get("unit_tests", {}).get("passed", 0)
    
    # Coverage
    coverage = artifacts.get("coverage", {})
    line_coverage = coverage.get("line_coverage", 0)
    branch_coverage = coverage.get("branch_coverage", 0)
    
    # Linting
    linting = artifacts.get("linting", {})
    lint_errors = linting.get("errors", 0)
    lint_warnings = linting.get("warnings", 0)
    
    # Provenance
    provenance = artifacts.get("provenance", {})
    exec_id = provenance.get("execution_id", "00000000-0000-0000-0000-000000000000")
    has_provenance = exec_id != "00000000-0000-0000-0000-000000000000"
    
    # Quality score calculation
    quality_score = 0.0
    if files_modified > 0 or lines_added > 0:
        quality_score += 0.3
    if unit_total > 0:
        quality_score += 0.2 * (unit_passed / unit_total if unit_total > 0 else 0)
    if line_coverage > 0 or branch_coverage > 0:
        quality_score += 0.2 * ((line_coverage + branch_coverage) / 200.0)
    if lint_errors == 0:
        quality_score += 0.1
    if has_provenance:
        quality_score += 0.2
    
    # CAWS compliance
    compliance = 0.0
    if data.get("quality_report"):
        compliance += 0.3
    if lint_errors == 0:
        compliance += 0.2
    if has_provenance:
        compliance += 0.3
    if artifacts.get("metadata"):
        compliance += 0.2
    
    print(json.dumps({
        "files_modified": files_modified,
        "lines_added": lines_added,
        "unit_tests_total": unit_total,
        "unit_tests_passed": unit_passed,
        "line_coverage": line_coverage,
        "branch_coverage": branch_coverage,
        "lint_errors": lint_errors,
        "lint_warnings": lint_warnings,
        "has_provenance": has_provenance,
        "quality_score": quality_score,
        "compliance_score": compliance,
        "has_code_changes": files_modified > 0 or lines_added > 0,
        "has_tests": unit_total > 0,
        "has_coverage": line_coverage > 0 or branch_coverage > 0,
        "has_linting": True,
        "meets_basic_requirements": (files_modified > 0 or lines_added > 0) or unit_total > 0 or has_provenance
    }))
except Exception as e:
    print(json.dumps({"error": str(e)}))
PYTHON_SCRIPT
)
    
    if echo "$METRICS" | grep -q "error"; then
        echo "  Error evaluating task metrics"
        continue
    fi
    
    # Display metrics
    echo "  Artifact Quality:"
    echo "    Files Modified: $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('files_modified', 0))")"
    echo "    Lines Added: $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('lines_added', 0))")"
    echo "    Unit Tests: $(echo "$METRICS" | python3 -c "import sys, json; m=json.load(sys.stdin); print(f\"{m.get('unit_tests_passed', 0)}/{m.get('unit_tests_total', 0)}\")")"
    echo "    Coverage: Line $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('line_coverage', 0))")%, Branch $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('branch_coverage', 0))")%"
    echo "    Lint Errors: $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('lint_errors', 0))")"
    echo "    Has Provenance: $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('has_provenance', False))")"
    echo "    Quality Score: $(echo "$METRICS" | python3 -c "import sys, json; print(f\"{json.load(sys.stdin).get('quality_score', 0):.2f}\")")"
    echo "    Meets Basic Requirements: $(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('meets_basic_requirements', False))")"
    
    echo ""
    echo "  CAWS Compliance:"
    echo "    Compliance Score: $(echo "$METRICS" | python3 -c "import sys, json; print(f\"{json.load(sys.stdin).get('compliance_score', 0):.2f}\")")"
    
    # Accumulate scores
    QUALITY=$(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('quality_score', 0))")
    COMPLIANCE=$(echo "$METRICS" | python3 -c "import sys, json; print(json.load(sys.stdin).get('compliance_score', 0))")
    
    TOTAL_QUALITY=$(echo "$TOTAL_QUALITY + $QUALITY" | bc -l 2>/dev/null || echo "$TOTAL_QUALITY")
    TOTAL_COMPLIANCE=$(echo "$TOTAL_COMPLIANCE + $COMPLIANCE" | bc -l 2>/dev/null || echo "$TOTAL_COMPLIANCE")
    EVALUATION_COUNT=$((EVALUATION_COUNT + 1))
    
    echo ""
done

# Calculate averages
if [ $EVALUATION_COUNT -gt 0 ]; then
    AVG_QUALITY=$(echo "scale=2; $TOTAL_QUALITY / $EVALUATION_COUNT" | bc -l 2>/dev/null || echo "0")
    AVG_COMPLIANCE=$(echo "scale=2; $TOTAL_COMPLIANCE / $EVALUATION_COUNT" | bc -l 2>/dev/null || echo "0")
else
    AVG_QUALITY=0
    AVG_COMPLIANCE=0
fi

echo "=================================================================================="
echo "SUMMARY REPORT"
echo "=================================================================================="
echo ""
echo "Total Tasks Evaluated: ${EVALUATION_COUNT}"
echo "Average Artifact Quality Score: ${AVG_QUALITY}"
echo "Average CAWS Compliance Score: ${AVG_COMPLIANCE}"
echo ""

# Save summary
cat > "${REPORT_FILE}" << EOF
{
  "evaluation_date": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")",
  "total_tasks_evaluated": ${EVALUATION_COUNT},
  "average_quality_score": ${AVG_QUALITY},
  "average_compliance_score": ${AVG_COMPLIANCE},
  "summary": {
    "total_tasks": ${TOTAL_COUNT},
    "completed_tasks": ${COMPLETED_COUNT},
    "evaluated_tasks": ${EVALUATION_COUNT}
  }
}
EOF

echo "Report saved to: ${REPORT_FILE}"

