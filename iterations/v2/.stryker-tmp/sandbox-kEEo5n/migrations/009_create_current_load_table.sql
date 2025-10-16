/**
 * Migration: Create Current Load Table
 * 
 * @author @darianrosebrook
 * @migration 009
 * @component ARBITER-001 (Agent Registry Manager)
 * 
 * Creates the current_load table that was missing from the original migration.
 * This table tracks real-time load information for agents.
 */

-- ============================================================================
-- Current Load Table
-- ============================================================================
-- Tracks real-time load information for agents

CREATE TABLE IF NOT EXISTS current_load (
    agent_id UUID PRIMARY KEY REFERENCES agent_profiles(id) ON DELETE CASCADE,
    active_tasks INTEGER NOT NULL DEFAULT 0 CHECK (active_tasks >= 0),
    queued_tasks INTEGER NOT NULL DEFAULT 0 CHECK (queued_tasks >= 0),
    utilization_percent NUMERIC(5,2) NOT NULL DEFAULT 0.00 CHECK (utilization_percent >= 0 AND utilization_percent <= 100),
    
    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for load-based queries
CREATE INDEX IF NOT EXISTS idx_current_load_utilization ON current_load (utilization_percent ASC);

-- Index for active task queries
CREATE INDEX IF NOT EXISTS idx_current_load_active_tasks ON current_load (active_tasks ASC);

-- ============================================================================
-- Updated Timestamp Trigger
-- ============================================================================

CREATE TRIGGER update_current_load_updated_at
    BEFORE UPDATE ON current_load
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON TABLE current_load IS 'Real-time load tracking for agents (active tasks, queued tasks, utilization)';

COMMENT ON COLUMN current_load.active_tasks IS 'Number of currently active tasks for this agent';

COMMENT ON COLUMN current_load.queued_tasks IS 'Number of tasks queued for this agent';

COMMENT ON COLUMN current_load.utilization_percent IS 'Current utilization percentage (0-100)';

-- ============================================================================
-- Migration Complete
-- ============================================================================
