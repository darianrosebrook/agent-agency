-- Migration 000: Create Workspace Registry Table
-- Creates the workspace_registry table required by UnifiedOrchestrator MemorySystem
-- This table must be created before other migrations that depend on workspace functionality
-- @author @darianrosebrook

-- ============================================================================
-- WORKSPACE REGISTRY TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS workspace_registry (
    workspace_id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    path TEXT NOT NULL,
    access VARCHAR(50) NOT NULL CHECK (access IN ('enabled', 'disabled', 'readonly', 'blocked')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_accessed TIMESTAMP WITH TIME ZONE,
    access_count BIGINT DEFAULT 0,
    discovered_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    is_default BOOLEAN DEFAULT FALSE
);

-- Indexes for workspace_registry
CREATE INDEX IF NOT EXISTS idx_workspace_registry_name ON workspace_registry(name);
CREATE INDEX IF NOT EXISTS idx_workspace_registry_path ON workspace_registry(path);
CREATE INDEX IF NOT EXISTS idx_workspace_registry_access ON workspace_registry(access);
CREATE INDEX IF NOT EXISTS idx_workspace_registry_last_accessed ON workspace_registry(last_accessed DESC);
CREATE INDEX IF NOT EXISTS idx_workspace_registry_is_default ON workspace_registry(is_default);
CREATE INDEX IF NOT EXISTS idx_workspace_registry_discovered_at ON workspace_registry(discovered_at);

-- Comments
COMMENT ON TABLE workspace_registry IS 'Registry of known workspaces for cross-workspace memory access and discovery';
COMMENT ON COLUMN workspace_registry.workspace_id IS 'Unique identifier for the workspace';
COMMENT ON COLUMN workspace_registry.name IS 'Human-readable name of the workspace';
COMMENT ON COLUMN workspace_registry.path IS 'File system path to the workspace';
COMMENT ON COLUMN workspace_registry.access IS 'Access level: enabled, disabled, readonly, or blocked';
COMMENT ON COLUMN workspace_registry.access_count IS 'Number of times this workspace has been accessed';
COMMENT ON COLUMN workspace_registry.is_default IS 'Whether this is the default workspace';

