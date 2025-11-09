#!/bin/bash
# V3 Agent Task Execution Evaluation Script
# Repeatable test suite for evaluating task execution and artifact quality

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
API_URL="${API_URL:-http://localhost:8080}"
RESULTS_DIR="${RESULTS_DIR:-$PROJECT_ROOT/iterations/v3/test-results}"
TIMESTAMP=$(date -u +"%Y%m%d_%H%M%S")
RESULTS_FILE="$RESULTS_DIR/evaluation_${TIMESTAMP}.json"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Create results directory
mkdir -p "$RESULTS_DIR"

# Test tasks to evaluate
declare -a TEST_TASKS=(
    "Create a Python function that calculates fibonacci numbers"
    "Create comprehensive API documentation for the user service"
    "Write unit tests for the payment processing module"
    "Fix the memory leak in the data processing pipeline"
    "Add user profile editing functionality with validation"
)

log() {
    echo -e "${GREEN}[$(date +'%H:%M:%S')]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[$(date +'%H:%M:%S')] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[$(date +'%H:%M:%S')] ERROR:${NC} $1"
}

check_server() {
    log "Checking API server health..."
    if ! curl -s -f "$API_URL/health" > /dev/null 2>&1; then
        error "API server not responding at $API_URL"
        error "Please start the server: cargo run --bin agent-agency-api-server --features orchestration"
        exit 1
    fi
    log "API server is healthy"
}

submit_task() {
    local description="$1"
    local response=$(curl -s -X POST "$API_URL/api/v1/tasks" \
        -H "Content-Type: application/json" \
        -d "{\"description\": \"$description\", \"execution_mode\": \"auto\"}")
    
    local task_id=$(echo "$response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('task_id', ''))" 2>/dev/null || echo "")
    
    if [ -z "$task_id" ]; then
        error "Failed to submit task: $description"
        echo "$response" >&2
        return 1
    fi
    
    echo "$task_id"
}

wait_for_completion() {
    local task_id="$1"
    local max_wait="${2:-300}"  # Default 5 minutes
    local check_interval="${3:-5}"  # Check every 5 seconds
    local elapsed=0
    
    log "Waiting for task $task_id to complete (max ${max_wait}s)..."
    
    while [ $elapsed -lt $max_wait ]; do
        local status=$(curl -s "$API_URL/api/v1/tasks/$task_id/status" 2>/dev/null | \
            python3 -c "import sys, json; print(json.load(sys.stdin).get('status', 'unknown'))" 2>/dev/null || echo "unknown")
        
        if [ "$status" = "completed" ] || [ "$status" = "failed" ]; then
            log "Task $task_id finished with status: $status"
            echo "$status"
            return 0
        fi
        
        sleep "$check_interval"
        elapsed=$((elapsed + check_interval))
        
        if [ $((elapsed % 30)) -eq 0 ]; then
            log "  Still waiting... (${elapsed}s elapsed, status: $status)"
        fi
    done
    
    warn "Task $task_id timed out after ${max_wait}s"
    echo "timeout"
    return 1
}

get_task_result() {
    local task_id="$1"
    curl -s "$API_URL/api/v1/tasks/$task_id/result" 2>/dev/null || echo "{}"
}

evaluate_artifacts() {
    local task_id="$1"
    local result_json="$2"
    
    python3 << PYTHON_SCRIPT
import sys
import json

try:
    data = json.load(sys.stdin)
    artifacts = data.get("artifacts", {})
    
    # Extract metrics
    code_stats = artifacts.get("code_changes", {}).get("statistics", {})
    tests = artifacts.get("tests", {})
    coverage = artifacts.get("coverage", {})
    linting = artifacts.get("linting", {})
    provenance = artifacts.get("provenance", {})
    
    evaluation = {
        "task_id": "$task_id",
        "has_code_changes": code_stats.get("files_modified", 0) > 0 or code_stats.get("lines_added", 0) > 0,
        "files_modified": code_stats.get("files_modified", 0),
        "lines_added": code_stats.get("lines_added", 0),
        "lines_removed": code_stats.get("lines_removed", 0),
        "unit_tests_total": tests.get("unit_tests", {}).get("total", 0),
        "unit_tests_passed": tests.get("unit_tests", {}).get("passed", 0),
        "line_coverage": coverage.get("line_coverage", 0.0),
        "branch_coverage": coverage.get("branch_coverage", 0.0),
        "lint_errors": linting.get("errors", 0),
        "lint_warnings": linting.get("warnings", 0),
        "has_provenance": provenance.get("execution_id", "00000000-0000-0000-0000-000000000000") != "00000000-0000-0000-0000-000000000000",
        "execution_id": provenance.get("execution_id", "unknown"),
        "git_branch": provenance.get("git_info", {}).get("branch", "unknown"),
        "git_commit": provenance.get("git_info", {}).get("commit_hash", "unknown"),
    }
    
    # Calculate quality score
    quality_score = 0.0
    if evaluation["has_code_changes"]:
        quality_score += 0.3
    if evaluation["unit_tests_total"] > 0:
        quality_score += 0.2 * (evaluation["unit_tests_passed"] / evaluation["unit_tests_total"] if evaluation["unit_tests_total"] > 0 else 0)
    if evaluation["line_coverage"] > 0 or evaluation["branch_coverage"] > 0:
        quality_score += 0.2 * ((evaluation["line_coverage"] + evaluation["branch_coverage"]) / 200.0)
    if evaluation["lint_errors"] == 0:
        quality_score += 0.1
    if evaluation["has_provenance"]:
        quality_score += 0.2
    
    evaluation["quality_score"] = quality_score
    evaluation["meets_basic_requirements"] = evaluation["has_code_changes"] or evaluation["unit_tests_total"] > 0 or evaluation["has_provenance"]
    
    print(json.dumps(evaluation, indent=2))
except Exception as e:
    print(json.dumps({"error": str(e), "task_id": "$task_id"}))
PYTHON_SCRIPT
}

run_evaluation() {
    log "Starting V3 Agent Task Execution Evaluation"
    log "API URL: $API_URL"
    log "Results will be saved to: $RESULTS_FILE"
    echo ""
    
    check_server
    
    local results=()
    local task_count=0
    
    for task_desc in "${TEST_TASKS[@]}"; do
        task_count=$((task_count + 1))
        log "=========================================="
        log "Task $task_count/${#TEST_TASKS[@]}: $task_desc"
        log "=========================================="
        
        # Submit task
        local task_id=$(submit_task "$task_desc")
        if [ -z "$task_id" ]; then
            warn "Skipping task due to submission failure"
            continue
        fi
        
        log "Task ID: $task_id"
        
        # Wait for completion
        local status=$(wait_for_completion "$task_id")
        
        # Get result
        local result_json=$(get_task_result "$task_id")
        
        # Evaluate artifacts
        local evaluation=$(echo "$result_json" | evaluate_artifacts "$task_id")
        
        # Store result
        local task_result=$(python3 << PYTHON_SCRIPT
import sys
import json

task_id = "$task_id"
status = "$status"
desc = """$task_desc"""
evaluation_json = """$evaluation"""

try:
    eval_data = json.loads(evaluation_json)
    result = {
        "task_number": $task_count,
        "description": desc,
        "task_id": task_id,
        "status": status,
        "evaluation": eval_data,
        "timestamp": "$(date -u +"%Y-%m-%dT%H:%M:%SZ")"
    }
    print(json.dumps(result))
except Exception as e:
    print(json.dumps({"error": str(e), "task_id": task_id}))
PYTHON_SCRIPT
)
        
        results+=("$task_result")
        
        # Print summary
        log "Evaluation Summary:"
        echo "$evaluation" | python3 -c "import sys, json; d=json.load(sys.stdin); print(f\"  Has Code Changes: {d.get('has_code_changes', False)}\"); print(f\"  Files Modified: {d.get('files_modified', 0)}\"); print(f\"  Lines Added: {d.get('lines_added', 0)}\"); print(f\"  Unit Tests: {d.get('unit_tests_passed', 0)}/{d.get('unit_tests_total', 0)}\"); print(f\"  Quality Score: {d.get('quality_score', 0):.2f}\"); print(f\"  Meets Basic Requirements: {d.get('meets_basic_requirements', False)}\")" 2>/dev/null || true
        
        echo ""
        sleep 2  # Brief pause between tasks
    done
    
    # Generate final report
    log "Generating final report..."
    python3 << PYTHON_SCRIPT > "$RESULTS_FILE"
import sys
import json
from datetime import datetime

results_json = """$(IFS=$'\n'; echo "${results[*]}")"""

try:
    results = []
    for line in results_json.split('\n'):
        if line.strip():
            try:
                results.append(json.loads(line))
            except:
                pass
    
    total_tasks = len(results)
    completed_tasks = len([r for r in results if r.get("status") == "completed"])
    tasks_with_artifacts = len([r for r in results if r.get("evaluation", {}).get("has_code_changes", False)])
    avg_quality = sum([r.get("evaluation", {}).get("quality_score", 0) for r in results]) / total_tasks if total_tasks > 0 else 0
    
    report = {
        "evaluation_date": datetime.utcnow().isoformat() + "Z",
        "api_url": "$API_URL",
        "total_tasks": total_tasks,
        "completed_tasks": completed_tasks,
        "tasks_with_artifacts": tasks_with_artifacts,
        "average_quality_score": avg_quality,
        "completion_rate": completed_tasks / total_tasks if total_tasks > 0 else 0,
        "artifact_rate": tasks_with_artifacts / total_tasks if total_tasks > 0 else 0,
        "tasks": results
    }
    
    print(json.dumps(report, indent=2))
except Exception as e:
    print(json.dumps({"error": str(e)}))
PYTHON_SCRIPT
    
    log "Evaluation complete!"
    log "Results saved to: $RESULTS_FILE"
    
    # Print summary
    echo ""
    log "=========================================="
    log "EVALUATION SUMMARY"
    log "=========================================="
    cat "$RESULTS_FILE" | python3 -c "import sys, json; d=json.load(sys.stdin); print(f\"Total Tasks: {d.get('total_tasks', 0)}\"); print(f\"Completed: {d.get('completed_tasks', 0)}\"); print(f\"With Artifacts: {d.get('tasks_with_artifacts', 0)}\"); print(f\"Average Quality Score: {d.get('average_quality_score', 0):.2f}\"); print(f\"Completion Rate: {d.get('completion_rate', 0)*100:.1f}%\"); print(f\"Artifact Rate: {d.get('artifact_rate', 0)*100:.1f}%\")" 2>/dev/null || true
}

# Main execution
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [options]"
    echo ""
    echo "Options:"
    echo "  --api-url URL     API server URL (default: http://localhost:8080)"
    echo "  --results-dir DIR Directory for results (default: test-results/)"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  API_URL          API server URL"
    echo "  RESULTS_DIR      Results directory"
    exit 0
fi

run_evaluation

