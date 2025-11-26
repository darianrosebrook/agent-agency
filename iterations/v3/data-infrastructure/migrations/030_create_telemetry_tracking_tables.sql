-- Migration 030: Create Telemetry Tracking Tables
-- Creates database tables for telemetry data collection:
-- telemetry_model_contributions, telemetry_agent_activity, task_stats_history
-- @author @darianrosebrook

-- ===========================================
-- TELEMETRY: MODEL CONTRIBUTIONS TABLE
-- ===========================================
-- Tracks LLM model usage for analytics and cost monitoring

CREATE TABLE IF NOT EXISTS telemetry_model_contributions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    success_count INTEGER NOT NULL DEFAULT 0,
    failure_count INTEGER NOT NULL DEFAULT 0,
    avg_response_time_ms DOUBLE PRECISION,
    min_response_time_ms DOUBLE PRECISION,
    max_response_time_ms DOUBLE PRECISION,
    total_cost_usd DOUBLE PRECISION DEFAULT 0.0,
    metadata JSONB DEFAULT '{}',
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for telemetry_model_contributions
CREATE INDEX IF NOT EXISTS idx_telemetry_model_contributions_model_name 
    ON telemetry_model_contributions(model_name);
CREATE INDEX IF NOT EXISTS idx_telemetry_model_contributions_recorded_at 
    ON telemetry_model_contributions(recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_model_contributions_model_time 
    ON telemetry_model_contributions(model_name, recorded_at DESC);

-- ===========================================
-- TELEMETRY: AGENT ACTIVITY TABLE
-- ===========================================
-- Tracks agent execution events for activity monitoring

CREATE TABLE IF NOT EXISTS telemetry_agent_activity (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    activity_type VARCHAR(100) NOT NULL,
    activity_count INTEGER NOT NULL DEFAULT 1,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    duration_ms INTEGER,
    success BOOLEAN DEFAULT TRUE,
    error_message TEXT,
    metadata JSONB DEFAULT '{}',
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for telemetry_agent_activity
CREATE INDEX IF NOT EXISTS idx_telemetry_agent_activity_agent_id 
    ON telemetry_agent_activity(agent_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_agent_activity_activity_type 
    ON telemetry_agent_activity(activity_type);
CREATE INDEX IF NOT EXISTS idx_telemetry_agent_activity_recorded_at 
    ON telemetry_agent_activity(recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_agent_activity_task_id 
    ON telemetry_agent_activity(task_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_agent_activity_agent_time 
    ON telemetry_agent_activity(agent_id, recorded_at DESC);

-- ===========================================
-- TASK STATS HISTORY TABLE
-- ===========================================
-- Daily snapshots of task statistics for trend analysis

CREATE TABLE IF NOT EXISTS task_stats_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    snapshot_date DATE NOT NULL UNIQUE,
    total INTEGER NOT NULL DEFAULT 0,
    completed INTEGER NOT NULL DEFAULT 0,
    in_progress INTEGER NOT NULL DEFAULT 0,
    pending INTEGER NOT NULL DEFAULT 0,
    failed INTEGER NOT NULL DEFAULT 0,
    cancelled INTEGER NOT NULL DEFAULT 0,
    paused INTEGER NOT NULL DEFAULT 0,
    completion_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    success_rate DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    avg_completion_time_ms DOUBLE PRECISION,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for task_stats_history
CREATE INDEX IF NOT EXISTS idx_task_stats_history_snapshot_date 
    ON task_stats_history(snapshot_date DESC);

-- ===========================================
-- TELEMETRY: LLM REQUEST LOG TABLE
-- ===========================================
-- Detailed log of individual LLM requests for research/debugging

CREATE TABLE IF NOT EXISTS telemetry_llm_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    request_id UUID NOT NULL UNIQUE DEFAULT gen_random_uuid(),
    model_name VARCHAR(255) NOT NULL,
    provider VARCHAR(100) NOT NULL DEFAULT 'unknown',
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    agent_id UUID REFERENCES workers(id) ON DELETE SET NULL,
    prompt_tokens INTEGER NOT NULL DEFAULT 0,
    completion_tokens INTEGER NOT NULL DEFAULT 0,
    total_tokens INTEGER NOT NULL DEFAULT 0,
    response_time_ms INTEGER,
    success BOOLEAN NOT NULL DEFAULT TRUE,
    error_type VARCHAR(100),
    error_message TEXT,
    temperature DOUBLE PRECISION,
    max_tokens INTEGER,
    request_type VARCHAR(100) DEFAULT 'completion',
    cost_usd DOUBLE PRECISION DEFAULT 0.0,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for telemetry_llm_requests
CREATE INDEX IF NOT EXISTS idx_telemetry_llm_requests_model_name 
    ON telemetry_llm_requests(model_name);
CREATE INDEX IF NOT EXISTS idx_telemetry_llm_requests_task_id 
    ON telemetry_llm_requests(task_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_llm_requests_agent_id 
    ON telemetry_llm_requests(agent_id);
CREATE INDEX IF NOT EXISTS idx_telemetry_llm_requests_created_at 
    ON telemetry_llm_requests(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_telemetry_llm_requests_provider 
    ON telemetry_llm_requests(provider);
CREATE INDEX IF NOT EXISTS idx_telemetry_llm_requests_success 
    ON telemetry_llm_requests(success);

-- ===========================================
-- FUNCTION: Snapshot Task Stats
-- ===========================================
-- Function to create a daily snapshot of task statistics

CREATE OR REPLACE FUNCTION snapshot_task_stats()
RETURNS void AS $$
DECLARE
    today DATE := CURRENT_DATE;
    stats RECORD;
BEGIN
    -- Calculate current stats
    SELECT 
        COUNT(*) as total,
        COUNT(*) FILTER (WHERE status = 'completed') as completed,
        COUNT(*) FILTER (WHERE status = 'in_progress') as in_progress,
        COUNT(*) FILTER (WHERE status = 'pending') as pending,
        COUNT(*) FILTER (WHERE status = 'failed') as failed,
        COUNT(*) FILTER (WHERE status = 'cancelled') as cancelled,
        COUNT(*) FILTER (WHERE status = 'paused') as paused,
        CASE 
            WHEN COUNT(*) > 0 
            THEN (COUNT(*) FILTER (WHERE status = 'completed'))::DOUBLE PRECISION / COUNT(*)
            ELSE 0.0 
        END as completion_rate,
        CASE 
            WHEN (COUNT(*) FILTER (WHERE status IN ('completed', 'failed'))) > 0 
            THEN (COUNT(*) FILTER (WHERE status = 'completed'))::DOUBLE PRECISION / 
                 (COUNT(*) FILTER (WHERE status IN ('completed', 'failed')))
            ELSE 0.0 
        END as success_rate,
        AVG(
            CASE 
                WHEN status = 'completed' AND completed_at IS NOT NULL 
                THEN EXTRACT(EPOCH FROM (completed_at - created_at)) * 1000 
            END
        ) as avg_completion_time_ms
    INTO stats
    FROM tasks;

    -- Insert or update today's snapshot
    INSERT INTO task_stats_history (
        snapshot_date, total, completed, in_progress, pending, 
        failed, cancelled, paused, completion_rate, success_rate, 
        avg_completion_time_ms
    ) VALUES (
        today, stats.total, stats.completed, stats.in_progress, stats.pending,
        stats.failed, stats.cancelled, stats.paused, stats.completion_rate, 
        stats.success_rate, stats.avg_completion_time_ms
    )
    ON CONFLICT (snapshot_date) DO UPDATE SET
        total = EXCLUDED.total,
        completed = EXCLUDED.completed,
        in_progress = EXCLUDED.in_progress,
        pending = EXCLUDED.pending,
        failed = EXCLUDED.failed,
        cancelled = EXCLUDED.cancelled,
        paused = EXCLUDED.paused,
        completion_rate = EXCLUDED.completion_rate,
        success_rate = EXCLUDED.success_rate,
        avg_completion_time_ms = EXCLUDED.avg_completion_time_ms;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- FUNCTION: Aggregate Model Contributions
-- ===========================================
-- Function to aggregate LLM request data into model contributions

CREATE OR REPLACE FUNCTION aggregate_model_contributions()
RETURNS void AS $$
BEGIN
    INSERT INTO telemetry_model_contributions (
        model_name, request_count, total_tokens, prompt_tokens, 
        completion_tokens, success_count, failure_count,
        avg_response_time_ms, min_response_time_ms, max_response_time_ms,
        total_cost_usd, recorded_at
    )
    SELECT 
        model_name,
        COUNT(*) as request_count,
        SUM(total_tokens) as total_tokens,
        SUM(prompt_tokens) as prompt_tokens,
        SUM(completion_tokens) as completion_tokens,
        COUNT(*) FILTER (WHERE success = TRUE) as success_count,
        COUNT(*) FILTER (WHERE success = FALSE) as failure_count,
        AVG(response_time_ms) as avg_response_time_ms,
        MIN(response_time_ms) as min_response_time_ms,
        MAX(response_time_ms) as max_response_time_ms,
        SUM(cost_usd) as total_cost_usd,
        DATE_TRUNC('hour', created_at) as recorded_at
    FROM telemetry_llm_requests
    WHERE created_at >= NOW() - INTERVAL '1 hour'
    GROUP BY model_name, DATE_TRUNC('hour', created_at)
    ON CONFLICT DO NOTHING;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- VIEWS FOR TELEMETRY ANALYTICS
-- ===========================================

-- Model usage summary (last 24 hours)
CREATE OR REPLACE VIEW telemetry_model_usage_24h AS
SELECT
    model_name,
    SUM(request_count) as total_requests,
    SUM(total_tokens) as total_tokens,
    SUM(success_count) as successful_requests,
    SUM(failure_count) as failed_requests,
    CASE 
        WHEN SUM(request_count) > 0 
        THEN SUM(success_count)::DOUBLE PRECISION / SUM(request_count)
        ELSE 0.0 
    END as success_rate,
    AVG(avg_response_time_ms) as avg_response_time_ms,
    SUM(total_cost_usd) as total_cost_usd
FROM telemetry_model_contributions
WHERE recorded_at >= NOW() - INTERVAL '24 hours'
GROUP BY model_name
ORDER BY total_requests DESC;

-- Agent activity summary (last 24 hours)
CREATE OR REPLACE VIEW telemetry_agent_activity_24h AS
SELECT
    agent_id,
    w.name as agent_name,
    activity_type,
    SUM(activity_count) as total_activities,
    COUNT(*) FILTER (WHERE success = TRUE) as successful,
    COUNT(*) FILTER (WHERE success = FALSE) as failed,
    AVG(duration_ms) as avg_duration_ms
FROM telemetry_agent_activity a
LEFT JOIN workers w ON a.agent_id = w.id
WHERE recorded_at >= NOW() - INTERVAL '24 hours'
GROUP BY agent_id, w.name, activity_type
ORDER BY total_activities DESC;

-- Task completion trends (last 30 days)
CREATE OR REPLACE VIEW task_completion_trends_30d AS
SELECT
    snapshot_date,
    total,
    completed,
    in_progress,
    pending,
    failed,
    cancelled,
    completion_rate,
    success_rate,
    avg_completion_time_ms
FROM task_stats_history
WHERE snapshot_date >= CURRENT_DATE - INTERVAL '30 days'
ORDER BY snapshot_date DESC;

-- ===========================================
-- LOG MIGRATION
-- ===========================================

INSERT INTO migration_log (version, description, applied_at)
VALUES ('030', 'Create telemetry tracking tables (model_contributions, agent_activity, task_stats_history, llm_requests)', NOW())
ON CONFLICT (version) DO NOTHING;

