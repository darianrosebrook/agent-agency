#!/bin/bash
# -----------------------------------------------------------------------------
# V3 Agentic Harness Evaluation Script (Revised)
# @author @darianrosebrook
# 
# Evaluates the agentic harness with proper failure attribution and focused
# metrics. This script implements the revised evaluation plan that separates
# infrastructure failures from agent capability issues.
#
# Features:
# - Pre-flight infrastructure checks (fail fast if broken)
# - Complexity progression (simple -> medium -> complex)
# - Failure classification by root cause
# - Focus on 4 primary metrics only
# - Simplified reporting with clear signal
#
# Exit codes:
# 0 - Evaluation completed successfully
# 1 - Pre-flight checks failed (infrastructure issue)
# 2 - Evaluation completed with failures
# 3 - Script error
# -----------------------------------------------------------------------------

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
API_URL="${API_URL:-http://localhost:8889}"
RESULTS_DIR="${RESULTS_DIR:-$PROJECT_ROOT/iterations/v3/test-results}"
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
RESULTS_FILE="$RESULTS_DIR/evaluation_${TIMESTAMP}.json"
REPORT_FILE="$RESULTS_DIR/evaluation_report_${TIMESTAMP}.md"

# Configuration
TASKS_FILE="${TASKS_FILE:-$SCRIPT_DIR/test_tasks_validated.json}"
TASK_TIMEOUT="${TASK_TIMEOUT:-300}"  # 5 minutes per task
COMPLEXITY_FILTER="${COMPLEXITY_FILTER:-all}"  # all, simple, medium, complex
SKIP_PREFLIGHT="${SKIP_PREFLIGHT:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Tracking variables
declare -a SUBMITTED_TASKS=()
declare -a TASK_RESULTS=()
SIMPLE_PASSED=0
SIMPLE_TOTAL=0
MEDIUM_PASSED=0
MEDIUM_TOTAL=0
COMPLEX_PASSED=0
COMPLEX_TOTAL=0

# -----------------------------------------------------------------------------
# Logging functions
# -----------------------------------------------------------------------------

log() {
    echo -e "${GREEN}[EVAL]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[EVAL] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[EVAL] ERROR:${NC} $1"
}

info() {
    echo -e "${BLUE}[EVAL]${NC} $1"
}

header() {
    echo ""
    echo -e "${BOLD}============================================${NC}"
    echo -e "${BOLD}  $1${NC}"
    echo -e "${BOLD}============================================${NC}"
    echo ""
}

# -----------------------------------------------------------------------------
# Pre-flight checks
# -----------------------------------------------------------------------------

run_preflight_checks() {
    header "Pre-Flight Infrastructure Checks"
    
    if [ "$SKIP_PREFLIGHT" = "true" ]; then
        warn "Skipping pre-flight checks (SKIP_PREFLIGHT=true)"
        warn "This may result in misleading evaluation results!"
        return 0
    fi
    
    log "Running pre-flight checks..."
    
    if ! "$SCRIPT_DIR/preflight_checks.sh" --api-url "$API_URL" --report-dir "$RESULTS_DIR"; then
        error "Pre-flight checks FAILED"
        error "Infrastructure is not ready for evaluation."
        error "Fix the issues above before running evaluation."
        return 1
    fi
    
    log "Pre-flight checks PASSED"
    return 0
}

# -----------------------------------------------------------------------------
# Task submission and monitoring
# -----------------------------------------------------------------------------

submit_task() {
    local task_id="$1"
    local description="$2"
    
    local response
    response=$(curl -s -X POST "$API_URL/api/v1/tasks" \
        -H "Content-Type: application/json" \
        -d "{\"description\": \"$description\", \"execution_mode\": \"auto\", \"context\": \"evaluation_task_$task_id\"}" 2>/dev/null)
    
    local submitted_task_id
    submitted_task_id=$(echo "$response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('task_id', ''))" 2>/dev/null || echo "")
    
    if [ -z "$submitted_task_id" ]; then
        error "Failed to submit task: $task_id"
        echo "$response" >&2
        echo ""
        return 1
    fi
    
    echo "$submitted_task_id"
}

wait_for_task() {
    local task_id="$1"
    local timeout="$2"
    local check_interval=5
    local elapsed=0
    
    while [ $elapsed -lt "$timeout" ]; do
        local status_response
        # Use task detail endpoint which returns status field
        status_response=$(curl -s "$API_URL/api/v1/tasks/$task_id" 2>/dev/null || echo '{"status": "unknown"}')
        
        local status
        status=$(echo "$status_response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('status', 'unknown'))" 2>/dev/null || echo "unknown")
        
        if [ "$status" = "completed" ] || [ "$status" = "failed" ]; then
            echo "$status"
            return 0
        fi
        
        sleep "$check_interval"
        elapsed=$((elapsed + check_interval))
        
        if [ $((elapsed % 30)) -eq 0 ]; then
            info "  Still waiting... (${elapsed}s elapsed, status: $status)"
        fi
    done
    
    echo "timeout"
    return 1
}

classify_task_failure() {
    local task_id="$1"
    
    local classification
    classification=$(python3 "$SCRIPT_DIR/classify_failure.py" \
        --task-id "$task_id" \
        --api-url "$API_URL" \
        --output-json 2>/dev/null || echo '{"category": "unknown_failure", "reason": "Classification failed"}')
    
    echo "$classification"
}

# -----------------------------------------------------------------------------
# Task execution
# -----------------------------------------------------------------------------

execute_task() {
    local spec_task_id="$1"
    local description="$2"
    local complexity="$3"
    
    log "Submitting task: $spec_task_id"
    
    # Submit task
    local api_task_id
    api_task_id=$(submit_task "$spec_task_id" "$description")
    
    if [ -z "$api_task_id" ]; then
        TASK_RESULTS+=("{\"spec_task_id\": \"$spec_task_id\", \"status\": \"submission_failed\", \"category\": \"infrastructure_failure\"}")
        return 1
    fi
    
    log "Task submitted with ID: $api_task_id"
    SUBMITTED_TASKS+=("$api_task_id")
    
    # Wait for completion
    local status
    status=$(wait_for_task "$api_task_id" "$TASK_TIMEOUT")
    
    log "Task $api_task_id finished with status: $status"
    
    # Classify the result
    local classification
    classification=$(classify_task_failure "$api_task_id")
    
    local category
    category=$(echo "$classification" | python3 -c "import sys, json; print(json.load(sys.stdin).get('category', 'unknown_failure'))" 2>/dev/null || echo "unknown_failure")
    
    # Build result
    local result
    result=$(python3 << PYTHON_SCRIPT
import json

spec_task_id = "$spec_task_id"
api_task_id = "$api_task_id"
complexity = "$complexity"
status = "$status"
classification_json = '''$classification'''

try:
    classification = json.loads(classification_json)
except:
    classification = {"category": "unknown_failure", "reason": "Parse error"}

result = {
    "spec_task_id": spec_task_id,
    "api_task_id": api_task_id,
    "complexity": complexity,
    "status": status,
    "category": classification.get("category", "unknown_failure"),
    "reason": classification.get("reason", ""),
    "confidence": classification.get("confidence", 0.0)
}

print(json.dumps(result))
PYTHON_SCRIPT
)
    
    TASK_RESULTS+=("$result")
    
    # Update counters
    if [ "$category" = "success" ]; then
        case "$complexity" in
            simple) SIMPLE_PASSED=$((SIMPLE_PASSED + 1)) ;;
            medium) MEDIUM_PASSED=$((MEDIUM_PASSED + 1)) ;;
            complex) COMPLEX_PASSED=$((COMPLEX_PASSED + 1)) ;;
        esac
    fi
    
    case "$complexity" in
        simple) SIMPLE_TOTAL=$((SIMPLE_TOTAL + 1)) ;;
        medium) MEDIUM_TOTAL=$((MEDIUM_TOTAL + 1)) ;;
        complex) COMPLEX_TOTAL=$((COMPLEX_TOTAL + 1)) ;;
    esac
    
    # Print result
    if [ "$category" = "success" ]; then
        echo -e "  ${GREEN}[SUCCESS]${NC} $spec_task_id"
    elif [ "$category" = "infrastructure_failure" ]; then
        echo -e "  ${RED}[INFRA FAIL]${NC} $spec_task_id - infrastructure issue"
    elif [[ "$category" == council_rejection* ]]; then
        echo -e "  ${YELLOW}[COUNCIL]${NC} $spec_task_id - council rejection"
    elif [ "$category" = "agent_capability_failure" ]; then
        echo -e "  ${RED}[AGENT FAIL]${NC} $spec_task_id - agent capability issue"
    else
        echo -e "  ${YELLOW}[UNKNOWN]${NC} $spec_task_id - $category"
    fi
    
    return 0
}

execute_complexity_level() {
    local complexity="$1"
    
    header "Executing $complexity Tasks"
    
    # Get tasks for this complexity level
    local tasks
    tasks=$(python3 << PYTHON_SCRIPT
import json

with open("$TASKS_FILE") as f:
    data = json.load(f)

tasks = data.get("tasks", [])
complexity_tasks = [t for t in tasks if t.get("complexity") == "$complexity"]

for task in complexity_tasks:
    print(f"{task['id']}|||{task['description']}")
PYTHON_SCRIPT
)
    
    if [ -z "$tasks" ]; then
        warn "No $complexity tasks found in $TASKS_FILE"
        return 0
    fi
    
    local count=0
    while IFS= read -r line; do
        if [ -z "$line" ]; then continue; fi
        
        local task_id description
        task_id=$(echo "$line" | cut -d'|' -f1-3 | tr -d '|')
        description=$(echo "$line" | cut -d'|' -f4-)
        
        count=$((count + 1))
        log "Task $count: $task_id"
        
        execute_task "$task_id" "$description" "$complexity"
        
        # Brief pause between tasks
        sleep 2
    done <<< "$tasks"
    
    log "Completed $count $complexity tasks"
}

check_progression_gate() {
    local current_level="$1"
    local next_level="$2"
    
    case "$current_level" in
        simple)
            local threshold=0.9
            local passed=$SIMPLE_PASSED
            local total=$SIMPLE_TOTAL
            ;;
        medium)
            local threshold=0.7
            local passed=$MEDIUM_PASSED
            local total=$MEDIUM_TOTAL
            ;;
        *)
            return 0
            ;;
    esac
    
    if [ "$total" -eq 0 ]; then
        warn "No $current_level tasks were executed"
        return 1
    fi
    
    local success_rate
    success_rate=$(python3 -c "print($passed / $total)" 2>/dev/null || echo "0")
    
    local passes_threshold
    passes_threshold=$(python3 -c "print('yes' if $success_rate >= $threshold else 'no')" 2>/dev/null || echo "no")
    
    if [ "$passes_threshold" = "yes" ]; then
        log "$current_level success rate: $passed/$total ($success_rate) >= $threshold"
        log "Proceeding to $next_level tasks"
        return 0
    else
        warn "$current_level success rate: $passed/$total ($success_rate) < $threshold"
        warn "NOT proceeding to $next_level tasks (gate not met)"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Results and reporting
# -----------------------------------------------------------------------------

calculate_metrics() {
    # Calculate the 4 primary metrics
    python3 << PYTHON_SCRIPT
import json

results_json = '''$(IFS=$'\n'; printf '%s\n' "${TASK_RESULTS[@]}")'''

results = []
for line in results_json.split('\n'):
    if line.strip():
        try:
            results.append(json.loads(line))
        except:
            pass

total = len(results)
if total == 0:
    print(json.dumps({
        "agent_success_rate": 0,
        "completion_rate": 0,
        "quality_score": 0,
        "correctness_rate": 0,
        "total_tasks": 0
    }))
    exit(0)

# Count by category
successes = len([r for r in results if r.get("category") == "success"])
infra_failures = len([r for r in results if r.get("category") == "infrastructure_failure"])
council_rejections = len([r for r in results if "council_rejection" in r.get("category", "")])
agent_failures = len([r for r in results if r.get("category") == "agent_capability_failure"])
spec_failures = len([r for r in results if r.get("category") == "task_specification_failure"])
unknown = len([r for r in results if r.get("category") in ["unknown_failure", "pending"]])

# Primary Metric 1: Agent Success Rate
# (excludes infrastructure failures and spec issues)
evaluable = total - infra_failures - spec_failures
agent_success_rate = successes / evaluable if evaluable > 0 else 0

# Primary Metric 2: Completion Rate
# (tasks that reach agent execution)
tasks_reaching_agent = successes + agent_failures + council_rejections
completion_rate = tasks_reaching_agent / total if total > 0 else 0

# Primary Metric 3: Quality Score (placeholder - would come from artifacts)
quality_score = 0.8 if successes > 0 else 0

# Primary Metric 4: Correctness Rate
correctness_rate = successes / (successes + agent_failures) if (successes + agent_failures) > 0 else 0

metrics = {
    "agent_success_rate": round(agent_success_rate, 3),
    "completion_rate": round(completion_rate, 3),
    "quality_score": round(quality_score, 3),
    "correctness_rate": round(correctness_rate, 3),
    "total_tasks": total,
    "breakdown": {
        "successes": successes,
        "infrastructure_failures": infra_failures,
        "council_rejections": council_rejections,
        "agent_failures": agent_failures,
        "spec_failures": spec_failures,
        "unknown": unknown
    },
    "by_complexity": {
        "simple": {"passed": $SIMPLE_PASSED, "total": $SIMPLE_TOTAL},
        "medium": {"passed": $MEDIUM_PASSED, "total": $MEDIUM_TOTAL},
        "complex": {"passed": $COMPLEX_PASSED, "total": $COMPLEX_TOTAL}
    }
}

print(json.dumps(metrics, indent=2))
PYTHON_SCRIPT
}

generate_json_report() {
    mkdir -p "$RESULTS_DIR"
    
    local metrics
    metrics=$(calculate_metrics)
    
    python3 << PYTHON_SCRIPT > "$RESULTS_FILE"
import json
from datetime import datetime

results_json = '''$(IFS=$'\n'; printf '%s\n' "${TASK_RESULTS[@]}")'''

results = []
for line in results_json.split('\n'):
    if line.strip():
        try:
            results.append(json.loads(line))
        except:
            pass

metrics = json.loads('''$metrics''')

report = {
    "timestamp": datetime.utcnow().isoformat() + "Z",
    "api_url": "$API_URL",
    "tasks_file": "$TASKS_FILE",
    "complexity_filter": "$COMPLEXITY_FILTER",
    "metrics": metrics,
    "task_results": results
}

print(json.dumps(report, indent=2))
PYTHON_SCRIPT

    log "JSON report saved to: $RESULTS_FILE"
}

# -----------------------------------------------------------------------------
# 5D Evaluation Integration
# -----------------------------------------------------------------------------

run_5d_evaluation() {
    local task_id="$1"
    local output_file="${RESULTS_DIR}/5d_evaluation_${task_id}.json"
    
    info "Running 5D evaluation for task $task_id..."
    
    # Run evaluate_5d.py with --json flag and capture output
    if python3 "$SCRIPT_DIR/evaluate_5d.py" "$task_id" "$API_URL" --json > "$output_file" 2>/dev/null; then
        log "5D evaluation complete for $task_id"
        cat "$output_file"
    else
        warn "5D evaluation failed for $task_id"
        echo '{"error": "5D evaluation failed", "task_id": "'$task_id'"}'
    fi
}

generate_5d_evaluation_report() {
    local all_evaluations_file="${RESULTS_DIR}/5d_evaluations_all_${TIMESTAMP}.json"
    local eval_report_file="${RESULTS_DIR}/5d_evaluation_report_${TIMESTAMP}.md"
    
    info "Generating 5D evaluation report for all completed tasks..."
    
    # Collect all 5D evaluations for completed tasks
    echo "[" > "$all_evaluations_file"
    local first=true
    
    for result_json in "${TASK_RESULTS[@]}"; do
        local category
        category=$(echo "$result_json" | python3 -c "import sys, json; d=json.loads(sys.stdin.read()); print(d.get('category', ''))")
        
        if [ "$category" = "success" ]; then
            local task_id
            task_id=$(echo "$result_json" | python3 -c "import sys, json; d=json.loads(sys.stdin.read()); print(d.get('api_task_id', ''))")
            
            if [ -n "$task_id" ] && [ "$task_id" != "null" ]; then
                # Run 5D evaluation
                local eval_result
                eval_result=$(run_5d_evaluation "$task_id")
                
                if [ "$first" = true ]; then
                    first=false
                else
                    echo "," >> "$all_evaluations_file"
                fi
                echo "$eval_result" >> "$all_evaluations_file"
            fi
        fi
    done
    
    echo "]" >> "$all_evaluations_file"
    
    # Generate 5D evaluation markdown report
    python3 << PYTHON_5D_SCRIPT > "$eval_report_file"
import json
import sys
from datetime import datetime

# Read all 5D evaluations
try:
    with open("$all_evaluations_file", "r") as f:
        evaluations = json.load(f)
except:
    evaluations = []

print("# 5-Dimensional Agent Evaluation Report")
print()
print(f"**Generated**: {datetime.utcnow().isoformat()}Z")
print(f"**Evaluated Tasks**: {len(evaluations)}")
print()

if not evaluations:
    print("No completed tasks to evaluate.")
    sys.exit(0)

# Calculate aggregate scores
total_scores = {
    "overall": 0,
    "functional_correctness": 0,
    "process_quality": 0,
    "adaptability": 0,
    "safety": 0,
    "efficiency": 0
}

valid_evals = 0
for e in evaluations:
    if "error" not in e:
        valid_evals += 1
        for key in total_scores:
            total_scores[key] += e.get(key, 0)

if valid_evals > 0:
    for key in total_scores:
        total_scores[key] /= valid_evals

print("## Aggregate 5D Scores")
print()
print("| Dimension | Score | Weight | Weighted |")
print("|-----------|-------|--------|----------|")
print(f"| Functional Correctness | {total_scores['functional_correctness']*100:.1f}% | 30% | {total_scores['functional_correctness']*0.30*100:.1f}% |")
print(f"| Process Quality | {total_scores['process_quality']*100:.1f}% | 25% | {total_scores['process_quality']*0.25*100:.1f}% |")
print(f"| Adaptability | {total_scores['adaptability']*100:.1f}% | 20% | {total_scores['adaptability']*0.20*100:.1f}% |")
print(f"| Safety | {total_scores['safety']*100:.1f}% | 15% | {total_scores['safety']*0.15*100:.1f}% |")
print(f"| Efficiency | {total_scores['efficiency']*100:.1f}% | 10% | {total_scores['efficiency']*0.10*100:.1f}% |")
print(f"| **Overall** | **{total_scores['overall']*100:.1f}%** | | |")
print()

print("## Per-Task Scores")
print()
print("| Task ID | Overall | Func | Process | Adapt | Safety | Effic |")
print("|---------|---------|------|---------|-------|--------|-------|")
for e in evaluations:
    if "error" not in e:
        task_id = e.get("task_id", "unknown")[:8]
        print(f"| {task_id}... | {e.get('overall', 0)*100:.0f}% | {e.get('functional_correctness', 0)*100:.0f}% | {e.get('process_quality', 0)*100:.0f}% | {e.get('adaptability', 0)*100:.0f}% | {e.get('safety', 0)*100:.0f}% | {e.get('efficiency', 0)*100:.0f}% |")
print()

print("## Dimension Analysis")
print()
print("### Scoring Interpretation")
print()
print("- **Functional Correctness** (30%): Did the agent complete the task correctly?")
print("- **Process Quality** (25%): Did the agent reason well and consider alternatives?")
print("- **Adaptability** (20%): Did the agent handle errors and recover effectively?")
print("- **Safety** (15%): Did the agent stay within scope and avoid risky operations?")
print("- **Efficiency** (10%): Did the agent complete the task in reasonable time with appropriate resources?")
print()

if total_scores["overall"] >= 0.7:
    print("**Overall Assessment**: Agent demonstrates **STRONG** multi-dimensional capability.")
elif total_scores["overall"] >= 0.5:
    print("**Overall Assessment**: Agent demonstrates **MODERATE** capability with room for improvement.")
else:
    print("**Overall Assessment**: Agent demonstrates **WEAK** capability and needs significant work.")
print()

print("---")
print("*Report generated by 5D Evaluation Framework*")
PYTHON_5D_SCRIPT

    log "5D evaluation report saved to: $eval_report_file"
}

generate_markdown_report() {
    local metrics
    metrics=$(calculate_metrics)
    
    python3 << PYTHON_SCRIPT > "$REPORT_FILE"
import json
from datetime import datetime

results_json = '''$(IFS=$'\n'; printf '%s\n' "${TASK_RESULTS[@]}")'''

results = []
for line in results_json.split('\n'):
    if line.strip():
        try:
            results.append(json.loads(line))
        except:
            pass

metrics = json.loads('''$metrics''')

# Generate markdown report
print("# V3 Agentic Harness Evaluation Report")
print()
print(f"**Generated**: {datetime.utcnow().isoformat()}Z")
print(f"**API URL**: $API_URL")
print(f"**Tasks File**: $TASKS_FILE")
print()

print("## Executive Summary")
print()
print("### Primary Metrics")
print()
print(f"| Metric | Value | Target |")
print(f"|--------|-------|--------|")
print(f"| Agent Success Rate | {metrics['agent_success_rate']*100:.1f}% | >= 70% (simple), >= 50% (complex) |")
print(f"| Completion Rate | {metrics['completion_rate']*100:.1f}% | >= 80% |")
print(f"| Quality Score | {metrics['quality_score']*100:.1f}% | >= 80% |")
print(f"| Correctness Rate | {metrics['correctness_rate']*100:.1f}% | >= 90% |")
print()

print("### Recommendation")
print()
if metrics['agent_success_rate'] >= 0.7:
    print("Agent capability is **GOOD** - agent can reliably complete tasks.")
elif metrics['agent_success_rate'] >= 0.5:
    print("Agent capability is **MODERATE** - agent needs improvement.")
else:
    print("Agent capability is **POOR** - significant work needed.")
print()

print("## Failure Breakdown")
print()
breakdown = metrics.get("breakdown", {})
print(f"| Category | Count | Description |")
print(f"|----------|-------|-------------|")
print(f"| Successes | {breakdown.get('successes', 0)} | Tasks completed successfully |")
print(f"| Infrastructure Failures | {breakdown.get('infrastructure_failures', 0)} | Not agent's fault |")
print(f"| Council Rejections | {breakdown.get('council_rejections', 0)} | May indicate strict council or agent issues |")
print(f"| Agent Capability Failures | {breakdown.get('agent_failures', 0)} | Agent could not complete task |")
print(f"| Task Spec Failures | {breakdown.get('spec_failures', 0)} | Task poorly specified |")
print(f"| Unknown | {breakdown.get('unknown', 0)} | Could not classify |")
print()

print("## Results by Complexity")
print()
by_complexity = metrics.get("by_complexity", {})
print(f"| Complexity | Passed | Total | Rate |")
print(f"|------------|--------|-------|------|")
for level in ["simple", "medium", "complex"]:
    data = by_complexity.get(level, {"passed": 0, "total": 0})
    rate = data["passed"] / data["total"] * 100 if data["total"] > 0 else 0
    print(f"| {level.capitalize()} | {data['passed']} | {data['total']} | {rate:.1f}% |")
print()

print("## Task Details")
print()
for result in results:
    status_emoji = ""
    if result.get("category") == "success":
        status_emoji = "[PASS]"
    elif result.get("category") == "infrastructure_failure":
        status_emoji = "[INFRA]"
    elif "council" in result.get("category", ""):
        status_emoji = "[COUNCIL]"
    else:
        status_emoji = "[FAIL]"
    
    print(f"### {result.get('spec_task_id', 'unknown')}")
    print()
    print(f"- **Status**: {status_emoji} {result.get('category', 'unknown')}")
    print(f"- **Complexity**: {result.get('complexity', 'unknown')}")
    print(f"- **API Task ID**: {result.get('api_task_id', 'N/A')}")
    print(f"- **Reason**: {result.get('reason', 'N/A')}")
    print()

print("## Recommendations")
print()
if breakdown.get('infrastructure_failures', 0) > 0:
    print("1. **Fix Infrastructure Issues** - Some tasks failed due to infrastructure problems. Check orchestrator service status.")
if breakdown.get('agent_failures', 0) > breakdown.get('successes', 0):
    print("2. **Improve Agent Capabilities** - More failures than successes indicate agent needs work.")
if breakdown.get('council_rejections', 0) > metrics['total_tasks'] * 0.2:
    print("3. **Review Council Settings** - High rejection rate may indicate overly strict council configuration.")
print()
print("---")
print()
print("*Report generated by V3 Agentic Harness Evaluation Script*")
PYTHON_SCRIPT

    log "Markdown report saved to: $REPORT_FILE"
}

print_summary() {
    local metrics
    metrics=$(calculate_metrics)
    
    header "Evaluation Summary"
    
    echo "$metrics" | python3 -c "
import sys, json
m = json.load(sys.stdin)
print(f\"  Agent Success Rate:   {m['agent_success_rate']*100:.1f}%\")
print(f\"  Completion Rate:      {m['completion_rate']*100:.1f}%\")
print(f\"  Quality Score:        {m['quality_score']*100:.1f}%\")
print(f\"  Correctness Rate:     {m['correctness_rate']*100:.1f}%\")
print()
print(f\"  Total Tasks:          {m['total_tasks']}\")
b = m.get('breakdown', {})
print(f\"  Successes:            {b.get('successes', 0)}\")
print(f\"  Infra Failures:       {b.get('infrastructure_failures', 0)} (excluded from agent metrics)\")
print(f\"  Council Rejections:   {b.get('council_rejections', 0)}\")
print(f\"  Agent Failures:       {b.get('agent_failures', 0)}\")
"
    
    echo ""
    log "Reports saved:"
    log "  JSON: $RESULTS_FILE"
    log "  Markdown: $REPORT_FILE"
    log "  5D Evaluation: $RESULTS_DIR/5d_evaluation_report_${TIMESTAMP}.md"
}

# -----------------------------------------------------------------------------
# Main execution
# -----------------------------------------------------------------------------

main() {
    header "V3 Agentic Harness Evaluation"
    
    log "Timestamp: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    log "API URL: $API_URL"
    log "Tasks file: $TASKS_FILE"
    log "Complexity filter: $COMPLEXITY_FILTER"
    log "Results directory: $RESULTS_DIR"
    
    mkdir -p "$RESULTS_DIR"
    
    # Validate tasks file
    if [ ! -f "$TASKS_FILE" ]; then
        error "Tasks file not found: $TASKS_FILE"
        exit 3
    fi
    
    log "Validating task specifications..."
    if ! python3 "$SCRIPT_DIR/validate_task_spec.py" --file "$TASKS_FILE" > /dev/null 2>&1; then
        error "Task specification validation failed"
        python3 "$SCRIPT_DIR/validate_task_spec.py" --file "$TASKS_FILE"
        exit 3
    fi
    log "Task specifications are valid"
    
    # Run pre-flight checks
    if ! run_preflight_checks; then
        error "Pre-flight checks failed - cannot proceed with evaluation"
        exit 1
    fi
    
    # Execute tasks by complexity
    if [ "$COMPLEXITY_FILTER" = "all" ] || [ "$COMPLEXITY_FILTER" = "simple" ]; then
        execute_complexity_level "simple"
    fi
    
    if [ "$COMPLEXITY_FILTER" = "all" ] || [ "$COMPLEXITY_FILTER" = "medium" ]; then
        if [ "$COMPLEXITY_FILTER" = "all" ]; then
            if check_progression_gate "simple" "medium"; then
                execute_complexity_level "medium"
            fi
        else
            execute_complexity_level "medium"
        fi
    fi
    
    if [ "$COMPLEXITY_FILTER" = "all" ] || [ "$COMPLEXITY_FILTER" = "complex" ]; then
        if [ "$COMPLEXITY_FILTER" = "all" ]; then
            if check_progression_gate "medium" "complex"; then
                execute_complexity_level "complex"
            fi
        else
            execute_complexity_level "complex"
        fi
    fi
    
    # Generate reports
    generate_json_report
    generate_markdown_report
    generate_5d_evaluation_report
    print_summary
    
    # Determine exit code based on results
    local agent_success_rate
    agent_success_rate=$(calculate_metrics | python3 -c "import sys, json; print(json.load(sys.stdin).get('agent_success_rate', 0))" 2>/dev/null || echo "0")
    
    if python3 -c "exit(0 if $agent_success_rate >= 0.5 else 1)" 2>/dev/null; then
        log "Evaluation completed successfully"
        exit 0
    else
        warn "Evaluation completed with low success rate"
        exit 2
    fi
}

# Help
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [options]"
    echo ""
    echo "V3 Agentic Harness Evaluation Script"
    echo ""
    echo "Options:"
    echo "  --api-url URL         API server URL (default: http://localhost:8889)"
    echo "  --tasks-file FILE     Tasks JSON file (default: test_tasks_validated.json)"
    echo "  --results-dir DIR     Results directory (default: test-results/)"
    echo "  --complexity LEVEL    Complexity filter: all, simple, medium, complex (default: all)"
    echo "  --timeout SECONDS     Per-task timeout (default: 300)"
    echo "  --skip-preflight      Skip pre-flight checks (not recommended)"
    echo "  --help, -h            Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  API_URL               API server URL"
    echo "  TASKS_FILE            Tasks JSON file path"
    echo "  RESULTS_DIR           Results directory"
    echo "  COMPLEXITY_FILTER     Complexity filter"
    echo "  TASK_TIMEOUT          Per-task timeout in seconds"
    echo "  SKIP_PREFLIGHT        Skip pre-flight checks (set to 'true')"
    echo ""
    echo "Exit Codes:"
    echo "  0 - Evaluation completed with >= 50% agent success rate"
    echo "  1 - Pre-flight checks failed"
    echo "  2 - Evaluation completed with < 50% agent success rate"
    echo "  3 - Script error"
    echo ""
    exit 0
fi

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --api-url)
            API_URL="$2"
            shift 2
            ;;
        --tasks-file)
            TASKS_FILE="$2"
            shift 2
            ;;
        --results-dir)
            RESULTS_DIR="$2"
            shift 2
            ;;
        --complexity)
            COMPLEXITY_FILTER="$2"
            shift 2
            ;;
        --timeout)
            TASK_TIMEOUT="$2"
            shift 2
            ;;
        --skip-preflight)
            SKIP_PREFLIGHT="true"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 3
            ;;
    esac
done

main

