-- Migration 024: Add project_id foreign key to tasks table
-- Adds project_id column with foreign key constraint to enable cascade delete
-- Migrates existing data from metadata.project_id to the new column
-- @author @darianrosebrook

BEGIN;

-- ============================================================================
-- ADD project_id COLUMN TO tasks TABLE
-- ============================================================================

-- Add project_id column (nullable initially to allow migration)
ALTER TABLE tasks 
ADD COLUMN IF NOT EXISTS project_id UUID;

-- ============================================================================
-- MIGRATE EXISTING DATA FROM metadata.project_id
-- ============================================================================

-- Migrate existing project_id values from metadata JSONB to project_id column
UPDATE tasks
SET project_id = (metadata->>'project_id')::UUID
WHERE metadata IS NOT NULL 
  AND metadata->>'project_id' IS NOT NULL
  AND (metadata->>'project_id')::UUID IS NOT NULL
  AND project_id IS NULL;

-- ============================================================================
-- ADD FOREIGN KEY CONSTRAINT WITH CASCADE DELETE
-- ============================================================================

-- Add foreign key constraint with CASCADE DELETE
-- This ensures tasks are automatically deleted when their project is deleted
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_tasks_project_id'
    ) THEN
        ALTER TABLE tasks
        ADD CONSTRAINT fk_tasks_project_id
        FOREIGN KEY (project_id) REFERENCES execution_plans(id) ON DELETE CASCADE;
    END IF;
END $$;

-- ============================================================================
-- ADD INDEX FOR PERFORMANCE
-- ============================================================================

-- Add index on project_id for efficient project task queries
CREATE INDEX IF NOT EXISTS idx_tasks_project_id ON tasks(project_id)
WHERE project_id IS NOT NULL;

-- ============================================================================
-- UPDATE COMMENTS
-- ============================================================================

COMMENT ON COLUMN tasks.project_id IS 'Foreign key to execution_plans. Tasks are automatically deleted when project is deleted via CASCADE.';
COMMENT ON INDEX idx_tasks_project_id IS 'Index for efficient project task queries and cascade delete operations';

COMMIT;









