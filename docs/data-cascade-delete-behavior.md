# Data Cascade Delete Behavior

**Author**: @darianrosebrook  
**Last Updated**: 2025-01-28

## Overview

This document describes the cascade delete behavior for projects, chats, and tasks in the Agent Agency system. Understanding these relationships is critical for data integrity and cleanup operations.

## Cascade Delete Summary

### Projects (execution_plans)

**Status**: ✅ **Full Cascade** (Like deleting a folder)

When a project is deleted:

- ✅ **Milestones**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `milestones.plan_id` → `execution_plans.id`
  - Migration: `005_create_planning_system.sql` line 28

- ✅ **Tasks**: Automatically deleted via `ON DELETE CASCADE`
  - Foreign key: `tasks.project_id` → `execution_plans.id`
  - Migration: `024_add_task_project_id_foreign_key.sql` line 42
  - Tasks are now linked via proper foreign key constraint

**Current Implementation**:
```rust
// From database_operations_adapter.rs:785
// Cascade delete now works for both milestones and tasks
DELETE FROM execution_plans WHERE id = $1
// PostgreSQL automatically deletes:
// - All milestones via CASCADE
// - All tasks via CASCADE
```

**Migration**: Migration `024_add_task_project_id_foreign_key.sql`:
- Adds `project_id` column to `tasks` table
- Migrates existing `metadata.project_id` values to the new column
- Adds foreign key constraint with `ON DELETE CASCADE`
- Adds index for performance

**Impact**: Deleting a project now automatically cleans up all related tasks, just like deleting a folder.

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
-- Migration 024, line 42
project_id UUID REFERENCES execution_plans(id) ON DELETE CASCADE
```
✅ **Cascade Delete**: Yes (automatic cleanup on project deletion)

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

## Implementation Status

### ✅ Issue Resolved: Foreign Key Constraint Added

**Solution Implemented**: Foreign key constraint with CASCADE DELETE (Migration 024)

**Migration Details**:
- Added `project_id` column to `tasks` table
- Migrated existing `metadata.project_id` values to the new column
- Added foreign key constraint: `tasks.project_id` → `execution_plans.id` with `ON DELETE CASCADE`
- Added index for performance: `idx_tasks_project_id`

**Code Changes**:
- Updated `Task` model to include `project_id: Option<Uuid>`
- Updated `CreateTask` and `UpdateTask` to include `project_id`
- Updated database queries to use `project_id` column instead of `metadata.project_id`
- Updated API handlers to use `project_id` for verification and filtering

**Result**: Deleting a project now automatically deletes all associated tasks via database cascade, ensuring data integrity.

---

## Recommendations

### ✅ Completed Actions

1. ✅ **Documented the behavior** (this document)
2. ✅ **Added foreign key constraint**: Migration 024 implements proper cascade delete
3. ✅ **Migrated existing data**: All `metadata.project_id` values migrated to `project_id` column
4. ✅ **Updated code**: All queries and handlers now use `project_id` column

### Future Improvements

1. **Remove metadata.project_id**: Consider removing `project_id` from metadata JSONB after migration period
2. **Validation**: Ensure all new tasks use `project_id` field, not metadata
3. **Testing**: Add integration tests for cascade delete behavior
4. **Monitoring**: Track orphaned tasks (should be zero after migration)

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
  - `024_add_task_project_id_foreign_key.sql` - **Task project foreign key with cascade delete**

- Implementation:
  - `iterations/v3/data-interfaces-adapters/src/database_operations_adapter.rs:785` - `delete_execution_plan()`
  - `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs:2977` - `delete_project_handler()`
  - `iterations/v3/data-infrastructure/src/models.rs` - `Task`, `CreateTask`, `UpdateTask` models
  - `iterations/v3/data-infrastructure/src/client/orchestrator.rs` - Database operations
  - `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs` - API handlers updated to use `project_id`

