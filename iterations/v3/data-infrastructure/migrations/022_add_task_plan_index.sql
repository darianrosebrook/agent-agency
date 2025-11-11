-- Migration 022: Add index on execution_plans.working_spec_id for efficient task-to-plan lookups
-- This enables fast lookups of execution plans by task_id when working_spec_id follows TASK-<UUID> format

-- Add index on working_spec_id for efficient task lookups
CREATE INDEX IF NOT EXISTS idx_execution_plans_working_spec_id ON execution_plans(working_spec_id);

-- Add comment explaining the index purpose
COMMENT ON INDEX idx_execution_plans_working_spec_id IS 'Index for efficient task-to-plan lookups when working_spec_id follows TASK-<UUID> format';

