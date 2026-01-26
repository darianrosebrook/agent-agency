-- Migration 032: Link execution plans to workspace registry
-- Adds a nullable workspace_id foreign key on execution_plans pointing to workspace_registry.
-- Author: @darianrosebrook

ALTER TABLE execution_plans
    ADD COLUMN IF NOT EXISTS workspace_id VARCHAR(255)
        REFERENCES workspace_registry(workspace_id)
        ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_execution_plans_workspace_id
    ON execution_plans(workspace_id);

COMMENT ON COLUMN execution_plans.workspace_id IS 'Optional workspace registry ID for the project/work plan';



