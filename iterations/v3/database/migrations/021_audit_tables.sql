-- ============================================================================
-- AUDIT TABLES MIGRATION
-- ============================================================================
-- Create audit trail tables for P0 compliance and task event logging
-- This addresses P0 requirement: "Audit trail DB + task event logging"
--
-- Schema includes:
-- - task_audit_logs: Comprehensive audit trail for all task operations
-- - audit_events: Generic audit events for system operations
-- - Proper indexing for efficient audit queries and compliance reporting
-- ============================================================================

-- Create task_audit_logs table for comprehensive task event logging
CREATE TABLE IF NOT EXISTS task_audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ts TIMESTAMPTZ NOT NULL DEFAULT now(),
    task_id VARCHAR(255) NOT NULL,
    user_id VARCHAR(255), -- User who performed the action
    action VARCHAR(50) NOT NULL, -- created, updated, paused, resumed, cancelled, completed, failed
    old_state VARCHAR(50), -- Previous task state
    new_state VARCHAR(50), -- New task state
    details JSONB DEFAULT '{}'::jsonb, -- Additional context (parameters, reasons, etc.)
    ip_address INET, -- Client IP for security auditing
    user_agent TEXT, -- Browser/client information
    session_id VARCHAR(255), -- Session tracking
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create audit_events table for general system audit events
CREATE TABLE IF NOT EXISTS audit_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    ts TIMESTAMPTZ NOT NULL DEFAULT now(),
    event_type VARCHAR(100) NOT NULL, -- api_call, auth_attempt, config_change, etc.
    severity VARCHAR(20) DEFAULT 'info' CHECK (severity IN ('debug', 'info', 'warning', 'error', 'critical')),
    source VARCHAR(100), -- api-server, worker, orchestrator, etc.
    user_id VARCHAR(255), -- Associated user if applicable
    session_id VARCHAR(255), -- Session context
    resource_type VARCHAR(50), -- task, user, system, etc.
    resource_id VARCHAR(255), -- ID of affected resource
    action VARCHAR(50), -- What happened (create, update, delete, etc.)
    details JSONB DEFAULT '{}'::jsonb, -- Event-specific data
    ip_address INET, -- Client IP
    user_agent TEXT, -- Client information
    success BOOLEAN DEFAULT true, -- Whether the operation succeeded
    error_message TEXT, -- Error details if applicable
    processing_time_ms INTEGER, -- How long the operation took
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Create indexes for efficient querying
CREATE INDEX idx_task_audit_logs_task_id ON task_audit_logs (task_id);
CREATE INDEX idx_task_audit_logs_ts ON task_audit_logs (ts DESC);
CREATE INDEX idx_task_audit_logs_user_id ON task_audit_logs (user_id);
CREATE INDEX idx_task_audit_logs_action ON task_audit_logs (action);
CREATE INDEX idx_task_audit_logs_session_id ON task_audit_logs (session_id);

CREATE INDEX idx_audit_events_ts ON audit_events (ts DESC);
CREATE INDEX idx_audit_events_event_type ON audit_events (event_type);
CREATE INDEX idx_audit_events_severity ON audit_events (severity);
CREATE INDEX idx_audit_events_source ON audit_events (source);
CREATE INDEX idx_audit_events_user_id ON audit_events (user_id);
CREATE INDEX idx_audit_events_resource_type ON audit_events (resource_type, resource_id);
CREATE INDEX idx_audit_events_session_id ON audit_events (session_id);
CREATE INDEX idx_audit_events_success ON audit_events (success);

-- Create composite indexes for common query patterns
CREATE INDEX idx_task_audit_logs_task_ts ON task_audit_logs (task_id, ts DESC);
CREATE INDEX idx_audit_events_type_ts ON audit_events (event_type, ts DESC);
CREATE INDEX idx_audit_events_user_ts ON audit_events (user_id, ts DESC);

-- Add table comments
COMMENT ON TABLE task_audit_logs IS 'Comprehensive audit trail for all task lifecycle events and state changes';
COMMENT ON TABLE audit_events IS 'General audit events for system operations, API calls, and security events';

-- Add column comments for task_audit_logs
COMMENT ON COLUMN task_audit_logs.task_id IS 'ID of the task being audited';
COMMENT ON COLUMN task_audit_logs.user_id IS 'User who performed the action (if applicable)';
COMMENT ON COLUMN task_audit_logs.action IS 'Type of action performed on the task';
COMMENT ON COLUMN task_audit_logs.old_state IS 'Previous task state before the action';
COMMENT ON COLUMN task_audit_logs.new_state IS 'New task state after the action';
COMMENT ON COLUMN task_audit_logs.details IS 'Additional context and metadata for the audit event';
COMMENT ON COLUMN task_audit_logs.ip_address IS 'IP address of the client that performed the action';
COMMENT ON COLUMN task_audit_logs.user_agent IS 'Browser/client user agent string';
COMMENT ON COLUMN task_audit_logs.session_id IS 'Session ID for tracking user sessions';

-- Add column comments for audit_events
COMMENT ON COLUMN audit_events.event_type IS 'Category of audit event (api_call, auth_attempt, config_change, etc.)';
COMMENT ON COLUMN audit_events.severity IS 'Severity level of the event';
COMMENT ON COLUMN audit_events.source IS 'System component that generated the event';
COMMENT ON COLUMN audit_events.resource_type IS 'Type of resource affected by the event';
COMMENT ON COLUMN audit_events.resource_id IS 'ID of the specific resource affected';
COMMENT ON COLUMN audit_events.details IS 'Event-specific data and context';
COMMENT ON COLUMN audit_events.success IS 'Whether the operation completed successfully';
COMMENT ON COLUMN audit_events.processing_time_ms IS 'Time taken to process the operation in milliseconds';

-- ============================================================================
-- AUDIT FUNCTIONS & UTILITIES
-- ============================================================================

-- Function to get audit trail for a specific task
CREATE OR REPLACE FUNCTION get_task_audit_trail(
    p_task_id VARCHAR(255),
    p_limit INTEGER DEFAULT 100,
    p_since TIMESTAMPTZ DEFAULT NULL
)
RETURNS TABLE (
    id UUID,
    ts TIMESTAMPTZ,
    user_id VARCHAR(255),
    action VARCHAR(50),
    old_state VARCHAR(50),
    new_state VARCHAR(50),
    details JSONB,
    ip_address INET,
    user_agent TEXT,
    session_id VARCHAR(255)
)
LANGUAGE SQL
STABLE
AS $$
    SELECT
        tal.id,
        tal.ts,
        tal.user_id,
        tal.action,
        tal.old_state,
        tal.new_state,
        tal.details,
        tal.ip_address,
        tal.user_agent,
        tal.session_id
    FROM task_audit_logs tal
    WHERE tal.task_id = p_task_id
      AND (p_since IS NULL OR tal.ts >= p_since)
    ORDER BY tal.ts DESC
    LIMIT p_limit;
$$;

COMMENT ON FUNCTION get_task_audit_trail(VARCHAR, INTEGER, TIMESTAMPTZ) IS 'Get complete audit trail for a specific task with optional time filtering';

-- Function to get audit events by user
CREATE OR REPLACE FUNCTION get_user_audit_events(
    p_user_id VARCHAR(255),
    p_limit INTEGER DEFAULT 50,
    p_days_back INTEGER DEFAULT 30
)
RETURNS TABLE (
    id UUID,
    ts TIMESTAMPTZ,
    event_type VARCHAR(100),
    severity VARCHAR(20),
    source VARCHAR(100),
    resource_type VARCHAR(50),
    resource_id VARCHAR(255),
    action VARCHAR(50),
    details JSONB,
    success BOOLEAN,
    error_message TEXT,
    processing_time_ms INTEGER
)
LANGUAGE SQL
STABLE
AS $$
    SELECT
        ae.id,
        ae.ts,
        ae.event_type,
        ae.severity,
        ae.source,
        ae.resource_type,
        ae.resource_id,
        ae.action,
        ae.details,
        ae.success,
        ae.error_message,
        ae.processing_time_ms
    FROM audit_events ae
    WHERE ae.user_id = p_user_id
      AND ae.ts >= (now() - interval '1 day' * p_days_back)
    ORDER BY ae.ts DESC
    LIMIT p_limit;
$$;

COMMENT ON FUNCTION get_user_audit_events(VARCHAR, INTEGER, INTEGER) IS 'Get audit events for a specific user within a time window';

-- Function to get system audit summary
CREATE OR REPLACE FUNCTION get_audit_summary(
    p_days_back INTEGER DEFAULT 7
)
RETURNS TABLE (
    date DATE,
    total_events BIGINT,
    error_events BIGINT,
    warning_events BIGINT,
    critical_events BIGINT,
    unique_users BIGINT,
    top_event_types JSONB
)
LANGUAGE SQL
STABLE
AS $$
    WITH daily_stats AS (
        SELECT
            DATE(ae.ts) as event_date,
            COUNT(*) as total_count,
            COUNT(*) FILTER (WHERE ae.severity = 'error') as error_count,
            COUNT(*) FILTER (WHERE ae.severity = 'warning') as warning_count,
            COUNT(*) FILTER (WHERE ae.severity = 'critical') as critical_count,
            COUNT(DISTINCT ae.user_id) as user_count
        FROM audit_events ae
        WHERE ae.ts >= (now() - interval '1 day' * p_days_back)
        GROUP BY DATE(ae.ts)
    ),
    top_types AS (
        SELECT
            DATE(ae.ts) as event_date,
            jsonb_object_agg(
                ae.event_type,
                COUNT(*)
                ORDER BY COUNT(*) DESC
            ) as event_counts
        FROM audit_events ae
        WHERE ae.ts >= (now() - interval '1 day' * p_days_back)
        GROUP BY DATE(ae.ts)
    )
    SELECT
        ds.event_date,
        ds.total_count,
        ds.error_count,
        ds.warning_count,
        ds.critical_count,
        ds.user_count,
        tt.event_counts
    FROM daily_stats ds
    LEFT JOIN top_types tt ON ds.event_date = tt.event_date
    ORDER BY ds.event_date DESC;
$$;

COMMENT ON FUNCTION get_audit_summary(INTEGER) IS 'Get daily audit summary statistics for monitoring and compliance';

-- ============================================================================
-- DATA INTEGRITY & CLEANUP
-- ============================================================================

-- Function to clean up old audit events (keep last 90 days by default)
CREATE OR REPLACE FUNCTION cleanup_old_audit_events(
    p_retention_days INTEGER DEFAULT 90
)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM audit_events
    WHERE ts < (now() - interval '1 day' * p_retention_days);
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_audit_events(INTEGER) IS 'Clean up audit events older than specified days, returns count of deleted events';

-- Function to clean up old task audit logs (keep last 180 days by default)
CREATE OR REPLACE FUNCTION cleanup_old_task_audit_logs(
    p_retention_days INTEGER DEFAULT 180
)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM task_audit_logs
    WHERE ts < (now() - interval '1 day' * p_retention_days);
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_old_task_audit_logs(INTEGER) IS 'Clean up task audit logs older than specified days, returns count of deleted events';

-- ============================================================================
-- INITIAL DATA & TESTING
-- ============================================================================

-- Insert sample audit events for testing
INSERT INTO task_audit_logs (task_id, user_id, action, old_state, new_state, details, session_id) VALUES
('task-001', 'user-123', 'created', NULL, 'pending', '{"description": "Sample task creation"}'::jsonb, 'session-abc'),
('task-001', 'user-123', 'started', 'pending', 'running', '{"worker_id": "worker-1"}'::jsonb, 'session-abc'),
('task-001', 'user-123', 'completed', 'running', 'completed', '{"result": "success", "duration_ms": 1250}'::jsonb, 'session-abc');

INSERT INTO audit_events (event_type, severity, source, user_id, resource_type, resource_id, action, details, success, processing_time_ms, session_id) VALUES
('api_call', 'info', 'api-server', 'user-123', 'task', 'task-001', 'create', '{"endpoint": "/api/v1/tasks", "method": "POST"}'::jsonb, true, 45, 'session-abc'),
('api_call', 'info', 'api-server', 'user-123', 'task', 'task-001', 'update', '{"endpoint": "/api/v1/tasks/task-001/pause", "method": "POST"}'::jsonb, true, 23, 'session-abc'),
('system_event', 'info', 'orchestrator', NULL, 'system', 'health_check', 'check', '{"status": "healthy", "components": ["api", "database", "workers"]}'::jsonb, true, 150, NULL),
('auth_attempt', 'warning', 'api-server', 'unknown', 'auth', NULL, 'login', '{"reason": "invalid_credentials", "attempts": 3}'::jsonb, false, 120, 'session-failed');

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

-- Note: This migration creates comprehensive audit trail infrastructure.
-- The audit system supports:
-- - Complete task lifecycle tracking
-- - General system event auditing
-- - Compliance reporting and monitoring
-- - Automatic cleanup of old events
-- - Efficient querying for investigations
