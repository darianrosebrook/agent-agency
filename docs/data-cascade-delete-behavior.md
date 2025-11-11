# Data Cascade Delete Behavior

**Author**: @darianrosebrook  
**Last Updated**: 2025-01-28

## Overview

This document describes the cascade delete behavior for projects, chats, and tasks in the Agent Agency system. Understanding these relationships is critical for data integrity and cleanup operations.

## Cascade Delete Summary

### Projects (execution_plans)

**Status**: ⚠️ **Partial Cascade**

When a project is deleted:

- ✅ **Milestones**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `milestones.plan_id` → `execution_plans.id`
  - Migration: `005_create_planning_system.sql` line 28

- ❌ **Tasks**: **NOT automatically deleted** (orphaned)
  - Tasks are linked to projects via `metadata.project_id` JSONB field
  - No foreign key constraint exists
  - Tasks remain in database with orphaned `metadata.project_id` reference

**Current Implementation**:
```rust
// From database_operations_adapter.rs:785
// Comment says "cascade will delete related records" 
// but this only applies to milestones, not tasks
DELETE FROM execution_plans WHERE id = $1
```

**Impact**: Deleting a project leaves orphaned tasks in the database.

**Recommendation**: Either:
1. Add manual task cleanup in `delete_execution_plan()` handler
2. Add foreign key constraint (requires schema migration)
3. Document this behavior and add cleanup job

---

### Chats (chat_sessions)

**Status**: ✅ **Full Cascade** (Like deleting a folder)

When a chat session is deleted:

- ✅ **Chat Messages**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `chat_messages.session_id` → `chat_sessions.id`
  - Migration: `010_create_chat_persistence.sql` line 37

- ✅ **Message Attachments**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `chat_message_attachments.message_id` → `chat_messages.id`
  - Migration: `010_create_chat_persistence.sql` line 68

- ✅ **Session Tags**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `chat_session_tags.session_id` → `chat_sessions.id`
  - Migration: `013_add_chat_search_and_organization.sql` line 117

**Behavior**: Deleting a chat session is like deleting a folder - all related data is automatically cleaned up.

---

### Tasks

**Status**: ✅ **Full Cascade** (for related data)

When a task is deleted:

- ✅ **Task Execution States**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `task_execution_states.task_id` → `tasks.id`
  - Migration: `020_create_task_state_persistence.sql` line 11

- ✅ **Task State Checkpoints**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `task_state_checkpoints.task_id` → `tasks.id`
  - Migration: `020_create_task_state_persistence.sql` line 35

- ✅ **Task Executions**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `task_executions.task_id` → `tasks.id`
  - Migration: `014_create_agent_management_tables.sql` line 95

- ✅ **Debate Sessions**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `debate_sessions.task_id` → `tasks.id`
  - Migration: `014_create_agent_management_tables.sql` line 200

- ✅ **Provenance Entries**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `provenance_entries.task_id` → `tasks.id`
  - Migration: `015_create_observation_tables.sql` line 65

- ✅ **Judge Evaluations**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `judge_evaluations.verdict_id` → `council_verdicts.verdict_id`
  - Migration: `014_create_agent_management_tables.sql` line 186

- ✅ **CAWS Rule Violations**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `caws_rule_violations.task_id` → `tasks.id`
  - Migration: `019_create_rules_governance_tables.sql` line 37

**Behavior**: Deleting a task automatically cleans up all related execution data, state, checkpoints, and provenance.

**Note**: Tasks are NOT automatically deleted when their parent project is deleted (see Projects section above).

---

## Database Schema Relationships

### Projects → Milestones
```sql
-- Migration 005, line 28
plan_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE
```
✅ **Cascade Delete**: Yes

### Projects → Tasks
```sql
-- No foreign key constraint exists
-- Tasks linked via metadata JSONB field:
-- metadata: { "project_id": "..." }
```
❌ **Cascade Delete**: No (orphaned on project deletion)

### Chats → Messages
```sql
-- Migration 010, line 37
session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE
```
✅ **Cascade Delete**: Yes

### Tasks → Related Data
Multiple tables with `ON DELETE CASCADE`:
- `task_execution_states.task_id`
- `task_state_checkpoints.task_id`
- `task_executions.task_id`
- `debate_sessions.task_id`
- `provenance_entries.task_id`
- `caws_rule_violations.task_id`

✅ **Cascade Delete**: Yes (for all related data)

---

## Current Implementation Issues

### Issue 1: Orphaned Tasks

**Problem**: When a project is deleted, tasks remain in the database with orphaned `metadata.project_id` references.

**Current Code**:
```rust
// database_operations_adapter.rs:785
async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
    // Comment says "cascade will delete related records"
    // but this only applies to milestones, not tasks
    sqlx::query("DELETE FROM execution_plans WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await
}
```

**Impact**:
- Tasks remain in database after project deletion
- `get_project_tasks()` will return empty array (filtered by metadata)
- Tasks become orphaned and inaccessible via project API
- Potential data accumulation over time

**Recommendations**:

1. **Option A: Manual Cleanup** (Quick fix)
   ```rust
   async fn delete_execution_plan(&self, id: Uuid) -> Result<()> {
       // Delete tasks linked via metadata
       sqlx::query(
           r#"
           DELETE FROM tasks 
           WHERE metadata->>'project_id' = $1
           "#
       )
       .bind(id.to_string())
       .execute(pool)
       .await?;
       
       // Then delete project (cascades to milestones)
       sqlx::query("DELETE FROM execution_plans WHERE id = $1")
           .bind(id)
           .execute(pool)
           .await?;
   }
   ```

2. **Option B: Foreign Key Constraint** (Schema change)
   ```sql
   -- Add project_id column to tasks table
   ALTER TABLE tasks ADD COLUMN project_id UUID REFERENCES execution_plans(id) ON DELETE CASCADE;
   
   -- Migrate existing data from metadata
   UPDATE tasks SET project_id = (metadata->>'project_id')::UUID 
   WHERE metadata->>'project_id' IS NOT NULL;
   
   -- Add index
   CREATE INDEX idx_tasks_project_id ON tasks(project_id);
   ```

3. **Option C: Cleanup Job** (Background process)
   - Periodic job to find and delete orphaned tasks
   - Less ideal but doesn't require schema changes

---

## Recommendations

### Immediate Actions

1. ✅ **Document the behavior** (this document)
2. ⚠️ **Add warning in UI**: When deleting a project, warn that tasks will remain orphaned
3. 🔧 **Consider manual cleanup**: Update `delete_execution_plan()` to delete tasks manually

### Long-term Improvements

1. **Schema Migration**: Add `project_id` foreign key to `tasks` table
2. **Data Migration**: Migrate existing `metadata.project_id` to `project_id` column
3. **Validation**: Ensure all new tasks use foreign key, not metadata
4. **Testing**: Add integration tests for cascade delete behavior

---

## Testing Cascade Behavior

To verify cascade delete behavior:

```sql
-- Test project deletion
BEGIN;
INSERT INTO execution_plans (id, title, ...) VALUES (...);
INSERT INTO milestones (plan_id, ...) VALUES (...);
INSERT INTO tasks (id, metadata) VALUES (..., '{"project_id": "..."}');
DELETE FROM execution_plans WHERE id = ...;
-- Check: milestones deleted? tasks deleted?
ROLLBACK;

-- Test chat deletion
BEGIN;
INSERT INTO chat_sessions (id, ...) VALUES (...);
INSERT INTO chat_messages (session_id, ...) VALUES (...);
DELETE FROM chat_sessions WHERE id = ...;
-- Check: messages deleted?
ROLLBACK;

-- Test task deletion
BEGIN;
INSERT INTO tasks (id, ...) VALUES (...);
INSERT INTO task_execution_states (task_id, ...) VALUES (...);
INSERT INTO task_state_checkpoints (task_id, ...) VALUES (...);
DELETE FROM tasks WHERE id = ...;
-- Check: execution states deleted? checkpoints deleted?
ROLLBACK;
```

---

## Related Files

- Database Migrations:
  - `005_create_planning_system.sql` - Projects and milestones
  - `010_create_chat_persistence.sql` - Chat sessions and messages
  - `014_create_agent_management_tables.sql` - Tasks and related tables
  - `015_create_observation_tables.sql` - Provenance entries
  - `019_create_rules_governance_tables.sql` - CAWS rule violations
  - `020_create_task_state_persistence.sql` - Task execution states

- Implementation:
  - `iterations/v3/data-interfaces-adapters/src/database_operations_adapter.rs:785` - `delete_execution_plan()`
  - `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs:2977` - `delete_project_handler()`

