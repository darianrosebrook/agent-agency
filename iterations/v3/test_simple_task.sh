#!/bin/bash
# Simple task execution test script
# Tests the agent architecture with a real development task

set -e

API_URL="${API_URL:-http://localhost:8080}"
TASK_DESCRIPTION="${TASK_DESCRIPTION:-Create a simple Rust function that adds two numbers and returns the result. The function should be in a new file called src/math.rs with proper documentation.}"

echo "Testing Agent Architecture with Simple Task"
echo "=========================================="
echo ""
echo "Task: $TASK_DESCRIPTION"
echo "API URL: $API_URL"
echo ""

# Check if API server is running
if ! curl -s "$API_URL/health" > /dev/null 2>&1; then
    echo "⚠️  API server not running at $API_URL"
    echo "Starting API server in background..."
    cd "$(dirname "$0")"
    cargo run --bin agent-agency-api-server -- --host 127.0.0.1 --port 8080 > /tmp/api-server.log 2>&1 &
    API_PID=$!
    echo "API server started with PID: $API_PID"
    
    # Wait for server to be ready
    echo "Waiting for API server to be ready..."
    for i in {1..30}; do
        if curl -s "$API_URL/health" > /dev/null 2>&1; then
            echo "✅ API server is ready"
            break
        fi
        if [ $i -eq 30 ]; then
            echo "❌ API server failed to start after 30 seconds"
            kill $API_PID 2>/dev/null || true
            exit 1
        fi
        sleep 1
    done
fi

# Submit task
echo ""
echo "Submitting task..."
TASK_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/tasks" \
    -H "Content-Type: application/json" \
    -d "{
        \"description\": \"$TASK_DESCRIPTION\",
        \"execution_mode\": \"auto\",
        \"max_iterations\": 3,
        \"risk_tier\": 3
    }")

TASK_ID=$(echo "$TASK_RESPONSE" | jq -r '.task_id // .id // empty')

if [ -z "$TASK_ID" ] || [ "$TASK_ID" = "null" ]; then
    echo "❌ Failed to submit task"
    echo "Response: $TASK_RESPONSE"
    exit 1
fi

echo "✅ Task submitted successfully"
echo "Task ID: $TASK_ID"
echo ""

# Monitor task execution
echo "Monitoring task execution..."
MAX_WAIT=300  # 5 minutes max
ELAPSED=0
POLL_INTERVAL=2

while [ $ELAPSED -lt $MAX_WAIT ]; do
    STATUS_RESPONSE=$(curl -s "$API_URL/api/v1/tasks/$TASK_ID")
    STATUS=$(echo "$STATUS_RESPONSE" | jq -r '.status // .state // "unknown"')
    
    echo -n "Status: $STATUS"
    
    if [ "$STATUS" = "completed" ] || [ "$STATUS" = "success" ]; then
        echo ""
        echo "✅ Task completed successfully!"
        break
    elif [ "$STATUS" = "failed" ] || [ "$STATUS" = "error" ]; then
        echo ""
        echo "❌ Task failed"
        ERROR=$(echo "$STATUS_RESPONSE" | jq -r '.error_message // .error // "Unknown error"')
        echo "Error: $ERROR"
        exit 1
    fi
    
    echo " (waiting...)"
    sleep $POLL_INTERVAL
    ELAPSED=$((ELAPSED + POLL_INTERVAL))
done

if [ $ELAPSED -ge $MAX_WAIT ]; then
    echo ""
    echo "⚠️  Task execution timed out after $MAX_WAIT seconds"
    echo "Final status: $STATUS"
fi

# Get task details
echo ""
echo "Task Details:"
echo "============="
curl -s "$API_URL/api/v1/tasks/$TASK_ID" | jq '.'

# Get chain of thought if available
echo ""
echo "Chain of Thought:"
echo "================="
COT_RESPONSE=$(curl -s "$API_URL/api/v1/tasks/$TASK_ID/chain-of-thought" 2>/dev/null || echo "{}")
if [ "$COT_RESPONSE" != "{}" ]; then
    echo "$COT_RESPONSE" | jq '.'
else
    echo "No chain of thought available"
fi

# Cleanup
if [ -n "$API_PID" ]; then
    echo ""
    echo "Stopping API server (PID: $API_PID)..."
    kill $API_PID 2>/dev/null || true
fi

echo ""
echo "Test completed"
