-- ============================================================================
-- TASK AUDIT LOGS MIGRATION
-- ============================================================================
-- Create dedicated audit logs table for task execution tracking
-- This addresses P0 requirement: "Persist audit trail + surface it on tasks"
--
-- Schema based on missing.md specification:
-- - task_id: Links events to specific tasks
-- - category: Groups events (orchestration, worker, artifact, alert)
-- - actor: Who performed the action (system, user:<id>, worker:<id>)
-- - action: What happened (enqueued, started, step, canceled, error, completed)
-- - payload: Structured data about the event
-- ============================================================================

-- Create the task-specific audit logs table
CREATE TABLE IF NOT EXISTS task_audit_logs (
  id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  ts              TIMESTAMPTZ NOT NULL DEFAULT now(),
  task_id         UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
  category        TEXT NOT NULL CHECK (category IN ('orchestration', 'worker', 'artifact', 'alert', 'system')),
  actor           TEXT NOT NULL,  -- "system", "user:<id>", "worker:<id>", "council:<id>"
  action          TEXT NOT NULL,  -- e.g., enqueued, started, step, canceled, error, completed
  payload         JSONB NOT NULL DEFAULT '{}'::jsonb,
  idx             BIGINT GENERATED ALWAYS AS IDENTITY
);

-- Create indexes for efficient querying
CREATE INDEX idx_task_audit_logs_task_id_ts ON task_audit_logs (task_id, ts);
CREATE INDEX idx_task_audit_logs_category_ts ON task_audit_logs (category, ts DESC);
CREATE INDEX idx_task_audit_logs_actor ON task_audit_logs (actor);
CREATE INDEX idx_task_audit_logs_action ON task_audit_logs (action);

-- Add table comment
COMMENT ON TABLE task_audit_logs IS 'Task execution audit trail - immutable log of all task state changes and operations';

-- Add column comments for clarity
COMMENT ON COLUMN task_audit_logs.category IS 'Event category: orchestration (task lifecycle), worker (execution), artifact (file ops), alert (monitoring), system (internal)';
COMMENT ON COLUMN task_audit_logs.actor IS 'Entity that performed the action: system, user:<uuid>, worker:<uuid>, council:<uuid>';
COMMENT ON COLUMN task_audit_logs.action IS 'Specific action taken: enqueued, started, step, canceled, error, completed, paused, resumed';
COMMENT ON COLUMN task_audit_logs.payload IS 'Structured event data with context, parameters, and results';
COMMENT ON COLUMN task_audit_logs.idx IS 'Sequential index for pagination and ordering guarantees';

-- ============================================================================
-- PERFORMANCE & MONITORING
-- ============================================================================

-- Create a view for task event summaries (useful for dashboards)
CREATE OR REPLACE VIEW task_event_summaries AS
SELECT
    task_id,
    category,
    action,
    COUNT(*) as event_count,
    MIN(ts) as first_event_at,
    MAX(ts) as last_event_at,
    EXTRACT(EPOCH FROM (MAX(ts) - MIN(ts))) as duration_seconds
FROM task_audit_logs
GROUP BY task_id, category, action;

-- Create a function to get recent task events with pagination
CREATE OR REPLACE FUNCTION get_task_events_paginated(
    p_task_id UUID,
    p_since TIMESTAMPTZ DEFAULT NULL,
    p_limit INTEGER DEFAULT 50,
    p_offset BIGINT DEFAULT 0
)
RETURNS TABLE (
    id UUID,
    ts TIMESTAMPTZ,
    category TEXT,
    actor TEXT,
    action TEXT,
    payload JSONB,
    idx BIGINT
)
LANGUAGE SQL
STABLE
AS $$
    SELECT
        tal.id,
        tal.ts,
        tal.category,
        tal.actor,
        tal.action,
        tal.payload,
        tal.idx
    FROM task_audit_logs tal
    WHERE tal.task_id = p_task_id
      AND (p_since IS NULL OR tal.ts >= p_since)
    ORDER BY tal.idx DESC
    LIMIT p_limit
    OFFSET p_offset;
$$;

-- Add function comment
COMMENT ON FUNCTION get_task_events_paginated(UUID, TIMESTAMPTZ, INTEGER, BIGINT) IS 'Get paginated task events with efficient ordering by sequence index';

-- ============================================================================
-- DATA INTEGRITY & CLEANUP
-- ============================================================================

-- Create a constraint to ensure payload contains required fields for certain actions
ALTER TABLE task_audit_logs
ADD CONSTRAINT check_payload_structure
CHECK (
    -- For error actions, payload must contain error details
    (action = 'error' AND payload ? 'error_type' AND payload ? 'error_message') OR
    -- For step actions, payload must contain step info
    (action = 'step' AND payload ? 'step_name') OR
    -- For other actions, payload can be empty or minimal
    (action NOT IN ('error', 'step'))
);

-- Create a trigger to prevent updates to audit logs (immutable)
CREATE OR REPLACE FUNCTION prevent_audit_log_updates()
RETURNS TRIGGER AS $$
BEGIN
    -- Allow INSERTs but prevent UPDATEs and DELETEs
    IF TG_OP = 'UPDATE' THEN
        RAISE EXCEPTION 'Audit logs are immutable - cannot update existing entries';
    END IF;
    IF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'Audit logs are immutable - cannot delete existing entries';
    END IF;
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER task_audit_logs_immutable
    BEFORE UPDATE OR DELETE ON task_audit_logs
    FOR EACH ROW EXECUTE FUNCTION prevent_audit_log_updates();

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

-- Note: This migration creates the foundation for task audit trails.
-- Future migrations may add additional indexes or partitioning strategies
-- based on production usage patterns.
