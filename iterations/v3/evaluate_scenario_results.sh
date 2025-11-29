#!/bin/bash
# V3 Test Scenario Results Evaluator

set -e

SUBMITTED_FILE="submitted_tasks_v3.txt"
RESULTS_FILE="scenario_evaluation_results.txt"

echo "=== V3 Test Scenario Evaluation ==="
echo ""

if [ ! -f "$SUBMITTED_FILE" ]; then
    echo "❌ No submitted tasks file found. Run run_test_scenarios.sh first."
    exit 1
fi

# Clean previous results
rm -f "$RESULTS_FILE"

echo "Evaluating submitted scenarios..."
echo ""

# Function to evaluate a single task
evaluate_task() {
    local task_line="$1"
    local task_id=$(echo "$task_line" | cut -d'|' -f1)
    local title=$(echo "$task_line" | cut -d'|' -f2)
    local submitted_time=$(echo "$task_line" | cut -d'|' -f3)

    echo "🔍 Evaluating: $title"
    echo "   Task ID: $task_id"
    echo "   Submitted: $submitted_time"

    # Get task status
    status_response=$(curl -s "http://localhost:8080/api/v1/tasks/$task_id" 2>/dev/null)
    if [ $? -ne 0 ] || [ -z "$status_response" ]; then
        echo "   ❌ Status: Unable to fetch"
        echo "$task_id|$title|FAILED|Unable to fetch status" >> "$RESULTS_FILE"
        echo ""
        return
    fi

    # Parse status
    status=$(echo "$status_response" | jq -r '.status // "unknown"' 2>/dev/null)
    started_at=$(echo "$status_response" | jq -r '.started_at // empty' 2>/dev/null)
    updated_at=$(echo "$status_response" | jq -r '.updated_at // empty' 2>/dev/null)

    echo "   Status: $status"

    if [ "$status" = "completed" ]; then
        echo "   ✅ Result: SUCCESS"
        echo "   Started: $started_at"
        echo "   Completed: $updated_at"
        echo "$task_id|$title|SUCCESS|Completed successfully" >> "$RESULTS_FILE"
    elif [ "$status" = "failed" ]; then
        echo "   ❌ Result: FAILED"
        echo "   Started: $started_at"
        echo "   Failed: $updated_at"
        echo "$task_id|$title|FAILED|Execution failed" >> "$RESULTS_FILE"
    elif [ "$status" = "running" ]; then
        echo "   🔄 Result: IN PROGRESS"
        echo "   Started: $started_at"
        echo "   Last Update: $updated_at"
        echo "$task_id|$title|RUNNING|Still executing" >> "$RESULTS_FILE"
    else
        echo "   ❓ Result: $status"
        echo "   Last Update: $updated_at"
        echo "$task_id|$title|$status|Unknown status" >> "$RESULTS_FILE"
    fi

    echo ""
}

# Evaluate all tasks
while IFS= read -r task_line; do
    if [ -n "$task_line" ] && [[ ! "$task_line" =~ ^[[:space:]]*# ]]; then
        evaluate_task "$task_line"
    fi
done < "$SUBMITTED_FILE"

# Generate summary
echo "=== Evaluation Summary ==="
echo ""

if [ -f "$RESULTS_FILE" ]; then
    total_tasks=$(wc -l < "$SUBMITTED_FILE")
    successful_tasks=$(grep "|SUCCESS|" "$RESULTS_FILE" | wc -l)
    failed_tasks=$(grep "|FAILED|" "$RESULTS_FILE" | wc -l)
    running_tasks=$(grep "|RUNNING|" "$RESULTS_FILE" | wc -l)

    echo "📊 Overall Results:"
    echo "   Total Tasks: $total_tasks"
    echo "   ✅ Successful: $successful_tasks"
    echo "   ❌ Failed: $failed_tasks"
    echo "   🔄 Still Running: $running_tasks"

    if [ "$total_tasks" -gt 0 ]; then
        success_rate=$((successful_tasks * 100 / total_tasks))
        echo "   Success Rate: ${success_rate}%"
    fi

    echo ""
    echo "📋 Detailed Results:"
    cat "$RESULTS_FILE"
else
    echo "❌ No evaluation results generated"
fi

echo ""
echo "📄 Full results saved to: $RESULTS_FILE"
echo ""
echo "🔍 Check server logs for detailed execution information:"
echo "   tail -f /tmp/api-server.log | grep -E '(Task.*completed|Task.*failed|Phase.*complete)'"
