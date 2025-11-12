-- Migration 026: Create Council Sessions Table
-- Creates database table for council session tracking and lifecycle management
-- Author: @darianrosebrook

BEGIN;

-- ===========================================
-- COUNCIL SESSIONS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS council_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL UNIQUE,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    working_spec_id VARCHAR(255),
    review_context JSONB NOT NULL DEFAULT '{}'::jsonb,
    status VARCHAR(50) NOT NULL DEFAULT 'initialized' CHECK (status IN ('initialized', 'judge_selection', 'review_in_progress', 'aggregation_in_progress', 'decision_making', 'completed', 'failed', 'timeout')),
    selected_judges JSONB NOT NULL DEFAULT '[]'::jsonb,
    contributions JSONB NOT NULL DEFAULT '[]'::jsonb,
    aggregation_result JSONB,
    final_decision JSONB,
    progress FLOAT NOT NULL DEFAULT 0.0 CHECK (progress >= 0.0 AND progress <= 1.0),
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for council_sessions
CREATE INDEX IF NOT EXISTS idx_council_sessions_session_id ON council_sessions(session_id);
CREATE INDEX IF NOT EXISTS idx_council_sessions_task_id ON council_sessions(task_id);
CREATE INDEX IF NOT EXISTS idx_council_sessions_status ON council_sessions(status);
CREATE INDEX IF NOT EXISTS idx_council_sessions_started_at ON council_sessions(started_at DESC);
CREATE INDEX IF NOT EXISTS idx_council_sessions_created_at ON council_sessions(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_council_sessions_working_spec_id ON council_sessions(working_spec_id);

-- Composite index for efficient session queries
CREATE INDEX IF NOT EXISTS idx_council_sessions_task_status ON council_sessions(task_id, status) WHERE task_id IS NOT NULL;

-- ===========================================
-- TRIGGERS
-- ====================================

-- Update updated_at timestamp on session update
CREATE OR REPLACE FUNCTION update_council_session_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_council_session_updated_at
    BEFORE UPDATE ON council_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_council_session_updated_at();

-- ===========================================
-- COMMENTS
-- ===========================================

COMMENT ON TABLE council_sessions IS 'Council review sessions for tracking task evaluation lifecycle';
COMMENT ON COLUMN council_sessions.session_id IS 'Unique session identifier (UUID)';
COMMENT ON COLUMN council_sessions.task_id IS 'Optional foreign key to tasks table';
COMMENT ON COLUMN council_sessions.status IS 'Current session status in lifecycle';
COMMENT ON COLUMN council_sessions.progress IS 'Session progress (0.0-1.0)';
COMMENT ON COLUMN council_sessions.review_context IS 'Review context JSON for council evaluation';
COMMENT ON COLUMN council_sessions.final_decision IS 'Final council decision JSON when completed';

-- Log migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('026', 'create_council_sessions', NOW())
ON CONFLICT (version) DO NOTHING;

COMMIT;

