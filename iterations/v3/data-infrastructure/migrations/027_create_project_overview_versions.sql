-- Migration 027: Create Project Overview Versions Table
-- Creates database table for tracking project overview version history
-- @author @darianrosebrook

-- ============================================================================
-- PROJECT OVERVIEW VERSIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS project_overview_versions (
    version_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE,
    overview TEXT NOT NULL,
    change_summary TEXT,
    created_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE INDEX IF NOT EXISTS idx_project_overview_versions_project_id 
    ON project_overview_versions(project_id);
CREATE INDEX IF NOT EXISTS idx_project_overview_versions_created_at 
    ON project_overview_versions(created_at DESC);

-- Composite index for efficient version listing queries
CREATE INDEX IF NOT EXISTS idx_project_overview_versions_project_created 
    ON project_overview_versions(project_id, created_at DESC);

