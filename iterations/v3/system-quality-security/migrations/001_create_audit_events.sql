-- Audit Events Table
-- Stores comprehensive audit trails for security events and system activities

CREATE TABLE IF NOT EXISTS audit_events (
    -- Primary identifier
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    -- Event metadata
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    event_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('debug', 'info', 'warning', 'error', 'critical')),
    actor VARCHAR(255) NOT NULL,
    resource VARCHAR(500) NOT NULL,
    action VARCHAR(100) NOT NULL,
    result VARCHAR(50) NOT NULL CHECK (result IN ('success', 'failure', 'denied', 'timeout')),

    -- Event details (optional failure details for failure results)
    details TEXT,

    -- Structured context data (JSON)
    context JSONB DEFAULT '{}',

    -- Additional metadata (JSON)
    metadata JSONB DEFAULT '{}',

    -- Network and session information
    source_ip INET,
    user_agent TEXT,
    session_id UUID,
    request_id UUID
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_audit_timestamp ON audit_events(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_event_type ON audit_events(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_severity ON audit_events(severity);
CREATE INDEX IF NOT EXISTS idx_audit_actor ON audit_events(actor);
CREATE INDEX IF NOT EXISTS idx_audit_resource ON audit_events(resource);
CREATE INDEX IF NOT EXISTS idx_audit_result ON audit_events(result);
CREATE INDEX IF NOT EXISTS idx_audit_session ON audit_events(session_id);
CREATE INDEX IF NOT EXISTS idx_audit_request ON audit_events(request_id);
CREATE INDEX IF NOT EXISTS idx_audit_source_ip ON audit_events(source_ip);

-- Composite indexes for common queries
CREATE INDEX IF NOT EXISTS idx_audit_actor_resource ON audit_events(actor, resource);
CREATE INDEX IF NOT EXISTS idx_audit_timestamp_actor ON audit_events(timestamp DESC, actor);
CREATE INDEX IF NOT EXISTS idx_audit_event_severity ON audit_events(event_type, severity);

-- Partial indexes for critical events
CREATE INDEX IF NOT EXISTS idx_audit_critical_events ON audit_events(timestamp DESC)
    WHERE severity = 'critical';
CREATE INDEX IF NOT EXISTS idx_audit_failures ON audit_events(timestamp DESC)
    WHERE result IN ('failure', 'denied');

-- GIN indexes for JSON fields (for complex queries on context/metadata)
CREATE INDEX IF NOT EXISTS idx_audit_context_gin ON audit_events USING GIN (context);
CREATE INDEX IF NOT EXISTS idx_audit_metadata_gin ON audit_events USING GIN (metadata);

-- Partitioning setup (by month for large deployments)
-- Uncomment to enable partitioning:
-- CREATE TABLE audit_events_y2024m01 PARTITION OF audit_events
--     FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');

-- Retention policy function
CREATE OR REPLACE FUNCTION cleanup_old_audit_events(retention_days INTEGER DEFAULT 365)
RETURNS BIGINT AS $$
DECLARE
    deleted_count BIGINT;
BEGIN
    DELETE FROM audit_events
    WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Audit event statistics function
CREATE OR REPLACE FUNCTION get_audit_statistics(
    start_date TIMESTAMPTZ DEFAULT NULL,
    end_date TIMESTAMPTZ DEFAULT NULL
)
RETURNS TABLE (
    total_events BIGINT,
    events_by_type JSONB,
    events_by_severity JSONB,
    events_by_result JSONB,
    unique_actors BIGINT,
    unique_resources BIGINT,
    oldest_event TIMESTAMPTZ,
    newest_event TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(*) as total_events,
        jsonb_object_agg(event_type, cnt) as events_by_type,
        jsonb_object_agg(severity, cnt) as events_by_severity,
        jsonb_object_agg(result, cnt) as events_by_result,
        COUNT(DISTINCT actor) as unique_actors,
        COUNT(DISTINCT resource) as unique_resources,
        MIN(timestamp) as oldest_event,
        MAX(timestamp) as newest_event
    FROM (
        SELECT
            event_type,
            severity,
            result,
            actor,
            resource,
            timestamp
        FROM audit_events
        WHERE (start_date IS NULL OR timestamp >= start_date)
          AND (end_date IS NULL OR timestamp <= end_date)
    ) stats;
END;
$$ LANGUAGE plpgsql;

-- View for recent audit activity
CREATE OR REPLACE VIEW recent_audit_activity AS
SELECT
    id,
    timestamp,
    event_type,
    severity,
    actor,
    resource,
    action,
    result,
    details
FROM audit_events
WHERE timestamp > NOW() - INTERVAL '24 hours'
ORDER BY timestamp DESC
LIMIT 1000;

-- Comments for documentation
COMMENT ON TABLE audit_events IS 'Comprehensive audit trail for security events and system activities';
COMMENT ON COLUMN audit_events.id IS 'Unique identifier for each audit event';
COMMENT ON COLUMN audit_events.timestamp IS 'When the event occurred';
COMMENT ON COLUMN audit_events.event_type IS 'Type of event (authentication, authorization, etc.)';
COMMENT ON COLUMN audit_events.severity IS 'Severity level (debug, info, warning, error, critical)';
COMMENT ON COLUMN audit_events.actor IS 'User or system that performed the action';
COMMENT ON COLUMN audit_events.resource IS 'Resource that was accessed or modified';
COMMENT ON COLUMN audit_events.action IS 'Specific action performed';
COMMENT ON COLUMN audit_events.result IS 'Outcome of the action (success, failure, denied, timeout)';
COMMENT ON COLUMN audit_events.details IS 'Optional details, especially for failures';
COMMENT ON COLUMN audit_events.context IS 'Structured context data as JSON';
COMMENT ON COLUMN audit_events.metadata IS 'Additional metadata as JSON';
COMMENT ON COLUMN audit_events.source_ip IS 'IP address of the actor';
COMMENT ON COLUMN audit_events.user_agent IS 'User agent string if applicable';
COMMENT ON COLUMN audit_events.session_id IS 'Session identifier if applicable';
COMMENT ON COLUMN audit_events.request_id IS 'Request identifier for correlation';
