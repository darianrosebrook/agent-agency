-- Migration 020: Create Task State Persistence Tables
-- Creates database tables for task execution state persistence and checkpoint management
-- Enables task resumption, crash recovery, and checkpoint/restore functionality
-- @author @darianrosebrook

-- ============================================================================
-- TASK EXECUTION STATES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS task_execution_states (
    task_id UUID PRIMARY KEY REFERENCES tasks(id) ON DELETE CASCADE,
    state_data JSONB NOT NULL,
    status VARCHAR(50) NOT NULL CHECK (status IN ('pending', 'running', 'paused', 'completed', 'failed', 'cancelled', 'crashed')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    checkpoint_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for task_execution_states
CREATE INDEX IF NOT EXISTS idx_task_execution_states_status ON task_execution_states(status);
CREATE INDEX IF NOT EXISTS idx_task_execution_states_last_updated ON task_execution_states(last_updated DESC);
CREATE INDEX IF NOT EXISTS idx_task_execution_states_checkpoint_at ON task_execution_states(checkpoint_at DESC);
CREATE INDEX IF NOT EXISTS idx_task_execution_states_resumable ON task_execution_states(status) 
    WHERE status IN ('paused', 'crashed', 'running');

-- GIN index for JSONB queries
CREATE INDEX IF NOT EXISTS idx_task_execution_states_state_data ON task_execution_states USING GIN(state_data);

-- ============================================================================
-- TASK STATE CHECKPOINTS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS task_state_checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    checkpoint_timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    state_data JSONB NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for task_state_checkpoints
CREATE INDEX IF NOT EXISTS idx_task_state_checkpoints_task_id ON task_state_checkpoints(task_id);
CREATE INDEX IF NOT EXISTS idx_task_state_checkpoints_timestamp ON task_state_checkpoints(task_id, checkpoint_timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_task_state_checkpoints_created_at ON task_state_checkpoints(created_at DESC);

-- ============================================================================
-- TRIGGERS FOR UPDATED_AT TIMESTAMPS
-- ============================================================================

-- Trigger to update last_updated timestamp
CREATE OR REPLACE FUNCTION update_task_execution_state_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.last_updated = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'task_execution_states_updated_at') THEN
        CREATE TRIGGER task_execution_states_updated_at
            BEFORE UPDATE ON task_execution_states
            FOR EACH ROW
            EXECUTE FUNCTION update_task_execution_state_updated_at();
    END IF;
END
$$;

