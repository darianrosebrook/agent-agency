#!/bin/bash
# V3 Test Scenario Runner
# Submits multiple test scenarios to evaluate V3 performance

set -e

API_URL="http://localhost:8889/api/v1/tasks"
SUBMITTED_FILE="submitted_tasks_v3.txt"

echo "=== V3 Test Scenario Runner ==="
echo "API URL: $API_URL"
echo "Results will be saved to: $SUBMITTED_FILE"
echo ""

# Clean up previous results
rm -f "$SUBMITTED_FILE"

# Function to submit task and capture result
submit_task() {
    local title="$1"
    local description="$2"
    local risk_tier="$3"
    local priority="$4"

    echo "🔄 Submitting: $title"

    response=$(curl -s -X POST "$API_URL" \
        -H "Content-Type: application/json" \
        -d "{
            \"title\": \"$title\",
            \"description\": \"$description\",
            \"risk_tier\": \"$risk_tier\",
            \"priority\": \"$priority\",
            \"execution_mode\": \"auto\"
        }")

    task_id=$(echo "$response" | jq -r '.task_id // empty')
    if [ -n "$task_id" ] && [ "$task_id" != "null" ]; then
        echo "✅ Task submitted successfully: $task_id"
        echo "$task_id|$title|$(date +%Y-%m-%d_%H:%M:%S)" >> "$SUBMITTED_FILE"
    else
        echo "❌ Failed to submit task"
        echo "Response: $response"
        return 1
    fi
}

echo "Submitting test scenarios..."
echo ""

# Core functionality tests
submit_task "Code Quality Analysis" "Analyze the data-infrastructure crate for code quality issues, performance bottlenecks, and maintainability concerns. Provide specific recommendations." "2" "high"

submit_task "Algorithm Optimization" "Design and implement an optimized sorting algorithm for large datasets. Compare time/space complexity with existing implementations." "2" "high"

submit_task "Security Assessment" "Perform a security assessment of the authentication system. Identify vulnerabilities and recommend fixes." "1" "critical"

# Architecture and design tests
submit_task "System Architecture Design" "Design a scalable microservices architecture for handling 100K concurrent users. Include service boundaries and communication patterns." "1" "critical"

submit_task "API Design" "Design a REST API for task management with proper error handling, pagination, and OpenAPI documentation." "2" "normal"

# Performance and reliability tests
submit_task "Performance Optimization" "Analyze and optimize database query performance. Implement indexing strategies and query improvements." "2" "high"

submit_task "Error Handling Strategy" "Design a comprehensive error handling strategy with retry mechanisms and circuit breakers." "2" "high"

# Advanced scenarios
submit_task "Data Pipeline Design" "Design a real-time data processing pipeline for IoT sensor data with validation and analytics." "2" "high"

submit_task "Machine Learning Integration" "Design ML model integration for user behavior prediction including training pipeline and monitoring." "2" "high"

submit_task "Crisis Management Plan" "Develop a crisis management plan for system outages including incident response and communication protocols." "1" "critical"

echo ""
echo "🎯 Test scenarios submitted successfully!"
echo ""
echo "Monitor execution with:"
echo "  tail -f /tmp/api-server.log | grep -E '(Task.*completed|Task.*failed|Phase.*complete|Council.*rejected)'"
echo ""
echo "Check task status:"
echo "  cat $SUBMITTED_FILE"
echo ""
echo "Individual task status check:"
echo "  curl -s http://localhost:8889/api/v1/tasks/\$(tail -1 $SUBMITTED_FILE | cut -d'|' -f1) | jq ."






