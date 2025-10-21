-- SLO Tracking and Alerting System
-- Service Level Objective (SLO) monitoring, tracking, and alerting

-- SLO Definitions table
CREATE TABLE IF NOT EXISTS slo_definitions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    service_name VARCHAR(255) NOT NULL,
    slo_type VARCHAR(50) NOT NULL CHECK (slo_type IN ('availability', 'latency', 'throughput', 'error_rate', 'custom')),
    target_value DECIMAL(5,4) NOT NULL CHECK (target_value >= 0 AND target_value <= 1), -- e.g., 0.99 for 99%
    window_minutes INTEGER NOT NULL DEFAULT 60,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    is_active BOOLEAN NOT NULL DEFAULT true
);

-- SLO Measurements table for time-series data
CREATE TABLE IF NOT EXISTS slo_measurements (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slo_id UUID NOT NULL REFERENCES slo_definitions(id) ON DELETE CASCADE,
    measured_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    actual_value DECIMAL(10,6) NOT NULL, -- Actual measured value (e.g., 0.985 for 98.5%)
    target_value DECIMAL(5,4) NOT NULL, -- Target for this measurement period
    is_violation BOOLEAN NOT NULL DEFAULT false,
    sample_count INTEGER NOT NULL DEFAULT 1,
    metadata JSONB -- Additional measurement data (e.g., error counts, latency percentiles)
);

-- SLO Alerts table
CREATE TABLE IF NOT EXISTS slo_alerts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slo_id UUID NOT NULL REFERENCES slo_definitions(id) ON DELETE CASCADE,
    alert_type VARCHAR(50) NOT NULL CHECK (alert_type IN ('warning', 'critical', 'violation', 'recovery')),
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    message TEXT NOT NULL,
    actual_value DECIMAL(10,6),
    target_value DECIMAL(5,4),
    triggered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ,
    acknowledged_at TIMESTAMPTZ,
    acknowledged_by VARCHAR(255),
    metadata JSONB -- Additional alert context
);

-- SLO Status snapshots table for current status
CREATE TABLE IF NOT EXISTS slo_status_snapshots (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    slo_id UUID NOT NULL REFERENCES slo_definitions(id) ON DELETE CASCADE,
    status VARCHAR(20) NOT NULL CHECK (status IN ('healthy', 'warning', 'critical', 'unknown')),
    current_value DECIMAL(10,6),
    target_value DECIMAL(5,4),
    error_budget_used DECIMAL(5,4), -- How much of error budget is consumed (0.0 to 1.0)
    time_remaining_minutes INTEGER,
    last_updated TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    next_evaluation TIMESTAMPTZ,
    metadata JSONB
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_slo_definitions_active ON slo_definitions(is_active) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_slo_definitions_service ON slo_definitions(service_name);
CREATE INDEX IF NOT EXISTS idx_slo_measurements_slo_time ON slo_measurements(slo_id, measured_at DESC);
CREATE INDEX IF NOT EXISTS idx_slo_measurements_violations ON slo_measurements(is_violation) WHERE is_violation = true;
CREATE INDEX IF NOT EXISTS idx_slo_alerts_slo_triggered ON slo_alerts(slo_id, triggered_at DESC);
CREATE INDEX IF NOT EXISTS idx_slo_alerts_unresolved ON slo_alerts(resolved_at) WHERE resolved_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_slo_status_snapshots_slo ON slo_status_snapshots(slo_id);
CREATE INDEX IF NOT EXISTS idx_slo_status_snapshots_status ON slo_status_snapshots(status);

-- Insert default SLO definitions for the Agent Agency system
INSERT INTO slo_definitions (name, description, service_name, slo_type, target_value, window_minutes) VALUES
    ('api_availability', 'API service availability (99.9% uptime)', 'api-server', 'availability', 0.999, 60),
    ('api_latency_p95', 'API response time P95 (250ms)', 'api-server', 'latency', 0.95, 60), -- 95% of requests under 250ms
    ('worker_execution_success', 'Task execution success rate (99%)', 'worker', 'error_rate', 0.99, 60),
    ('council_consensus_rate', 'Council decision consensus rate (95%)', 'council', 'custom', 0.95, 60),
    ('system_health', 'Overall system health score (95%)', 'system', 'custom', 0.95, 60)
ON CONFLICT (name) DO NOTHING;

-- Function to calculate SLO status from recent measurements
CREATE OR REPLACE FUNCTION calculate_slo_status(slo_id_param UUID, window_minutes_param INTEGER DEFAULT 60)
RETURNS TABLE (
    current_value DECIMAL(10,6),
    target_value DECIMAL(5,4),
    error_budget_used DECIMAL(5,4),
    time_remaining_minutes INTEGER,
    status VARCHAR(20)
) AS $$
DECLARE
    def_record RECORD;
    window_start TIMESTAMPTZ;
    total_measurements INTEGER;
    violation_measurements INTEGER;
    avg_actual DECIMAL(10,6);
BEGIN
    -- Get SLO definition
    SELECT * INTO def_record FROM slo_definitions WHERE id = slo_id_param AND is_active = true;

    IF NOT FOUND THEN
        RETURN;
    END IF;

    -- Calculate window start time
    window_start := NOW() - INTERVAL '1 minute' * window_minutes_param;

    -- Get measurements in the window
    SELECT
        COUNT(*) as total,
        COUNT(*) FILTER (WHERE is_violation = true) as violations,
        AVG(actual_value) as avg_value
    INTO total_measurements, violation_measurements, avg_actual
    FROM slo_measurements
    WHERE slo_id = slo_id_param
      AND measured_at >= window_start;

    -- Calculate metrics
    IF total_measurements = 0 THEN
        -- No data available
        RETURN QUERY SELECT
            NULL::DECIMAL(10,6),
            def_record.target_value,
            NULL::DECIMAL(5,4),
            window_minutes_param,
            'unknown'::VARCHAR(20);
    ELSE
        -- Calculate error budget used
        DECLARE
            error_rate DECIMAL(5,4) := violation_measurements::DECIMAL / total_measurements::DECIMAL;
            error_budget_used DECIMAL(5,4) := GREATEST(0, (1.0 - def_record.target_value) - error_rate) / (1.0 - def_record.target_value);
        BEGIN
            -- Determine status
            DECLARE
                status_val VARCHAR(20);
            BEGIN
                IF error_budget_used >= 1.0 THEN
                    status_val := 'critical';
                ELSIF error_budget_used >= 0.8 THEN
                    status_val := 'warning';
                ELSE
                    status_val := 'healthy';
                END IF;

                RETURN QUERY SELECT
                    avg_actual,
                    def_record.target_value,
                    LEAST(1.0, error_budget_used),
                    window_minutes_param,
                    status_val;
            END;
        END;
    END IF;
END;
$$ LANGUAGE plpgsql;

-- Function to get SLO alerts for a service
CREATE OR REPLACE FUNCTION get_service_slo_alerts(service_name_param VARCHAR, limit_param INTEGER DEFAULT 50)
RETURNS TABLE (
    alert_id UUID,
    slo_name VARCHAR,
    alert_type VARCHAR,
    severity VARCHAR,
    message TEXT,
    triggered_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        a.id,
        d.name,
        a.alert_type,
        a.severity,
        a.message,
        a.triggered_at,
        a.resolved_at
    FROM slo_alerts a
    JOIN slo_definitions d ON a.slo_id = d.id
    WHERE d.service_name = service_name_param
      AND (a.resolved_at IS NULL OR a.resolved_at > NOW() - INTERVAL '24 hours')
    ORDER BY a.triggered_at DESC
    LIMIT limit_param;
END;
$$ LANGUAGE plpgsql;

-- Function to update SLO status snapshots
CREATE OR REPLACE FUNCTION update_slo_status_snapshots()
RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER := 0;
    slo_record RECORD;
BEGIN
    -- Update status for all active SLOs
    FOR slo_record IN SELECT id FROM slo_definitions WHERE is_active = true LOOP
        -- Insert new status snapshot
        INSERT INTO slo_status_snapshots (
            slo_id,
            status,
            current_value,
            target_value,
            error_budget_used,
            time_remaining_minutes,
            last_updated,
            next_evaluation
        )
        SELECT
            slo_record.id,
            status,
            current_value,
            target_value,
            error_budget_used,
            time_remaining_minutes,
            NOW(),
            NOW() + INTERVAL '1 minute'
        FROM calculate_slo_status(slo_record.id);

        updated_count := updated_count + 1;
    END LOOP;

    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

-- Function to check and generate SLO alerts
CREATE OR REPLACE FUNCTION check_slo_alerts()
RETURNS INTEGER AS $$
DECLARE
    slo_record RECORD;
    status_record RECORD;
    alert_count INTEGER := 0;
BEGIN
    -- Check each active SLO
    FOR slo_record IN SELECT * FROM slo_definitions WHERE is_active = true LOOP
        -- Get current status
        SELECT * INTO status_record
        FROM slo_status_snapshots
        WHERE slo_id = slo_record.id
        ORDER BY last_updated DESC
        LIMIT 1;

        IF FOUND THEN
            -- Check for alerts based on status
            IF status_record.status = 'critical' THEN
                -- Check if we already have an active critical alert
                IF NOT EXISTS (
                    SELECT 1 FROM slo_alerts
                    WHERE slo_id = slo_record.id
                      AND alert_type = 'critical'
                      AND resolved_at IS NULL
                      AND triggered_at > NOW() - INTERVAL '1 hour'
                ) THEN
                    INSERT INTO slo_alerts (slo_id, alert_type, severity, message, actual_value, target_value)
                    VALUES (
                        slo_record.id,
                        'critical',
                        'critical',
                        format('SLO violation: %s is in critical state (%.2f%% error budget used)',
                               slo_record.name, status_record.error_budget_used * 100),
                        status_record.current_value,
                        status_record.target_value
                    );
                    alert_count := alert_count + 1;
                END IF;
            ELSIF status_record.status = 'warning' THEN
                -- Check if we already have an active warning alert
                IF NOT EXISTS (
                    SELECT 1 FROM slo_alerts
                    WHERE slo_id = slo_record.id
                      AND alert_type = 'warning'
                      AND resolved_at IS NULL
                      AND triggered_at > NOW() - INTERVAL '2 hours'
                ) THEN
                    INSERT INTO slo_alerts (slo_id, alert_type, severity, message, actual_value, target_value)
                    VALUES (
                        slo_record.id,
                        'warning',
                        'medium',
                        format('SLO warning: %s is in warning state (%.2f%% error budget used)',
                               slo_record.name, status_record.error_budget_used * 100),
                        status_record.current_value,
                        status_record.target_value
                    );
                    alert_count := alert_count + 1;
                END IF;
            END IF;

            -- Check for recovery from critical/warning
            IF status_record.status = 'healthy' THEN
                -- Resolve any active alerts for this SLO
                UPDATE slo_alerts
                SET resolved_at = NOW()
                WHERE slo_id = slo_record.id
                  AND resolved_at IS NULL;
            END IF;
        END IF;
    END LOOP;

    RETURN alert_count;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE slo_definitions IS 'Service Level Objective definitions with targets and windows';
COMMENT ON TABLE slo_measurements IS 'Time-series measurements for SLO compliance tracking';
COMMENT ON TABLE slo_alerts IS 'SLO violation alerts and notifications';
COMMENT ON TABLE slo_status_snapshots IS 'Current SLO status snapshots for real-time monitoring';
COMMENT ON FUNCTION calculate_slo_status(UUID, INTEGER) IS 'Calculate current SLO status from recent measurements';
COMMENT ON FUNCTION get_service_slo_alerts(VARCHAR, INTEGER) IS 'Get recent SLO alerts for a specific service';
COMMENT ON FUNCTION update_slo_status_snapshots() IS 'Update status snapshots for all active SLOs';
COMMENT ON FUNCTION check_slo_alerts() IS 'Check for SLO violations and generate alerts';
