-- Migration 029: Create Task Progress Tracking Table
-- Creates a dedicated table for task progress tracking
-- Enables progress persistence and retrieval for the ProgressTrackingServiceAdapter
-- @author @darianrosebrook

-- ============================================================================
-- TASK PROGRESS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS task_progress (
    task_id UUID PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    progress_percent SMALLINT NOT NULL DEFAULT 0 CHECK (progress_percent >= 0 AND progress_percent <= 100),
    current_stage VARCHAR(255) NOT NULL DEFAULT 'Unknown',
    status_message TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for task_progress
CREATE INDEX IF NOT EXISTS idx_task_progress_updated_at ON task_progress(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_task_progress_percent ON task_progress(progress_percent);
CREATE INDEX IF NOT EXISTS idx_task_progress_current_stage ON task_progress(current_stage);

-- GIN index for JSONB queries
CREATE INDEX IF NOT EXISTS idx_task_progress_metadata ON task_progress USING GIN(metadata);

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT TIMESTAMPS
-- ============================================================================

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_task_progress_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'task_progress_updated_at') THEN
        CREATE TRIGGER task_progress_updated_at
            BEFORE UPDATE ON task_progress
            FOR EACH ROW
            EXECUTE FUNCTION update_task_progress_updated_at();
    END IF;
END
$$;








