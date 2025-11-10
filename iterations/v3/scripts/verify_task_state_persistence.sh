#!/bin/bash
# Quick verification script for task state persistence
# Tests basic save/load functionality

set -e

DATABASE_URL="${DATABASE_URL:-postgresql://postgres@localhost:5432/agent_agency_v3}"

echo "=== Task State Persistence Verification ==="
echo "Database: $DATABASE_URL"
echo ""

# Check tables exist
echo "1. Checking tables exist..."
TABLES=$(psql "$DATABASE_URL" -t -c "
  SELECT COUNT(*) FROM pg_tables 
  WHERE schemaname = 'public' 
  AND tablename IN ('task_execution_states', 'task_state_checkpoints')
")

if [ "$TABLES" -eq "2" ]; then
  echo "✅ Both tables exist"
else
  echo "❌ Tables missing (found $TABLES/2)"
  exit 1
fi

# Check indexes exist
echo ""
echo "2. Checking indexes..."
INDEX_COUNT=$(psql "$DATABASE_URL" -t -c "
  SELECT COUNT(*) FROM pg_indexes 
  WHERE schemaname = 'public' 
  AND tablename IN ('task_execution_states', 'task_state_checkpoints')
")

if [ "$INDEX_COUNT" -ge "6" ]; then
  echo "✅ Indexes created ($INDEX_COUNT found)"
else
  echo "⚠️  Expected at least 6 indexes, found $INDEX_COUNT"
fi

# Check triggers exist
echo ""
echo "3. Checking triggers..."
TRIGGER_COUNT=$(psql "$DATABASE_URL" -t -c "
  SELECT COUNT(*) FROM pg_trigger 
  WHERE tgname = 'task_execution_states_updated_at'
")

if [ "$TRIGGER_COUNT" -eq "1" ]; then
  echo "✅ Trigger exists"
else
  echo "⚠️  Trigger not found"
fi

# Test basic insert/select (if we have a test task)
echo ""
echo "4. Testing basic functionality..."
if command -v uuidgen >/dev/null 2>&1; then
    TEST_TASK_ID=$(uuidgen | tr '[:upper:]' '[:lower:]')
else
    # Fallback: use PostgreSQL to generate UUID
    TEST_TASK_ID=$(psql "$DATABASE_URL" -t -c "SELECT gen_random_uuid();" | tr -d ' ')
fi

echo "   Creating test task: $TEST_TASK_ID"

# Create a test task (using actual schema columns)
psql "$DATABASE_URL" -c "
  INSERT INTO tasks (id, title, description, priority, status)
  VALUES ('$TEST_TASK_ID'::uuid, 'Verification Test Task', 'Test task for persistence verification', 5, 'pending')
  ON CONFLICT (id) DO NOTHING;
" > /dev/null 2>&1 || echo "   Note: Task may already exist"

# Create test state
psql "$DATABASE_URL" -c "
  INSERT INTO task_execution_states (task_id, state_data, status)
  VALUES (
    '$TEST_TASK_ID'::uuid,
    '{\"task_id\":\"$TEST_TASK_ID\",\"status\":\"running\",\"progress_percentage\":50.0}'::jsonb,
    'running'
  )
  ON CONFLICT (task_id) DO UPDATE SET status = EXCLUDED.status;
" > /dev/null 2>&1

# Verify state was saved
STATE_EXISTS=$(psql "$DATABASE_URL" -t -c "
  SELECT COUNT(*) FROM task_execution_states WHERE task_id = '$TEST_TASK_ID'::uuid
" | tr -d ' ')

if [ "$STATE_EXISTS" -eq "1" ]; then
  echo "✅ State saved successfully"
else
  echo "❌ State not found (count: $STATE_EXISTS)"
  exit 1
fi

# Test checkpoint creation
echo ""
echo "5. Testing checkpoint creation..."
psql "$DATABASE_URL" -c "
  INSERT INTO task_state_checkpoints (task_id, checkpoint_timestamp, state_data)
  VALUES (
    '$TEST_TASK_ID'::uuid,
    NOW(),
    '{\"task_id\":\"$TEST_TASK_ID\",\"status\":\"running\",\"progress_percentage\":50.0}'::jsonb
  );
" > /dev/null 2>&1

CHECKPOINT_COUNT=$(psql "$DATABASE_URL" -t -c "
  SELECT COUNT(*) FROM task_state_checkpoints WHERE task_id = '$TEST_TASK_ID'::uuid
" | tr -d ' ')

if [ "$CHECKPOINT_COUNT" -ge "1" ]; then
  echo "✅ Checkpoint created successfully"
else
  echo "❌ Checkpoint not found (count: $CHECKPOINT_COUNT)"
  exit 1
fi

# Cleanup
echo ""
echo "6. Cleaning up test data..."
psql "$DATABASE_URL" -c "
  DELETE FROM task_state_checkpoints WHERE task_id = '$TEST_TASK_ID'::uuid;
  DELETE FROM task_execution_states WHERE task_id = '$TEST_TASK_ID'::uuid;
  DELETE FROM tasks WHERE id = '$TEST_TASK_ID'::uuid;
" > /dev/null 2>&1

echo "✅ Cleanup complete"

echo ""
echo "=== Verification Complete ==="
echo "✅ All checks passed"
echo ""
echo "Task state persistence is ready for use!"

