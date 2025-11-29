#!/bin/bash
# Task Execution Evaluation Script
# Monitors task execution and collects evaluation data

set -e

API_URL="http://localhost:8080"
TASK_ID="${1:-}"

if [ -z "$TASK_ID" ]; then
    echo "Usage: $0 <task_id>"
    echo "Or submit a new task: $0"
    exit 1
fi

echo "=== Task Execution Evaluation ==="
echo "Task ID: $TASK_ID"
echo ""

# Function to check task status
check_task_status() {
    local task_id=$1
    curl -s "${API_URL}/api/v1/tasks/${task_id}" | jq .
}

# Function to get task observations
get_observations() {
    local task_id=$1
    curl -s "${API_URL}/api/v1/tasks/${task_id}/observations" 2>/dev/null | jq . || echo "No observations endpoint or no data"
}

# Function to get execution plan
get_execution_plan() {
    local task_id=$1
    curl -s "${API_URL}/api/v1/tasks/${task_id}/plan" 2>/dev/null | jq . || echo "No plan endpoint or no data"
}

# Monitor task for up to 5 minutes
echo "Monitoring task execution..."
for i in {1..60}; do
    echo ""
    echo "--- Check $i ($(date +%H:%M:%S)) ---"
    
    STATUS=$(check_task_status "$TASK_ID" | jq -r '.status // "unknown"')
    PROGRESS=$(check_task_status "$TASK_ID" | jq -r '.progress_percentage // 0')
    
    echo "Status: $STATUS"
    echo "Progress: ${PROGRESS}%"
    
    if [ "$STATUS" = "completed" ] || [ "$STATUS" = "failed" ]; then
        echo ""
        echo "=== Task Finished ==="
        check_task_status "$TASK_ID" | jq .
        break
    fi
    
    # Get observations if available
    OBS=$(get_observations "$TASK_ID" 2>/dev/null)
    if [ "$OBS" != "No observations endpoint or no data" ] && [ -n "$OBS" ]; then
        echo "Observations:"
        echo "$OBS" | jq . | head -20
    fi
    
    sleep 5
done

echo ""
echo "=== Final Task State ==="
check_task_status "$TASK_ID" | jq .

echo ""
echo "=== Observations ==="
get_observations "$TASK_ID"

echo ""
echo "=== Execution Plan ==="
get_execution_plan "$TASK_ID"


