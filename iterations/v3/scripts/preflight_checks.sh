#!/bin/bash
# -----------------------------------------------------------------------------
# V3 Agentic Harness Pre-Flight Infrastructure Checks
# @author @darianrosebrook
# 
# Validates infrastructure health before running evaluation.
# MUST PASS all checks before evaluation can proceed.
#
# Checks performed:
# 1. Orchestrator service status (has_executor, available)
# 2. Database connectivity
# 3. Observability endpoints (chain-of-thought, council-decisions, worker-actions)
# 4. Baseline task execution
#
# Exit codes:
# 0 - All checks passed, safe to proceed with evaluation
# 1 - Infrastructure check failed, DO NOT proceed
# 2 - Baseline task failed, infrastructure may be unstable
# -----------------------------------------------------------------------------

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
API_URL="${API_URL:-http://localhost:8889}"
REPORT_DIR="${REPORT_DIR:-$PROJECT_ROOT/iterations/v3/test-results}"
TIMESTAMP=$(date -u +"%Y%m%dT%H%M%SZ")
REPORT_FILE="$REPORT_DIR/preflight_${TIMESTAMP}.json"

# Preflight configuration
BASELINE_TASK_TIMEOUT="${BASELINE_TASK_TIMEOUT:-120}"  # 2 minutes for baseline task
OBSERVABILITY_WAIT="${OBSERVABILITY_WAIT:-10}"  # Wait for observability data to populate

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
BOLD='\033[1m'
NC='\033[0m'

# Tracking variables (using simple arrays for compatibility)
CHECK_NAMES=""
CHECK_STATUSES=""
OVERALL_STATUS="passed"
PASSED_COUNT=0
FAILED_COUNT=0
WARNING_COUNT=0

# -----------------------------------------------------------------------------
# Logging functions
# -----------------------------------------------------------------------------

log() {
    echo -e "${GREEN}[PREFLIGHT]${NC} $1"
}

warn() {
    echo -e "${YELLOW}[PREFLIGHT] WARNING:${NC} $1"
}

error() {
    echo -e "${RED}[PREFLIGHT] ERROR:${NC} $1"
}

info() {
    echo -e "${BLUE}[PREFLIGHT]${NC} $1"
}

header() {
    echo ""
    echo -e "${BOLD}=== $1 ===${NC}"
}

check_passed() {
    local check_name="$1"
    CHECK_NAMES="${CHECK_NAMES}${check_name}|"
    CHECK_STATUSES="${CHECK_STATUSES}passed|"
    PASSED_COUNT=$((PASSED_COUNT + 1))
    echo -e "  ${GREEN}[PASS]${NC} $check_name"
}

check_failed() {
    local check_name="$1"
    local reason="${2:-Unknown failure}"
    CHECK_NAMES="${CHECK_NAMES}${check_name}|"
    CHECK_STATUSES="${CHECK_STATUSES}failed|"
    FAILED_COUNT=$((FAILED_COUNT + 1))
    OVERALL_STATUS="failed"
    echo -e "  ${RED}[FAIL]${NC} $check_name: $reason"
    error "$check_name: $reason"
}

check_warning() {
    local check_name="$1"
    local reason="${2:-}"
    CHECK_NAMES="${CHECK_NAMES}${check_name}|"
    CHECK_STATUSES="${CHECK_STATUSES}warning|"
    WARNING_COUNT=$((WARNING_COUNT + 1))
    echo -e "  ${YELLOW}[WARN]${NC} $check_name: $reason"
    warn "$check_name: $reason"
}

# -----------------------------------------------------------------------------
# Check 1: API Server Health
# -----------------------------------------------------------------------------

check_api_server() {
    header "Check 1: API Server Health"
    
    log "Testing API server at $API_URL..."
    
    # Basic health check
    local health_response
    if ! health_response=$(curl -s -f "$API_URL/health" 2>/dev/null); then
        check_failed "API Server Reachable" "Cannot connect to $API_URL/health"
        return 1
    fi
    
    check_passed "API Server Reachable"
    
    # Parse health response
    local status
    status=$(echo "$health_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('status', 'unknown'))" 2>/dev/null || echo "unknown")
    
    if [ "$status" = "ok" ] || [ "$status" = "healthy" ]; then
        check_passed "API Server Healthy (status: $status)"
    else
        check_warning "API Server Status" "Unexpected status: $status"
    fi
    
    return 0
}

# -----------------------------------------------------------------------------
# Check 2: Orchestrator Service Status
# -----------------------------------------------------------------------------

check_orchestrator_service() {
    header "Check 2: Orchestrator Service Status"
    
    log "Querying orchestrator status endpoint..."
    
    local status_response
    if ! status_response=$(curl -s -f "$API_URL/api/v1/system/orchestrator-status" 2>/dev/null); then
        check_failed "Orchestrator Status Endpoint" "Cannot reach /api/v1/system/orchestrator-status"
        return 1
    fi
    
    check_passed "Orchestrator Status Endpoint"
    
    # Parse orchestrator status
    local overall_status orchestrator_available has_executor db_connected
    
    overall_status=$(echo "$status_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('overall_status', 'unknown'))" 2>/dev/null || echo "unknown")
    orchestrator_available=$(echo "$status_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('orchestrator_service', {}).get('available', False))" 2>/dev/null || echo "False")
    has_executor=$(echo "$status_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('orchestrator_service', {}).get('has_executor', False))" 2>/dev/null || echo "False")
    db_connected=$(echo "$status_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('database', {}).get('connected', False))" 2>/dev/null || echo "False")
    
    # Check orchestrator availability
    if [ "$orchestrator_available" = "True" ]; then
        check_passed "Orchestrator Service Available"
    else
        check_failed "Orchestrator Service Available" "orchestrator_service.available is false"
    fi
    
    # Check task executor (CRITICAL for task execution)
    if [ "$has_executor" = "True" ]; then
        check_passed "Task Executor Available"
    else
        check_failed "Task Executor Available" "orchestrator_service.has_executor is false - tasks will queue but not execute"
    fi
    
    # Check database connectivity
    if [ "$db_connected" = "True" ]; then
        check_passed "Database Connected"
    else
        check_failed "Database Connected" "database.connected is false"
    fi
    
    # Overall status assessment
    if [ "$overall_status" = "healthy" ]; then
        check_passed "Overall Orchestrator Status: healthy"
    elif [ "$overall_status" = "degraded" ]; then
        check_warning "Overall Orchestrator Status" "Status is 'degraded' - some features may not work"
    else
        check_failed "Overall Orchestrator Status" "Status is '$overall_status'"
    fi
    
    # Log full status for debugging
    info "Full orchestrator status response:"
    echo "$status_response" | python3 -m json.tool 2>/dev/null || echo "$status_response"
    
    return 0
}

# -----------------------------------------------------------------------------
# Check 3: Database Connectivity (Direct Test)
# -----------------------------------------------------------------------------

check_database() {
    header "Check 3: Database Connectivity"
    
    log "Testing database connectivity via API..."
    
    # Test by querying task list (uses database and provides status_counts)
    local stats_response
    if ! stats_response=$(curl -s -f "$API_URL/api/v1/tasks" 2>/dev/null); then
        check_failed "Database Query" "Cannot query tasks endpoint"
        return 1
    fi
    
    # Verify response is valid JSON with expected structure
    local has_status_counts
    has_status_counts=$(echo "$stats_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print('yes' if 'status_counts' in d else 'no')" 2>/dev/null || echo "no")
    
    if [ "$has_status_counts" != "yes" ]; then
        check_failed "Database Query" "Invalid response structure from tasks endpoint"
        return 1
    fi
    
    check_passed "Database Query (task list)"
    
    # Parse statistics from status_counts
    local total_tasks pending_tasks failed_tasks completed_tasks
    total_tasks=$(echo "$stats_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('total', 0))" 2>/dev/null || echo "0")
    pending_tasks=$(echo "$stats_response" | python3 -c "import sys, json; d=json.load(sys.stdin); sc=d.get('status_counts', {}); print(sc.get('pending', 0))" 2>/dev/null || echo "0")
    failed_tasks=$(echo "$stats_response" | python3 -c "import sys, json; d=json.load(sys.stdin); sc=d.get('status_counts', {}); print(sc.get('failed', 0))" 2>/dev/null || echo "0")
    completed_tasks=$(echo "$stats_response" | python3 -c "import sys, json; d=json.load(sys.stdin); sc=d.get('status_counts', {}); print(sc.get('completed', 0))" 2>/dev/null || echo "0")
    
    info "Current task statistics: total=$total_tasks, pending=$pending_tasks, failed=$failed_tasks, completed=$completed_tasks"
    
    # Warning if there are many pending/failed tasks
    if [ "$pending_tasks" -gt 10 ]; then
        check_warning "Pending Tasks" "$pending_tasks pending tasks in queue - may need cleanup"
    fi
    
    if [ "$failed_tasks" -gt "$completed_tasks" ] && [ "$failed_tasks" -gt 5 ]; then
        check_warning "Failed Task Ratio" "More failed ($failed_tasks) than completed ($completed_tasks) tasks"
    fi
    
    check_passed "Database Operational"
    return 0
}

# -----------------------------------------------------------------------------
# Check 4: Observability Endpoints
# -----------------------------------------------------------------------------

check_observability_endpoints() {
    header "Check 4: Observability Endpoints"
    
    log "Testing observability endpoints with a test task ID..."
    
    # Use a dummy UUID to test endpoint availability
    local test_uuid="00000000-0000-0000-0000-000000000000"
    
    # Test chain-of-thought endpoint (correct path is /api/v1/tasks/{id}/chain-of-thought)
    local cot_status
    cot_status=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/api/v1/tasks/$test_uuid/chain-of-thought" 2>/dev/null || echo "000")
    
    if [ "$cot_status" = "404" ] || [ "$cot_status" = "200" ]; then
        # 404 is expected for non-existent task, 200 means it works
        check_passed "Chain-of-Thought Endpoint (HTTP $cot_status)"
    else
        check_failed "Chain-of-Thought Endpoint" "Unexpected HTTP status: $cot_status"
    fi
    
    # Test council-decisions endpoint (correct path is /api/v1/tasks/{id}/council-decisions)
    local council_status
    council_status=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/api/v1/tasks/$test_uuid/council-decisions" 2>/dev/null || echo "000")
    
    if [ "$council_status" = "404" ] || [ "$council_status" = "200" ]; then
        check_passed "Council Decisions Endpoint (HTTP $council_status)"
    else
        check_failed "Council Decisions Endpoint" "Unexpected HTTP status: $council_status"
    fi
    
    # Test worker-actions endpoint (correct path is /api/v1/tasks/{id}/worker-actions)
    local worker_status
    worker_status=$(curl -s -o /dev/null -w "%{http_code}" "$API_URL/api/v1/tasks/$test_uuid/worker-actions" 2>/dev/null || echo "000")
    
    if [ "$worker_status" = "404" ] || [ "$worker_status" = "200" ]; then
        check_passed "Worker Actions Endpoint (HTTP $worker_status)"
    else
        check_failed "Worker Actions Endpoint" "Unexpected HTTP status: $worker_status"
    fi
    
    return 0
}

# -----------------------------------------------------------------------------
# Check 5: Baseline Task Execution
# -----------------------------------------------------------------------------

check_baseline_task() {
    header "Check 5: Baseline Task Execution"
    
    log "Submitting baseline task: 'Create a hello world function'"
    log "This tests the full task execution pipeline..."
    
    # Submit a simple baseline task
    local submit_response
    submit_response=$(curl -s -X POST "$API_URL/api/v1/tasks" \
        -H "Content-Type: application/json" \
        -d '{"description": "Create a simple hello world function that returns the string hello world", "execution_mode": "auto", "context": "baseline_preflight_check"}' 2>/dev/null)
    
    if [ -z "$submit_response" ]; then
        check_failed "Baseline Task Submission" "Empty response from task submission"
        return 1
    fi
    
    local task_id
    task_id=$(echo "$submit_response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('task_id', ''))" 2>/dev/null || echo "")
    
    if [ -z "$task_id" ]; then
        check_failed "Baseline Task Submission" "No task_id in response: $submit_response"
        return 1
    fi
    
    check_passed "Baseline Task Submitted (ID: $task_id)"
    
    # Wait for task to complete or fail
    log "Waiting up to ${BASELINE_TASK_TIMEOUT}s for baseline task to complete..."
    
    local elapsed=0
    local check_interval=5
    local final_status="unknown"
    
    while [ $elapsed -lt "$BASELINE_TASK_TIMEOUT" ]; do
        local status_response
        # Use task detail endpoint which returns status field
        status_response=$(curl -s "$API_URL/api/v1/tasks/$task_id" 2>/dev/null || echo '{"status": "unknown"}')
        
        final_status=$(echo "$status_response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('status', 'unknown'))" 2>/dev/null || echo "unknown")
        
        if [ "$final_status" = "completed" ]; then
            check_passed "Baseline Task Completed"
            break
        elif [ "$final_status" = "failed" ]; then
            # Get error details if available
            local error_msg
            error_msg=$(echo "$status_response" | python3 -c "import sys, json; d=json.load(sys.stdin); print(d.get('error', d.get('message', 'No error details')))" 2>/dev/null || echo "Unknown error")
            check_failed "Baseline Task Execution" "Task failed: $error_msg"
            
            # Try to get more details from observability
            log "Checking observability data for failure details..."
            sleep 2
            local cot_data
            cot_data=$(curl -s "$API_URL/api/v1/tasks/$task_id/chain-of-thought" 2>/dev/null || echo '[]')
            local cot_count
            cot_count=$(echo "$cot_data" | python3 -c "import sys, json; d=json.load(sys.stdin); print(len(d) if isinstance(d, list) else 0)" 2>/dev/null || echo "0")
            
            if [ "$cot_count" = "0" ]; then
                error "No chain-of-thought entries - task may have failed before reaching agent"
            else
                info "Found $cot_count chain-of-thought entries"
            fi
            
            return 1
        fi
        
        sleep "$check_interval"
        elapsed=$((elapsed + check_interval))
        
        if [ $((elapsed % 15)) -eq 0 ]; then
            info "  Still waiting... (${elapsed}s elapsed, status: $final_status)"
        fi
    done
    
    if [ "$final_status" != "completed" ]; then
        check_failed "Baseline Task Timeout" "Task did not complete within ${BASELINE_TASK_TIMEOUT}s (final status: $final_status)"
        return 1
    fi
    
    # Verify observability data was captured
    log "Verifying observability data was captured..."
    sleep "$OBSERVABILITY_WAIT"  # Wait for data to propagate
    
    local cot_data
    cot_data=$(curl -s "$API_URL/api/v1/tasks/$task_id/chain-of-thought" 2>/dev/null || echo '{}')
    local cot_count
    # API returns {"task_id": "...", "chain_of_thought": [...]} so extract the array
    cot_count=$(echo "$cot_data" | python3 -c "import sys, json; d=json.load(sys.stdin); cot=d.get('chain_of_thought', d) if isinstance(d, dict) else d; print(len(cot) if isinstance(cot, list) else 0)" 2>/dev/null || echo "0")
    
    if [ "$cot_count" -gt 0 ]; then
        check_passed "Observability Data Captured ($cot_count chain-of-thought entries)"
    else
        check_warning "Observability Data" "No chain-of-thought entries found for completed task"
    fi
    
    return 0
}

# -----------------------------------------------------------------------------
# Generate Preflight Report
# -----------------------------------------------------------------------------

generate_report() {
    header "Preflight Report"
    
    mkdir -p "$REPORT_DIR"
    
    # Build JSON report using Python for reliability
    python3 << PYTHON_SCRIPT > "$REPORT_FILE"
import json
from datetime import datetime

report = {
    "timestamp": datetime.utcnow().isoformat() + "Z",
    "api_url": "$API_URL",
    "overall_status": "$OVERALL_STATUS",
    "passed_count": $PASSED_COUNT,
    "failed_count": $FAILED_COUNT,
    "warning_count": $WARNING_COUNT,
    "can_proceed": "$OVERALL_STATUS" == "passed"
}

print(json.dumps(report, indent=2))
PYTHON_SCRIPT

    log "Report saved to: $REPORT_FILE"
    
    # Print summary
    echo ""
    echo -e "${BOLD}=== PREFLIGHT SUMMARY ===${NC}"
    echo ""
    
    echo -e "  Checks Passed:  ${GREEN}$PASSED_COUNT${NC}"
    echo -e "  Checks Failed:  ${RED}$FAILED_COUNT${NC}"
    echo -e "  Warnings:       ${YELLOW}$WARNING_COUNT${NC}"
    echo ""
    
    if [ "$OVERALL_STATUS" = "passed" ]; then
        echo -e "${GREEN}${BOLD}PREFLIGHT PASSED${NC} - Infrastructure is ready for evaluation"
        return 0
    else
        echo -e "${RED}${BOLD}PREFLIGHT FAILED${NC} - DO NOT proceed with evaluation"
        return 1
    fi
}

# -----------------------------------------------------------------------------
# Main Execution
# -----------------------------------------------------------------------------

main() {
    echo ""
    echo -e "${BOLD}======================================${NC}"
    echo -e "${BOLD}  V3 Agentic Harness Pre-Flight Checks${NC}"
    echo -e "${BOLD}======================================${NC}"
    echo ""
    echo "Timestamp: $(date -u +"%Y-%m-%d %H:%M:%S UTC")"
    echo "API URL: $API_URL"
    echo ""
    
    # Run all checks
    check_api_server || true
    check_orchestrator_service || true
    check_database || true
    check_observability_endpoints || true
    
    # Only run baseline task if previous checks passed
    if [ "$OVERALL_STATUS" = "passed" ]; then
        check_baseline_task || true
    else
        warn "Skipping baseline task check due to prior failures"
        check_failed "Baseline Task (skipped)" "Prior infrastructure checks failed"
    fi
    
    # Generate report and return appropriate exit code
    generate_report
    
    if [ "$OVERALL_STATUS" = "passed" ]; then
        exit 0
    else
        exit 1
    fi
}

# Handle help
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
    echo "Usage: $0 [options]"
    echo ""
    echo "V3 Agentic Harness Pre-Flight Infrastructure Checks"
    echo ""
    echo "Validates infrastructure health before running evaluation."
    echo "MUST PASS all checks before evaluation can proceed."
    echo ""
    echo "Options:"
    echo "  --api-url URL     API server URL (default: http://localhost:8889)"
    echo "  --report-dir DIR  Directory for reports (default: test-results/)"
    echo "  --help, -h        Show this help message"
    echo ""
    echo "Environment Variables:"
    echo "  API_URL                 API server URL"
    echo "  REPORT_DIR              Report directory"
    echo "  BASELINE_TASK_TIMEOUT   Timeout for baseline task (default: 120s)"
    echo "  OBSERVABILITY_WAIT      Wait time for observability data (default: 10s)"
    echo ""
    echo "Exit Codes:"
    echo "  0 - All checks passed, safe to proceed with evaluation"
    echo "  1 - Infrastructure check failed, DO NOT proceed"
    echo ""
    exit 0
fi

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --api-url)
            API_URL="$2"
            shift 2
            ;;
        --report-dir)
            REPORT_DIR="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

main
