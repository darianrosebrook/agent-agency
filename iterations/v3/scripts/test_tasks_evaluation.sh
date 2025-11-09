#!/bin/bash
# V3 Agent Task Evaluation Script
# Submits multiple tasks and monitors their execution

API_URL="http://localhost:8080"
TASK_IDS=()

# Function to submit a task and return task_id
submit_task() {
    local description="$1"
    local task_response=$(curl -s -X POST "${API_URL}/api/v1/tasks" \
        -H "Content-Type: application/json" \
        -d "{\"description\": \"${description}\", \"execution_mode\": \"auto\"}")
    
    local task_id=$(echo "$task_response" | python3 -c "import sys, json; print(json.load(sys.stdin).get('task_id', ''))" 2>/dev/null)
    echo "$task_id"
}

# Function to check task status
check_task_status() {
    local task_id="$1"
    curl -s "${API_URL}/api/v1/tasks/${task_id}/status" | python3 -m json.tool 2>/dev/null || curl -s "${API_URL}/api/v1/tasks/${task_id}/status"
}

# Function to wait for task completion
wait_for_completion() {
    local task_id="$1"
    local max_wait=300  # 5 minutes max
    local elapsed=0
    
    echo "Waiting for task ${task_id} to complete..."
    
    while [ $elapsed -lt $max_wait ]; do
        local status=$(check_task_status "$task_id" | python3 -c "import sys, json; print(json.load(sys.stdin).get('status', 'unknown'))" 2>/dev/null || echo "unknown")
        
        if [ "$status" = "completed" ] || [ "$status" = "failed" ]; then
            echo "Task ${task_id} finished with status: ${status}"
            return 0
        fi
        
        sleep 10
        elapsed=$((elapsed + 10))
        echo "  Elapsed: ${elapsed}s, Status: ${status}"
    done
    
    echo "Task ${task_id} timed out after ${max_wait}s"
    return 1
}

# Submit Task 1: Simple Code Generation
echo "=== Submitting Task 1: Simple Code Generation ==="
TASK1_ID=$(submit_task "Create a Python function that calculates fibonacci numbers")
echo "Task 1 ID: ${TASK1_ID}"
TASK_IDS+=("$TASK1_ID")

# Wait for Task 1 to complete before submitting next
wait_for_completion "$TASK1_ID"

# Submit Task 2: Documentation
echo ""
echo "=== Submitting Task 2: Documentation ==="
TASK2_ID=$(submit_task "Create comprehensive API documentation for the user service")
echo "Task 2 ID: ${TASK2_ID}"
TASK_IDS+=("$TASK2_ID")

wait_for_completion "$TASK2_ID"

# Submit Task 3: Test Writing
echo ""
echo "=== Submitting Task 3: Test Writing ==="
TASK3_ID=$(submit_task "Write unit tests for the payment processing module")
echo "Task 3 ID: ${TASK3_ID}"
TASK_IDS+=("$TASK3_ID")

wait_for_completion "$TASK3_ID"

# Submit Task 4: Bug Fix
echo ""
echo "=== Submitting Task 4: Bug Fix ==="
TASK4_ID=$(submit_task "Fix the memory leak in the data processing pipeline")
echo "Task 4 ID: ${TASK4_ID}"
TASK_IDS+=("$TASK4_ID")

wait_for_completion "$TASK4_ID"

# Submit Task 5: Feature Implementation
echo ""
echo "=== Submitting Task 5: Feature Implementation ==="
TASK5_ID=$(submit_task "Add user profile editing functionality with validation")
echo "Task 5 ID: ${TASK5_ID}"
TASK_IDS+=("$TASK5_ID")

wait_for_completion "$TASK5_ID"

# Print summary
echo ""
echo "=== Task Submission Summary ==="
echo "Total tasks submitted: ${#TASK_IDS[@]}"
for i in "${!TASK_IDS[@]}"; do
    echo "Task $((i+1)): ${TASK_IDS[$i]}"
done

# Save task IDs to file
echo "${TASK_IDS[@]}" > /tmp/v3_task_ids.txt
echo "Task IDs saved to /tmp/v3_task_ids.txt"

