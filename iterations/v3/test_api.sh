#!/bin/bash

BASE_URL="http://127.0.0.1:8090"

echo "Checking Health..."
curl -s "${BASE_URL}/health" | jq .
echo ""

echo "Checking Metrics..."
curl -s "${BASE_URL}/metrics" | jq .
echo ""

echo "Listing Tasks..."
curl -s "${BASE_URL}/api/v1/tasks" | jq .
echo ""

echo "Submitting a Task..."
curl -s -X POST "${BASE_URL}/api/v1/tasks" \
  -H "Content-Type: application/json" \
  -d '{
    "description": "Test task from evaluation script",
    "priority": "normal"
  }' | jq .
echo ""
