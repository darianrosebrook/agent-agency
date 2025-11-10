# Task State Persistence Testing Guide

**Date:** 2025-01-28  
**Purpose:** Guide for testing task state persistence implementation

---

## Overview

This guide covers testing the database-backed task state persistence implementation, including unit tests, integration tests, and end-to-end verification.

---

## Prerequisites

### Database Setup

1. **PostgreSQL Database Required**
   ```bash
   # Set DATABASE_URL environment variable
   export DATABASE_URL="postgresql://localhost:5432/agent_agency_test"
   ```

2. **Run Migrations**
   ```bash
   cd iterations/v3/data-infrastructure
   psql $DATABASE_URL -f migrations/020_create_task_state_persistence.sql
   ```

3. **Verify Tables Created**
   ```sql
   -- Connect to database
   psql $DATABASE_URL
   
   -- Check tables exist
   \dt task_execution_states
   \dt task_state_checkpoints
   
   -- Check indexes
   \d task_execution_states
   \d task_state_checkpoints
   ```

---

## Running Tests

### Unit Tests (In-Memory)

Tests the in-memory implementation (no database required):

```bash
cd iterations/v3/agent-orchestration
cargo test test_state_persistence_basic_operations
```

### Integration Tests (Database Required)

Tests the database-backed implementation:

```bash
cd iterations/v3/agent-orchestration

# Set database URL
export DATABASE_URL="postgresql://localhost:5432/agent_agency_test"

# Run all integration tests (requires --ignored flag for database tests)
cargo test --test integration_task_state_persistence -- --ignored

# Run specific test
cargo test --test integration_task_state_persistence test_database_persistence_save_and_load -- --ignored
```

### Test Coverage

**Available Tests:**

1. **`test_database_persistence_save_and_load`**
   - Tests basic save and load operations
   - Verifies state serialization/deserialization
   - Validates state integrity

2. **`test_database_persistence_list_resumable_tasks`**
   - Tests resumable task detection
   - Verifies status filtering (paused, crashed, running)
   - Excludes completed tasks

3. **`test_database_persistence_has_resumable_state`**
   - Tests resumable state checking
   - Handles nonexistent tasks
   - Validates status-based logic

4. **`test_database_persistence_checkpoints`**
   - Tests checkpoint creation
   - Verifies checkpoint listing
   - Validates timestamp ordering

5. **`test_database_persistence_delete_state`**
   - Tests state deletion
   - Verifies checkpoint cleanup
   - Validates cascading deletes

6. **`test_database_persistence_update_state`**
   - Tests state updates
   - Verifies field changes persist
   - Validates upsert behavior

7. **`test_database_persistence_crashed_state_resumable`**
   - Tests crashed state handling
   - Verifies crashed tasks are resumable
   - Validates recovery capability

8. **`test_database_persistence_multiple_tasks`**
   - Tests concurrent task handling
   - Verifies isolation between tasks
   - Validates bulk operations

---

## End-to-End Verification

### Manual E2E Test: Task Resumption

**Goal:** Verify that a task can be interrupted and resumed from saved state.

**Steps:**

1. **Start API Server**
   ```bash
   cd iterations/v3/data-interfaces-adapters
   export DATABASE_URL="postgresql://localhost:5432/agent_agency_v3"
   cargo run --bin api-server
   ```

2. **Submit a Task**
   ```bash
   curl -X POST http://localhost:8080/api/v1/tasks \
     -H "Content-Type: application/json" \
     -d '{
       "description": "Test task for state persistence",
       "execution_mode": "default"
     }'
   ```
   
   **Note the `task_id` from response**

3. **Verify State Saved**
   ```sql
   -- Connect to database
   psql $DATABASE_URL
   
   -- Check state exists
   SELECT task_id, status, last_updated 
   FROM task_execution_states 
   WHERE task_id = '<task_id>';
   ```

4. **Simulate Interruption**
   - Stop the API server (Ctrl+C)
   - Or pause the task via API (if pause endpoint exists)

5. **Restart API Server**
   ```bash
   cargo run --bin api-server
   ```

6. **Verify Resumable Tasks**
   ```bash
   # Check resumable tasks (if API endpoint exists)
   curl http://localhost:8080/api/v1/tasks/resumable
   ```

7. **Resume Task** (if resume endpoint exists)
   ```bash
   curl -X POST http://localhost:8080/api/v1/tasks/<task_id>/resume
   ```

### Automated E2E Test Script

Create a test script to automate the above:

```bash
#!/bin/bash
# test_task_resumption.sh

set -e

DATABASE_URL="${DATABASE_URL:-postgresql://localhost:5432/agent_agency_test}"
API_URL="${API_URL:-http://localhost:8080}"

echo "=== Task State Persistence E2E Test ==="

# 1. Submit task
echo "1. Submitting task..."
TASK_RESPONSE=$(curl -s -X POST "$API_URL/api/v1/tasks" \
  -H "Content-Type: application/json" \
  -d '{"description": "E2E test task", "execution_mode": "default"}')

TASK_ID=$(echo "$TASK_RESPONSE" | jq -r '.task_id')
echo "Task ID: $TASK_ID"

# 2. Wait for state to be saved
echo "2. Waiting for state to be saved..."
sleep 2

# 3. Verify state in database
echo "3. Verifying state in database..."
STATE_EXISTS=$(psql "$DATABASE_URL" -t -c \
  "SELECT COUNT(*) FROM task_execution_states WHERE task_id = '$TASK_ID'")

if [ "$STATE_EXISTS" -eq "0" ]; then
  echo "ERROR: State not found in database"
  exit 1
fi

echo "State found in database"

# 4. Check resumable status
echo "4. Checking resumable status..."
RESUMABLE=$(psql "$DATABASE_URL" -t -c \
  "SELECT COUNT(*) FROM task_execution_states 
   WHERE task_id = '$TASK_ID' 
   AND status IN ('paused', 'crashed', 'running')")

if [ "$RESUMABLE" -eq "0" ]; then
  echo "WARNING: Task is not resumable"
else
  echo "Task is resumable"
fi

echo "=== Test Complete ==="
```

---

## Test Data Cleanup

### Manual Cleanup

```sql
-- Delete test states
DELETE FROM task_execution_states WHERE task_id IN (
  SELECT id FROM tasks WHERE title LIKE 'Test%'
);

-- Delete test checkpoints
DELETE FROM task_state_checkpoints WHERE task_id IN (
  SELECT id FROM tasks WHERE title LIKE 'Test%'
);

-- Delete test tasks
DELETE FROM tasks WHERE title LIKE 'Test%';
```

### Automated Cleanup

Tests automatically clean up after themselves, but for manual testing:

```bash
# Clean up test data
psql $DATABASE_URL -c "
  DELETE FROM task_state_checkpoints WHERE task_id IN (
    SELECT id FROM tasks WHERE title LIKE 'Test%'
  );
  DELETE FROM task_execution_states WHERE task_id IN (
    SELECT id FROM tasks WHERE title LIKE 'Test%'
  );
  DELETE FROM tasks WHERE title LIKE 'Test%';
"
```

---

## Troubleshooting

### Common Issues

#### 1. Database Connection Errors

**Error:** `Failed to create test database client`

**Solution:**
- Verify `DATABASE_URL` is set correctly
- Check PostgreSQL is running: `pg_isready`
- Verify database exists: `psql -l | grep agent_agency_test`
- Check connection permissions

#### 2. Migration Errors

**Error:** `relation "task_execution_states" does not exist`

**Solution:**
- Run migration: `psql $DATABASE_URL -f migrations/020_create_task_state_persistence.sql`
- Verify tables exist: `\dt` in psql
- Check migration was applied: `SELECT * FROM schema_migrations;`

#### 3. Foreign Key Violations

**Error:** `foreign key constraint "task_execution_states_task_id_fkey"`

**Solution:**
- Ensure test tasks are created before state
- Use `create_test_task()` helper function
- Check `tasks` table has corresponding record

#### 4. Serialization Errors

**Error:** `Failed to serialize TaskExecutionState to JSON`

**Solution:**
- Verify all fields in `TaskExecutionState` are `Serialize`
- Check for circular references
- Ensure `serde_json` is available

---

## Performance Testing

### Load Test

Test persistence performance under load:

```rust
#[tokio::test]
#[ignore]
async fn test_persistence_performance() {
    let db_client = Arc::new(create_test_db_client().await);
    let persistence = Arc::new(DatabaseTaskStatePersistence::new(db_client.clone()));
    
    let start = std::time::Instant::now();
    let task_ids: Vec<Uuid> = (0..100).map(|_| Uuid::new_v4()).collect();
    
    // Create tasks
    for task_id in &task_ids {
        create_test_task(&db_client, *task_id).await.unwrap();
    }
    
    // Save states concurrently
    let mut handles = vec![];
    for task_id in &task_ids {
        let persistence = persistence.clone();
        let state = create_test_state(*task_id, ExecutionStateStatus::Running);
        handles.push(tokio::spawn(async move {
            persistence.save_state(&state).await
        }));
    }
    
    futures::future::join_all(handles).await;
    
    let duration = start.elapsed();
    println!("Saved 100 states in {:?}", duration);
    
    // Cleanup
    for task_id in &task_ids {
        persistence.delete_state(*task_id).await.unwrap();
    }
}
```

---

## Test Results

### Expected Results

- ✅ All unit tests pass
- ✅ All integration tests pass (with database)
- ✅ State persists across server restarts
- ✅ Tasks can be resumed after interruption
- ✅ Checkpoints can be created and listed
- ✅ State deletion works correctly

### Performance Benchmarks

- **Save State:** < 50ms per operation
- **Load State:** < 30ms per operation
- **List Resumable:** < 100ms for 1000 tasks
- **Create Checkpoint:** < 60ms per operation

---

## Continuous Integration

### CI Test Configuration

Add to CI pipeline:

```yaml
# .github/workflows/test.yml
- name: Test Task State Persistence
  env:
    DATABASE_URL: postgresql://postgres:postgres@localhost:5432/agent_agency_test
  run: |
    cd iterations/v3/data-infrastructure
    psql $DATABASE_URL -f migrations/020_create_task_state_persistence.sql
    cd ../agent-orchestration
    cargo test --test integration_task_state_persistence -- --ignored
```

---

## Next Steps

1. ✅ Integration tests created
2. ⚠️ Run tests with database
3. ⚠️ Verify end-to-end workflow
4. ⚠️ Add performance benchmarks
5. ⚠️ Document API endpoints for resumption

---

**Status:** Tests created and ready to run  
**Next:** Execute tests with database and verify end-to-end workflow

