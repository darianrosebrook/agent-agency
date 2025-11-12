#!/bin/bash
# Test script to simulate MCP tool notification call
# This simulates what the agent orchestrator would do when calling send_notification

set -e

DASHBOARD_URL="${DASHBOARD_URL:-http://localhost:3000}"

echo "🧪 Testing MCP Notification Tool Integration"
echo "=========================================="
echo ""

# Test 1: Error notification (should auto-generate voicemail)
echo "📤 Test 1: Sending error notification (auto voicemail)..."
RESPONSE1=$(curl -s -X POST "${DASHBOARD_URL}/api/notifications" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "error",
    "message": "Agent task failed: Database connection timeout after 3 retries. Need user intervention.",
    "errorCode": "DB_CONNECTION_TIMEOUT",
    "errorDetails": {
      "taskId": "task-12345",
      "retryCount": 3,
      "lastError": "Connection pool exhausted"
    },
    "actionUrl": "/tasks/task-12345",
    "actionLabel": "View Task"
  }')

echo "Response: $RESPONSE1"
echo ""

# Test 2: Warning notification (should auto-generate voicemail)
echo "📤 Test 2: Sending warning notification (auto voicemail)..."
RESPONSE2=$(curl -s -X POST "${DASHBOARD_URL}/api/notifications" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "warning",
    "message": "High memory usage detected on worker-3. Current usage: 85%. Threshold: 80%.",
    "errorCode": "HIGH_MEMORY_USAGE",
    "errorDetails": {
      "workerId": "worker-3",
      "memoryUsage": 85,
      "threshold": 80
    }
  }')

echo "Response: $RESPONSE2"
echo ""

# Test 3: Success notification with explicit voicemail request
echo "📤 Test 3: Sending success notification (explicit voicemail)..."
RESPONSE3=$(curl -s -X POST "${DASHBOARD_URL}/api/notifications" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "success",
    "message": "Task completed successfully. All tests passed. Ready for deployment.",
    "generateVoicemail": true
  }')

echo "Response: $RESPONSE3"
echo ""

# Test 4: Info notification without voicemail
echo "📤 Test 4: Sending info notification (no voicemail)..."
RESPONSE4=$(curl -s -X POST "${DASHBOARD_URL}/api/notifications" \
  -H "Content-Type: application/json" \
  -d '{
    "type": "info",
    "message": "System maintenance scheduled for tonight at 2 AM.",
    "generateVoicemail": false
  }')

echo "Response: $RESPONSE4"
echo ""

# Poll for notifications
echo "📥 Polling for notifications..."
sleep 1
POLL_RESPONSE=$(curl -s "${DASHBOARD_URL}/api/notifications/poll?lastPolled=0")
echo "Poll Response:"
echo "$POLL_RESPONSE" | python3 -m json.tool 2>/dev/null || echo "$POLL_RESPONSE"
echo ""

echo "✅ Test complete!"
echo ""
echo "Check the dashboard at ${DASHBOARD_URL}/notifications to see the notifications"
echo "Note: Voicemails will only generate if Kokoro TTS server is ready"









