-- V3 verdicts and waivers tables

CREATE TABLE IF NOT EXISTS verdicts (
  id UUID PRIMARY KEY,
  task_id TEXT NOT NULL,
  decision TEXT NOT NULL CHECK (decision IN ('accept','reject','modify')),
  votes JSONB NOT NULL,
  dissent TEXT NOT NULL,
  remediation JSONB NOT NULL DEFAULT '[]'::jsonb,
  constitutional_refs TEXT[] NOT NULL DEFAULT '{}',
  signature BYTEA,
  hash_chain BYTEA,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_verdicts_created ON verdicts (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_verdicts_task ON verdicts (task_id);
CREATE INDEX IF NOT EXISTS idx_verdicts_constitutional ON verdicts USING GIN (constitutional_refs);

CREATE TABLE IF NOT EXISTS waivers (
  id TEXT PRIMARY KEY,
  reason TEXT NOT NULL,
  scope TEXT,
  task_id TEXT NOT NULL,
  verdict_id UUID REFERENCES verdicts(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_waivers_task ON waivers (task_id);

-- Migration: Learning Signals Infrastructure
-- Purpose: Add learning signals table for adaptive routing and performance tracking
-- Version: 002
-- Created: 2025-01-24

-- Learning signals table for capturing task outcomes and judge performance
CREATE TABLE learning_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    verdict_id UUID REFERENCES council_verdicts(id) ON DELETE CASCADE,

-- Task outcome classification
outcome_type TEXT NOT NULL CHECK (
    outcome_type IN (
        'success',
        'partial_success',
        'failure',
        'timeout'
    )
),
outcome_data JSONB NOT NULL, -- Serialized TaskOutcome data

-- Judge dissent tracking
dissent_data JSONB NOT NULL DEFAULT '[]'::jsonb, -- Array of JudgeDissent

-- Performance metrics
latency_ms INTEGER NOT NULL,
quality_score FLOAT NOT NULL CHECK (
    quality_score >= 0.0
    AND quality_score <= 1.0
),
caws_compliance_score FLOAT NOT NULL CHECK (
    caws_compliance_score >= 0.0
    AND caws_compliance_score <= 1.0
),
claim_verification_score FLOAT CHECK (
    claim_verification_score >= 0.0
    AND claim_verification_score <= 1.0
),

-- Resource usage metrics
cpu_usage_percent FLOAT NOT NULL CHECK (
    cpu_usage_percent >= 0.0
    AND cpu_usage_percent <= 100.0
),
memory_usage_mb INTEGER NOT NULL CHECK (memory_usage_mb >= 0),
thermal_status TEXT NOT NULL CHECK (
    thermal_status IN (
        'normal',
        'warning',
        'throttling',
        'critical'
    )
),
ane_utilization FLOAT CHECK (
    ane_utilization >= 0.0
    AND ane_utilization <= 1.0
),
gpu_utilization FLOAT CHECK (
    gpu_utilization >= 0.0
    AND gpu_utilization <= 1.0
),
energy_consumption FLOAT CHECK (energy_consumption >= 0.0),

-- Task complexity assessment
task_complexity JSONB NOT NULL, -- Serialized TaskComplexity

-- Worker performance (optional)
worker_performance JSONB, -- Serialized WorkerPerformanceMetrics

-- Timestamps
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_learning_signals_task_id ON learning_signals (task_id);

CREATE INDEX idx_learning_signals_verdict_id ON learning_signals (verdict_id);

CREATE INDEX idx_learning_signals_outcome_type ON learning_signals (outcome_type);

CREATE INDEX idx_learning_signals_created_at ON learning_signals (created_at);

CREATE INDEX idx_learning_signals_quality_score ON learning_signals (quality_score);

CREATE INDEX idx_learning_signals_latency_ms ON learning_signals (latency_ms);

-- Composite indexes for common queries
CREATE INDEX idx_learning_signals_task_outcome ON learning_signals (task_id, outcome_type);

CREATE INDEX idx_learning_signals_time_quality ON learning_signals (created_at, quality_score);

-- Performance metrics aggregation view
CREATE VIEW learning_signals_performance AS
SELECT
    DATE_TRUNC ('hour', created_at) as hour_bucket,
    COUNT(*) as total_signals,
    AVG(quality_score) as avg_quality_score,
    AVG(latency_ms) as avg_latency_ms,
    AVG(caws_compliance_score) as avg_caws_compliance,
    AVG(cpu_usage_percent) as avg_cpu_usage,
    AVG(memory_usage_mb) as avg_memory_usage,
    COUNT(*) FILTER (
        WHERE
            outcome_type = 'success'
    ) * 100.0 / COUNT(*) as success_rate,
    COUNT(*) FILTER (
        WHERE
            outcome_type = 'failure'
    ) * 100.0 / COUNT(*) as failure_rate,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'normal'
    ) * 100.0 / COUNT(*) as thermal_normal_rate
FROM learning_signals
GROUP BY
    DATE_TRUNC ('hour', created_at)
ORDER BY hour_bucket DESC;

-- Judge performance aggregation view
CREATE VIEW judge_performance_metrics AS
WITH
    judge_dissent_analysis AS (
        SELECT
            id,
            task_id,
            verdict_id,
            outcome_type,
            quality_score,
            latency_ms,
            jsonb_array_length (dissent_data) as dissent_count,
            created_at
        FROM learning_signals
    ),
    judge_verdict_mapping AS (
        SELECT l.id, l.task_id, l.verdict_id, l.outcome_type, l.quality_score, l.latency_ms, l.dissent_count, l.created_at, cv.judge_verdicts
        FROM
            judge_dissent_analysis l
            LEFT JOIN council_verdicts cv ON l.verdict_id = cv.id
    )
SELECT
    DATE_TRUNC ('day', created_at) as day_bucket,
    COUNT(*) as total_evaluations,
    AVG(quality_score) as avg_quality_score,
    AVG(latency_ms) as avg_latency_ms,
    AVG(dissent_count) as avg_dissent_count,
    COUNT(*) FILTER (
        WHERE
            outcome_type = 'success'
    ) * 100.0 / COUNT(*) as success_rate,
    COUNT(*) FILTER (
        WHERE
            dissent_count = 0
    ) * 100.0 / COUNT(*) as consensus_rate
FROM judge_verdict_mapping
GROUP BY
    DATE_TRUNC ('day', created_at)
ORDER BY day_bucket DESC;

-- Resource usage trends view
CREATE VIEW resource_usage_trends AS
SELECT
    DATE_TRUNC ('hour', created_at) as hour_bucket,
    AVG(cpu_usage_percent) as avg_cpu_usage,
    AVG(memory_usage_mb) as avg_memory_usage,
    AVG(COALESCE(ane_utilization, 0)) as avg_ane_utilization,
    AVG(COALESCE(gpu_utilization, 0)) as avg_gpu_utilization,
    AVG(
        COALESCE(energy_consumption, 0)
    ) as avg_energy_consumption,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'normal'
    ) * 100.0 / COUNT(*) as thermal_normal_rate,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'warning'
    ) * 100.0 / COUNT(*) as thermal_warning_rate,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'throttling'
    ) * 100.0 / COUNT(*) as thermal_throttling_rate
FROM learning_signals
GROUP BY
    DATE_TRUNC ('hour', created_at)
ORDER BY hour_bucket DESC;

-- Learning recommendations table
CREATE TABLE learning_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recommendation_type TEXT NOT NULL CHECK (recommendation_type IN (
        'routing_optimization', 'resource_allocation', 'judge_selection',
        'task_complexity_adjustment', 'caws_compliance_improvement', 'claim_verification_enhancement'
    )),
    priority TEXT NOT NULL CHECK (priority IN ('critical', 'high', 'medium', 'low')),
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,
    expected_impact FLOAT NOT NULL CHECK (expected_impact >= 0.0 AND expected_impact <= 1.0),
    implementation_effort TEXT NOT NULL CHECK (implementation_effort IN (
        'trivial', 'simple', 'moderate', 'complex', 'very_complex'
    )),
    evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'rejected')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    implemented_at TIMESTAMPTZ,
    implemented_by TEXT
);

-- Indexes for learning recommendations
CREATE INDEX idx_learning_recommendations_type ON learning_recommendations (recommendation_type);

CREATE INDEX idx_learning_recommendations_priority ON learning_recommendations (priority);

CREATE INDEX idx_learning_recommendations_status ON learning_recommendations (status);

CREATE INDEX idx_learning_recommendations_created_at ON learning_recommendations (created_at);

-- Performance benchmarks table for model evaluation
CREATE TABLE performance_benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    benchmark_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    task_surface TEXT NOT NULL,
    benchmark_type TEXT NOT NULL CHECK (benchmark_type IN ('micro', 'macro', 'smoke_test', 'capability_assessment', 'comparative')),

-- Benchmark scores
scores JSONB NOT NULL, -- {taskCompletionRate, cawsComplianceScore, efficiencyRating, ...}
metrics JSONB NOT NULL, -- {latencyP95Ms, rewardHackingIncidents, ...}

-- Benchmark metadata
duration_seconds INTEGER NOT NULL,
dataset_version TEXT,
environment_info JSONB,

-- Results
passed BOOLEAN NOT NULL, failure_reason TEXT,

-- Timestamps
started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance benchmarks
CREATE INDEX idx_performance_benchmarks_benchmark_id ON performance_benchmarks (benchmark_id);

CREATE INDEX idx_performance_benchmarks_model_id ON performance_benchmarks (model_id);

CREATE INDEX idx_performance_benchmarks_task_surface ON performance_benchmarks (task_surface);

CREATE INDEX idx_performance_benchmarks_type ON performance_benchmarks (benchmark_type);

CREATE INDEX idx_performance_benchmarks_passed ON performance_benchmarks (passed);

CREATE INDEX idx_performance_benchmarks_completed_at ON performance_benchmarks (completed_at);

-- Composite index for model performance tracking
CREATE INDEX idx_performance_benchmarks_model_surface_time ON performance_benchmarks (
    model_id,
    task_surface,
    completed_at
);

-- Model performance summary view
CREATE VIEW model_performance_summary AS
WITH latest_benchmarks AS (
    SELECT DISTINCT ON (model_id, task_surface)
        model_id,
        task_surface,
        scores,
        metrics,
        completed_at,
        passed
    FROM performance_benchmarks
    WHERE benchmark_type IN ('macro', 'capability_assessment')
    ORDER BY model_id, task_surface, completed_at DESC
)
SELECT 
    model_id,
    task_surface,
    scores->>'taskCompletionRate'::float as task_completion_rate,
    scores->>'cawsComplianceScore'::float as caws_compliance_score,
    scores->>'efficiencyRating'::float as efficiency_rating,
    metrics->>'latencyP95Ms'::integer as latency_p95_ms,
    metrics->>'rewardHackingIncidents'::integer as reward_hacking_incidents,
    passed,
    completed_at
FROM latest_benchmarks;

-- Functions for learning signal analysis

-- Function to get performance trends for an entity
CREATE OR REPLACE FUNCTION get_performance_trends(
    entity_type TEXT,
    entity_id TEXT,
    time_window_hours INTEGER DEFAULT 24
)
RETURNS TABLE (
    time_bucket TIMESTAMPTZ,
    total_signals BIGINT,
    avg_quality_score FLOAT,
    avg_latency_ms FLOAT,
    success_rate FLOAT,
    dissent_rate FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        DATE_TRUNC('hour', ls.created_at) as time_bucket,
        COUNT(*) as total_signals,
        AVG(ls.quality_score) as avg_quality_score,
        AVG(ls.latency_ms) as avg_latency_ms,
        COUNT(*) FILTER (WHERE ls.outcome_type = 'success') * 100.0 / COUNT(*) as success_rate,
        COUNT(*) FILTER (WHERE jsonb_array_length(ls.dissent_data) > 0) * 100.0 / COUNT(*) as dissent_rate
    FROM learning_signals ls
    WHERE ls.created_at >= NOW() - INTERVAL '1 hour' * time_window_hours
    AND (
        (entity_type = 'task_type' AND ls.task_id::text LIKE '%' || entity_id || '%')
        OR (entity_type = 'judge' AND ls.dissent_data @> ('[{"judge_id": "' || entity_id || '"}]')::jsonb)
        OR (entity_type = 'system' AND true)
    )
    GROUP BY DATE_TRUNC('hour', ls.created_at)
    ORDER BY time_bucket;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate recommendation confidence
CREATE OR REPLACE FUNCTION calculate_recommendation_confidence(
    similar_signals_count INTEGER,
    success_rate FLOAT
)
RETURNS FLOAT AS $$
BEGIN
    IF similar_signals_count = 0 THEN
        RETURN 0.5; -- Default confidence with no data
    END IF;
    
    -- Confidence based on success rate and sample size
    DECLARE
        sample_confidence FLOAT := LEAST(similar_signals_count::FLOAT / 100.0, 1.0);
        confidence FLOAT := success_rate * 0.7 + sample_confidence * 0.3;
    BEGIN
        RETURN confidence;
    END;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_learning_signals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_learning_signals_updated_at
    BEFORE UPDATE ON learning_signals
    FOR EACH ROW
    EXECUTE FUNCTION update_learning_signals_updated_at();

CREATE TRIGGER trigger_learning_recommendations_updated_at
    BEFORE UPDATE ON learning_recommendations
    FOR EACH ROW
    EXECUTE FUNCTION update_learning_signals_updated_at();

-- Add comments for documentation
COMMENT ON
TABLE learning_signals IS 'Captures learning signals from council decisions for adaptive routing and performance tracking';

COMMENT ON
TABLE learning_recommendations IS 'Stores learning recommendations generated from signal analysis';

COMMENT ON
TABLE performance_benchmarks IS 'Stores benchmark results for model performance evaluation';

COMMENT ON COLUMN learning_signals.outcome_data IS 'Serialized TaskOutcome enum data';

COMMENT ON COLUMN learning_signals.dissent_data IS 'Array of JudgeDissent objects';

COMMENT ON COLUMN learning_signals.task_complexity IS 'Serialized TaskComplexity assessment';

COMMENT ON COLUMN learning_signals.worker_performance IS 'Optional serialized WorkerPerformanceMetrics';

COMMENT ON VIEW learning_signals_performance IS 'Hourly aggregated performance metrics from learning signals';

COMMENT ON VIEW judge_performance_metrics IS 'Daily aggregated judge performance metrics';

COMMENT ON VIEW resource_usage_trends IS 'Hourly resource usage trends and thermal status';

COMMENT ON VIEW model_performance_summary IS 'Latest benchmark results per model and task surface';
-- Production Database Hardening and Performance Optimizations
-- Migration: 003

-- Enable advanced PostgreSQL features for production
CREATE EXTENSION IF NOT EXISTS "pg_stat_statements";
CREATE EXTENSION IF NOT EXISTS "pg_buffercache";
CREATE EXTENSION IF NOT EXISTS "pg_prewarm";

-- Create monitoring tables for database health and performance
CREATE TABLE IF NOT EXISTS database_health_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(15, 4) NOT NULL,
    metric_unit VARCHAR(20),
    recorded_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    metadata JSONB DEFAULT '{}'
);

CREATE INDEX IF NOT EXISTS idx_health_metrics_name_time ON database_health_metrics (metric_name, recorded_at DESC);

-- Create connection tracking table for monitoring
CREATE TABLE IF NOT EXISTS connection_tracking (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    client_ip INET,
    client_port INTEGER,
    database_name VARCHAR(100),
    user_name VARCHAR(100),
    application_name VARCHAR(255),
    state VARCHAR(20),
    connected_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_activity_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    query_count INTEGER DEFAULT 0,
    bytes_sent BIGINT DEFAULT 0,
    bytes_received BIGINT DEFAULT 0
);

CREATE INDEX IF NOT EXISTS idx_connection_tracking_activity ON connection_tracking (last_activity_at DESC);
CREATE INDEX IF NOT EXISTS idx_connection_tracking_state ON connection_tracking (state);

-- Create slow query log table
CREATE TABLE IF NOT EXISTS slow_query_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    query_hash VARCHAR(64) NOT NULL,
    query_text TEXT NOT NULL,
    execution_time_ms INTEGER NOT NULL,
    rows_affected INTEGER,
    bytes_used BIGINT,
    client_ip INET,
    database_name VARCHAR(100),
    user_name VARCHAR(100),
    occurred_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_slow_query_hash ON slow_query_log (query_hash);
CREATE INDEX IF NOT EXISTS idx_slow_query_time ON slow_query_log (occurred_at DESC);
CREATE INDEX IF NOT EXISTS idx_slow_query_performance ON slow_query_log (execution_time_ms DESC);

-- Enhanced constraints and indexes for data integrity

-- Add NOT NULL constraints where missing
ALTER TABLE tasks
    ALTER COLUMN title SET NOT NULL,
    ALTER COLUMN description SET NOT NULL,
    ALTER COLUMN risk_tier SET NOT NULL,
    ALTER COLUMN status SET NOT NULL;

ALTER TABLE task_executions
    ALTER COLUMN task_id SET NOT NULL,
    ALTER COLUMN worker_id SET NOT NULL,
    ALTER COLUMN status SET NOT NULL,
    ALTER COLUMN worker_output SET NOT NULL;

ALTER TABLE council_verdicts
    ALTER COLUMN task_id SET NOT NULL,
    ALTER COLUMN verdict_id SET NOT NULL,
    ALTER COLUMN consensus_score SET NOT NULL,
    ALTER COLUMN final_verdict SET NOT NULL,
    ALTER COLUMN evaluation_time_ms SET NOT NULL;

-- Add CHECK constraints for data validation
ALTER TABLE tasks ADD CONSTRAINT chk_risk_tier_valid
    CHECK (risk_tier IN ('Tier1', 'Tier2', 'Tier3'));

ALTER TABLE task_executions ADD CONSTRAINT chk_execution_status_valid
    CHECK (status IN ('running', 'completed', 'failed', 'timeout'));

ALTER TABLE council_verdicts ADD CONSTRAINT chk_consensus_score_range
    CHECK (consensus_score >= 0.0 AND consensus_score <= 1.0);

-- Performance optimizations

-- Add composite indexes for common query patterns
CREATE INDEX IF NOT EXISTS idx_tasks_status_risk_tier ON tasks (status, risk_tier);
CREATE INDEX IF NOT EXISTS idx_tasks_worker_status ON tasks (assigned_worker_id, status);

CREATE INDEX IF NOT EXISTS idx_executions_task_status ON task_executions (task_id, status);
CREATE INDEX IF NOT EXISTS idx_executions_worker_status ON task_executions (worker_id, status);

-- Partial indexes for active records (performance optimization)
CREATE INDEX IF NOT EXISTS idx_tasks_active ON tasks (id) WHERE status IN ('pending', 'in_progress');
CREATE INDEX IF NOT EXISTS idx_workers_active ON workers (id) WHERE is_active = true;
CREATE INDEX IF NOT EXISTS idx_judges_active ON judges (id) WHERE is_active = true;

-- GIN indexes for JSONB columns (for efficient JSON queries)
CREATE INDEX IF NOT EXISTS idx_tasks_scope ON tasks USING GIN (scope);
CREATE INDEX IF NOT EXISTS idx_tasks_acceptance_criteria ON tasks USING GIN (acceptance_criteria);
CREATE INDEX IF NOT EXISTS idx_tasks_context ON tasks USING GIN (context);

CREATE INDEX IF NOT EXISTS idx_executions_worker_output ON task_executions USING GIN (worker_output);
CREATE INDEX IF NOT EXISTS idx_executions_metadata ON task_executions USING GIN (metadata);

CREATE INDEX IF NOT EXISTS idx_verdicts_individual_verdicts ON council_verdicts USING GIN (individual_verdicts);
CREATE INDEX IF NOT EXISTS idx_verdicts_final_verdict ON council_verdicts USING GIN (final_verdict);

-- Foreign key constraints for referential integrity
-- (Add these after ensuring data consistency)

-- ALTER TABLE task_executions ADD CONSTRAINT fk_executions_task
--     FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE;

-- ALTER TABLE task_executions ADD CONSTRAINT fk_executions_worker
--     FOREIGN KEY (worker_id) REFERENCES workers(id);

-- ALTER TABLE council_verdicts ADD CONSTRAINT fk_verdicts_task
--     FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE;

-- ALTER TABLE judge_evaluations ADD CONSTRAINT fk_evaluations_verdict
--     FOREIGN KEY (verdict_id) REFERENCES council_verdicts(verdict_id) ON DELETE CASCADE;

-- ALTER TABLE judge_evaluations ADD CONSTRAINT fk_evaluations_judge
--     FOREIGN KEY (judge_id) REFERENCES judges(id);

-- ALTER TABLE debate_sessions ADD CONSTRAINT fk_debate_task
--     FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE;

-- ALTER TABLE caws_compliance ADD CONSTRAINT fk_compliance_task
--     FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE;

-- Security hardening

-- Create read-only role for monitoring queries
-- Note: This should be done manually or through application setup
-- CREATE ROLE monitoring_ro WITH LOGIN PASSWORD 'secure_password';
-- GRANT CONNECT ON DATABASE agent_agency_v3 TO monitoring_ro;
-- GRANT USAGE ON SCHEMA public TO monitoring_ro;
-- GRANT SELECT ON ALL TABLES IN SCHEMA public TO monitoring_ro;

-- Set proper permissions on sensitive tables
-- GRANT SELECT, INSERT, UPDATE, DELETE ON tasks TO agent_agency_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON task_executions TO agent_agency_app;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON council_verdicts TO agent_agency_app;

-- Revoke public access to sensitive information
-- REVOKE ALL ON audit_trail FROM PUBLIC;
-- GRANT SELECT, INSERT ON audit_trail TO agent_agency_app;

-- Enable row-level security on audit trail for compliance
-- ALTER TABLE audit_trail ENABLE ROW LEVEL SECURITY;
-- CREATE POLICY audit_trail_policy ON audit_trail FOR SELECT USING (true);

-- Functions for database maintenance and monitoring

-- Function to get table sizes for monitoring
CREATE OR REPLACE FUNCTION get_table_sizes()
RETURNS TABLE(table_name TEXT, size_bytes BIGINT) AS $$
BEGIN
    RETURN QUERY
    SELECT
        schemaname || '.' || tablename as table_name,
        pg_total_relation_size(schemaname || '.' || tablename) as size_bytes
    FROM pg_tables
    WHERE schemaname = 'public'
    ORDER BY size_bytes DESC;
END;
$$ LANGUAGE plpgsql;

-- Function to get index usage statistics
CREATE OR REPLACE FUNCTION get_index_usage()
RETURNS TABLE(index_name TEXT, table_name TEXT, scans BIGINT, size_bytes BIGINT) AS $$
BEGIN
    RETURN QUERY
    SELECT
        schemaname || '.' || indexname as index_name,
        schemaname || '.' || tablename as table_name,
        idx_scan as scans,
        pg_relation_size(schemaname || '.' || indexname) as size_bytes
    FROM pg_stat_user_indexes
    WHERE schemaname = 'public'
    ORDER BY idx_scan DESC;
END;
$$ LANGUAGE plpgsql;

-- Function to get slow queries from pg_stat_statements (if available)
CREATE OR REPLACE FUNCTION get_slow_queries(threshold_ms INTEGER DEFAULT 1000)
RETURNS TABLE(query TEXT, calls BIGINT, total_time DECIMAL, mean_time DECIMAL) AS $$
BEGIN
    RETURN QUERY
    SELECT
        query,
        calls,
        total_time,
        mean_time
    FROM pg_stat_statements
    WHERE mean_time > threshold_ms
    ORDER BY mean_time DESC
    LIMIT 20;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup old performance metrics (keep last 30 days)
CREATE OR REPLACE FUNCTION cleanup_performance_metrics()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM performance_metrics
    WHERE recorded_at < NOW() - INTERVAL '30 days';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup old slow query logs (keep last 7 days)
CREATE OR REPLACE FUNCTION cleanup_slow_query_logs()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM slow_query_log
    WHERE occurred_at < NOW() - INTERVAL '7 days';

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Scheduled maintenance function (to be called periodically)
CREATE OR REPLACE FUNCTION run_database_maintenance()
RETURNS JSONB AS $$
DECLARE
    metrics_cleanup INTEGER;
    slow_query_cleanup INTEGER;
    total_size BIGINT;
BEGIN
    -- Cleanup old metrics
    metrics_cleanup := cleanup_performance_metrics();

    -- Cleanup old slow query logs
    slow_query_cleanup := cleanup_slow_query_logs();

    -- Get total database size
    SELECT SUM(size_bytes) INTO total_size FROM get_table_sizes();

    RETURN jsonb_build_object(
        'metrics_cleaned', metrics_cleanup,
        'slow_queries_cleaned', slow_query_cleanup,
        'total_database_size_bytes', total_size,
        'maintenance_run_at', NOW()
    );
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE database_health_metrics IS 'Database health and performance metrics for monitoring';
COMMENT ON TABLE connection_tracking IS 'Active database connection tracking for monitoring';
COMMENT ON TABLE slow_query_log IS 'Log of slow queries for performance analysis';

COMMENT ON FUNCTION get_table_sizes() IS 'Returns size information for all tables in the database';
COMMENT ON FUNCTION get_index_usage() IS 'Returns index usage statistics for performance analysis';
COMMENT ON FUNCTION get_slow_queries(threshold_ms INTEGER) IS 'Returns slow queries above the specified threshold';
COMMENT ON FUNCTION cleanup_performance_metrics() IS 'Cleans up old performance metrics (older than 30 days)';
COMMENT ON FUNCTION cleanup_slow_query_logs() IS 'Cleans up old slow query logs (older than 7 days)';
COMMENT ON FUNCTION run_database_maintenance() IS 'Runs comprehensive database maintenance tasks';


-- Migration 004: Add Provenance Tables
-- Adds tables for provenance record storage and management

-- Provenance records table
CREATE TABLE provenance_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    verdict_id UUID NOT NULL,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    decision_type VARCHAR(50) NOT NULL CHECK (
        decision_type IN ('accept', 'reject', 'require_modification', 'need_investigation')
    ),
    decision_data JSONB NOT NULL,
    consensus_score DECIMAL(3, 2) NOT NULL,
    judge_verdicts JSONB NOT NULL DEFAULT '{}',
    caws_compliance JSONB NOT NULL,
    claim_verification JSONB,
    git_commit_hash VARCHAR(40),
    git_trailer TEXT NOT NULL,
    signature TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for provenance records
CREATE INDEX idx_provenance_records_verdict_id ON provenance_records(verdict_id);
CREATE INDEX idx_provenance_records_task_id ON provenance_records(task_id);
CREATE INDEX idx_provenance_records_decision_type ON provenance_records(decision_type);
CREATE INDEX idx_provenance_records_timestamp ON provenance_records(timestamp);
CREATE INDEX idx_provenance_records_git_commit_hash ON provenance_records(git_commit_hash);

-- Trigger for automatic timestamp updates
CREATE TRIGGER update_provenance_records_updated_at 
    BEFORE UPDATE ON provenance_records 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Function to get provenance statistics
CREATE OR REPLACE FUNCTION get_provenance_statistics(
    p_time_range_start TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_time_range_end TIMESTAMP WITH TIME ZONE DEFAULT NULL
)
RETURNS JSONB AS $$
DECLARE
    result JSONB;
    time_filter TEXT := '';
BEGIN
    -- Build time filter if provided
    IF p_time_range_start IS NOT NULL AND p_time_range_end IS NOT NULL THEN
        time_filter := 'WHERE timestamp BETWEEN $1 AND $2';
    END IF;

    -- Execute dynamic query to get statistics
    EXECUTE format('
        SELECT jsonb_build_object(
            ''total_records'', COUNT(*),
            ''total_verdicts'', COUNT(DISTINCT verdict_id),
            ''acceptance_rate'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN 
                        COUNT(CASE WHEN decision_type = ''accept'' THEN 1 END)::DECIMAL / COUNT(*)
                    ELSE 0 
                END,
            ''average_consensus_score'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN AVG(consensus_score)
                    ELSE 0 
                END,
            ''average_compliance_score'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN AVG((caws_compliance->>''compliance_score'')::DECIMAL)
                    ELSE 0 
                END,
            ''average_verification_quality'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN AVG((claim_verification->>''verification_quality'')::DECIMAL)
                    ELSE 0 
                END,
            ''most_active_judge'', 
                COALESCE(
                    (SELECT judge_id 
                     FROM (
                         SELECT jsonb_object_keys(judge_verdicts) as judge_id
                         FROM provenance_records %s
                     ) judge_counts
                     GROUP BY judge_id 
                     ORDER BY COUNT(*) DESC 
                     LIMIT 1), 
                    ''Unknown''
                ),
            ''common_violations'', 
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            ''rule'', rule,
                            ''count'', count,
                            ''severity_distribution'', severity_distribution,
                            ''average_resolution_time_ms'', average_resolution_time_ms
                        )
                    )
                    FROM (
                        SELECT 
                            violation->>''rule'' as rule,
                            COUNT(*) as count,
                            jsonb_build_object() as severity_distribution,
                            0.0 as average_resolution_time_ms
                        FROM provenance_records %s,
                             jsonb_array_elements(caws_compliance->''violations'') as violation
                        GROUP BY violation->>''rule''
                        ORDER BY count DESC
                        LIMIT 10
                    ) violation_stats), 
                    ''[]''::jsonb
                ),
            ''time_range'', jsonb_build_object(
                ''start'', COALESCE(MIN(timestamp), NOW()),
                ''end'', COALESCE(MAX(timestamp), NOW())
            )
        )
        FROM provenance_records %s
    ', time_filter, time_filter, time_filter)
    USING p_time_range_start, p_time_range_end, p_time_range_start, p_time_range_end
    INTO result;

    RETURN COALESCE(result, '{}'::jsonb);
END;
$$ LANGUAGE plpgsql;

-- Function to query provenance records with filters
CREATE OR REPLACE FUNCTION query_provenance_records(
    p_task_id UUID DEFAULT NULL,
    p_verdict_id UUID DEFAULT NULL,
    p_decision_type VARCHAR(50) DEFAULT NULL,
    p_time_range_start TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_time_range_end TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_judge_id VARCHAR(255) DEFAULT NULL,
    p_compliance_status VARCHAR(50) DEFAULT NULL,
    p_limit INTEGER DEFAULT 1000,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id UUID,
    verdict_id UUID,
    task_id UUID,
    decision_type VARCHAR(50),
    decision_data JSONB,
    consensus_score DECIMAL(3, 2),
    judge_verdicts JSONB,
    caws_compliance JSONB,
    claim_verification JSONB,
    git_commit_hash VARCHAR(40),
    git_trailer TEXT,
    signature TEXT,
    timestamp TIMESTAMP WITH TIME ZONE,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        pr.id,
        pr.verdict_id,
        pr.task_id,
        pr.decision_type,
        pr.decision_data,
        pr.consensus_score,
        pr.judge_verdicts,
        pr.caws_compliance,
        pr.claim_verification,
        pr.git_commit_hash,
        pr.git_trailer,
        pr.signature,
        pr.timestamp,
        pr.metadata,
        pr.created_at,
        pr.updated_at
    FROM provenance_records pr
    WHERE 
        (p_task_id IS NULL OR pr.task_id = p_task_id)
        AND (p_verdict_id IS NULL OR pr.verdict_id = p_verdict_id)
        AND (p_decision_type IS NULL OR pr.decision_type = p_decision_type)
        AND (p_time_range_start IS NULL OR pr.timestamp >= p_time_range_start)
        AND (p_time_range_end IS NULL OR pr.timestamp <= p_time_range_end)
        AND (p_judge_id IS NULL OR pr.judge_verdicts ? p_judge_id)
        AND (p_compliance_status IS NULL OR 
             (p_compliance_status = 'compliant' AND (pr.caws_compliance->>'is_compliant')::BOOLEAN = true) OR
             (p_compliance_status = 'non_compliant' AND (pr.caws_compliance->>'is_compliant')::BOOLEAN = false) OR
             (p_compliance_status = 'partial_compliance' AND (pr.caws_compliance->>'compliance_score')::DECIMAL < 1.0 AND (pr.caws_compliance->>'is_compliant')::BOOLEAN = true))
    ORDER BY pr.timestamp DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE provenance_records IS 'Complete provenance records for CAWS verdicts with full audit trail';
COMMENT ON COLUMN provenance_records.decision_data IS 'Full decision data including confidence, summary, and reasoning';
COMMENT ON COLUMN provenance_records.judge_verdicts IS 'Individual judge verdicts contributing to the final decision';
COMMENT ON COLUMN provenance_records.caws_compliance IS 'CAWS compliance data including violations and waivers';
COMMENT ON COLUMN provenance_records.claim_verification IS 'Claim verification data and evidence quality scores';
COMMENT ON COLUMN provenance_records.git_trailer IS 'Git trailer for commit attribution and tracking';
COMMENT ON COLUMN provenance_records.signature IS 'Cryptographic signature for integrity verification';
-- Migration 005: Add Source Integrity Verification Tables
-- Adds tables for source integrity verification and hash storage

-- Source integrity records table
CREATE TABLE source_integrity_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_id VARCHAR(255) NOT NULL,
    source_type VARCHAR(50) NOT NULL CHECK (
        source_type IN ('file', 'url', 'content', 'code', 'document')
    ),
    content_hash VARCHAR(128) NOT NULL, -- Cryptographic hash as hex string (supports SHA-512)
    content_size BIGINT NOT NULL,
    hash_algorithm VARCHAR(20) NOT NULL DEFAULT 'sha256',
    integrity_status VARCHAR(20) NOT NULL CHECK (
        integrity_status IN ('verified', 'tampered', 'unknown', 'pending')
    ),
    tampering_indicators JSONB NOT NULL DEFAULT '[]',
    verification_metadata JSONB NOT NULL DEFAULT '{}',
    first_seen_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_verified_at TIMESTAMP WITH TIME ZONE,
    verification_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Ensure unique source_id per source_type
    UNIQUE(source_id, source_type)
);

-- Source integrity verification history
CREATE TABLE source_integrity_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_integrity_id UUID NOT NULL REFERENCES source_integrity_records(id) ON DELETE CASCADE,
    verification_type VARCHAR(50) NOT NULL CHECK (
        verification_type IN ('initial', 'periodic', 'on_access', 'manual', 'automated')
    ),
    verification_result VARCHAR(20) NOT NULL CHECK (
        verification_result IN ('passed', 'failed', 'warning', 'error')
    ),
    calculated_hash VARCHAR(128) NOT NULL,
    stored_hash VARCHAR(128) NOT NULL,
    hash_match BOOLEAN NOT NULL,
    tampering_detected BOOLEAN NOT NULL DEFAULT false,
    verification_details JSONB NOT NULL DEFAULT '{}',
    verified_by VARCHAR(255), -- system component or user
    verification_duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Source integrity alerts and notifications
CREATE TABLE source_integrity_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_integrity_id UUID NOT NULL REFERENCES source_integrity_records(id) ON DELETE CASCADE,
    alert_type VARCHAR(50) NOT NULL CHECK (
        alert_type IN ('tampering_detected', 'hash_mismatch', 'verification_failed', 'integrity_unknown')
    ),
    severity VARCHAR(20) NOT NULL CHECK (
        severity IN ('low', 'medium', 'high', 'critical')
    ),
    alert_message TEXT NOT NULL,
    alert_data JSONB NOT NULL DEFAULT '{}',
    acknowledged BOOLEAN NOT NULL DEFAULT false,
    acknowledged_by VARCHAR(255),
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by VARCHAR(255),
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_source_integrity_records_source_id ON source_integrity_records(source_id);
CREATE INDEX idx_source_integrity_records_source_type ON source_integrity_records(source_type);
CREATE INDEX idx_source_integrity_records_content_hash ON source_integrity_records(content_hash);
CREATE INDEX idx_source_integrity_records_integrity_status ON source_integrity_records(integrity_status);
CREATE INDEX idx_source_integrity_records_last_verified ON source_integrity_records(last_verified_at);
CREATE INDEX idx_source_integrity_records_created_at ON source_integrity_records(created_at);

CREATE INDEX idx_source_integrity_verifications_source_id ON source_integrity_verifications(source_integrity_id);
CREATE INDEX idx_source_integrity_verifications_verification_type ON source_integrity_verifications(verification_type);
CREATE INDEX idx_source_integrity_verifications_verification_result ON source_integrity_verifications(verification_result);
CREATE INDEX idx_source_integrity_verifications_created_at ON source_integrity_verifications(created_at);
CREATE INDEX idx_source_integrity_verifications_hash_match ON source_integrity_verifications(hash_match);

CREATE INDEX idx_source_integrity_alerts_source_id ON source_integrity_alerts(source_integrity_id);
CREATE INDEX idx_source_integrity_alerts_alert_type ON source_integrity_alerts(alert_type);
CREATE INDEX idx_source_integrity_alerts_severity ON source_integrity_alerts(severity);
CREATE INDEX idx_source_integrity_alerts_acknowledged ON source_integrity_alerts(acknowledged);
CREATE INDEX idx_source_integrity_alerts_resolved ON source_integrity_alerts(resolved);
CREATE INDEX idx_source_integrity_alerts_created_at ON source_integrity_alerts(created_at);

-- Triggers for automatic timestamp updates
CREATE TRIGGER update_source_integrity_records_updated_at 
    BEFORE UPDATE ON source_integrity_records 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate content hash using PostgreSQL's cryptographic functions
CREATE OR REPLACE FUNCTION calculate_content_hash(
    p_content TEXT,
    p_algorithm VARCHAR(20) DEFAULT 'sha256'
)
RETURNS VARCHAR(128) AS $$
DECLARE
    v_hash BYTEA;
    v_hash_hex VARCHAR(128);
BEGIN
    -- Validate algorithm parameter
    IF p_algorithm NOT IN ('md5', 'sha1', 'sha224', 'sha256', 'sha384', 'sha512') THEN
        RAISE EXCEPTION 'Unsupported hash algorithm: %. Supported algorithms: md5, sha1, sha224, sha256, sha384, sha512', p_algorithm;
    END IF;

    -- Calculate hash using PostgreSQL's built-in digest function
    v_hash := digest(p_content, p_algorithm);

    -- Convert to hex string
    v_hash_hex := encode(v_hash, 'hex');

    RETURN v_hash_hex;
EXCEPTION
    WHEN OTHERS THEN
        -- Log the error and fallback to SHA-256
        RAISE WARNING 'Hash calculation failed with algorithm %, falling back to SHA-256. Error: %', p_algorithm, SQLERRM;

        -- Calculate hash with fallback algorithm
        v_hash := digest(p_content, 'sha256');
        v_hash_hex := encode(v_hash, 'hex');

        RETURN v_hash_hex;
END;
$$ LANGUAGE plpgsql;

-- Function to verify source integrity
CREATE OR REPLACE FUNCTION verify_source_integrity(
    p_source_id VARCHAR(255),
    p_source_type VARCHAR(50),
    p_content TEXT,
    p_algorithm VARCHAR(20) DEFAULT 'sha256'
)
RETURNS JSONB AS $$
DECLARE
    v_calculated_hash VARCHAR(64);
    v_stored_record RECORD;
    v_result JSONB;
    v_hash_match BOOLEAN;
    v_tampering_detected BOOLEAN := false;
BEGIN
    -- Calculate hash of provided content
    v_calculated_hash := calculate_content_hash(p_content, p_algorithm);
    
    -- Look up stored record
    SELECT * INTO v_stored_record
    FROM source_integrity_records
    WHERE source_id = p_source_id AND source_type = p_source_type;
    
    -- Determine if hashes match
    IF v_stored_record.id IS NULL THEN
        -- New source, create record
        INSERT INTO source_integrity_records (
            source_id, source_type, content_hash, content_size, 
            hash_algorithm, integrity_status, verification_count
        ) VALUES (
            p_source_id, p_source_type, v_calculated_hash, 
            length(p_content), p_algorithm, 'verified', 1
        );
        
        v_hash_match := true;
        v_tampering_detected := false;
    ELSE
        -- Existing source, compare hashes
        v_hash_match := (v_calculated_hash = v_stored_record.content_hash);
        v_tampering_detected := NOT v_hash_match;
        
        -- Update verification count and last verified timestamp
        UPDATE source_integrity_records
        SET 
            verification_count = verification_count + 1,
            last_verified_at = NOW(),
            integrity_status = CASE 
                WHEN v_hash_match THEN 'verified'
                ELSE 'tampered'
            END
        WHERE id = v_stored_record.id;
    END IF;
    
    -- Record verification attempt
    INSERT INTO source_integrity_verifications (
        source_integrity_id, verification_type, verification_result,
        calculated_hash, stored_hash, hash_match, tampering_detected
    ) VALUES (
        COALESCE(v_stored_record.id, (
            SELECT id FROM source_integrity_records 
            WHERE source_id = p_source_id AND source_type = p_source_type
        )),
        'on_access', 
        CASE 
            WHEN v_hash_match THEN 'passed'
            ELSE 'failed'
        END,
        v_calculated_hash,
        COALESCE(v_stored_record.content_hash, v_calculated_hash),
        v_hash_match,
        v_tampering_detected
    );
    
    -- Create alert if tampering detected
    IF v_tampering_detected THEN
        INSERT INTO source_integrity_alerts (
            source_integrity_id, alert_type, severity, alert_message
        ) VALUES (
            v_stored_record.id,
            'tampering_detected',
            'high',
            'Source content hash mismatch detected - possible tampering'
        );
    END IF;
    
    -- Build result
    v_result := jsonb_build_object(
        'verified', v_hash_match,
        'tampering_detected', v_tampering_detected,
        'calculated_hash', v_calculated_hash,
        'stored_hash', COALESCE(v_stored_record.content_hash, v_calculated_hash),
        'integrity_status', CASE 
            WHEN v_hash_match THEN 'verified'
            ELSE 'tampered'
        END,
        'verification_timestamp', NOW()
    );
    
    RETURN v_result;
END;
$$ LANGUAGE plpgsql;

-- Function to get source integrity statistics
CREATE OR REPLACE FUNCTION get_source_integrity_statistics(
    p_time_range_start TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_time_range_end TIMESTAMP WITH TIME ZONE DEFAULT NULL
)
RETURNS JSONB AS $$
DECLARE
    v_result JSONB;
    v_time_filter TEXT := '';
BEGIN
    -- Build time filter if provided
    IF p_time_range_start IS NOT NULL AND p_time_range_end IS NOT NULL THEN
        v_time_filter := 'WHERE created_at BETWEEN $1 AND $2';
    ELSIF p_time_range_start IS NOT NULL THEN
        v_time_filter := 'WHERE created_at >= $1';
    ELSIF p_time_range_end IS NOT NULL THEN
        v_time_filter := 'WHERE created_at <= $1';
    END IF;
    
    -- Get statistics
    EXECUTE format('
        SELECT jsonb_build_object(
            ''total_sources'', COUNT(*),
            ''verified_sources'', COUNT(*) FILTER (WHERE integrity_status = ''verified''),
            ''tampered_sources'', COUNT(*) FILTER (WHERE integrity_status = ''tampered''),
            ''unknown_sources'', COUNT(*) FILTER (WHERE integrity_status = ''unknown''),
            ''pending_sources'', COUNT(*) FILTER (WHERE integrity_status = ''pending''),
            ''total_verifications'', SUM(verification_count),
            ''avg_verification_count'', AVG(verification_count),
            ''last_verification'', MAX(last_verified_at)
        )
        FROM source_integrity_records %s
    ', v_time_filter)
    INTO v_result
    USING p_time_range_start, p_time_range_end;
    
    RETURN v_result;
END;
$$ LANGUAGE plpgsql;
-- Add contract payload column to verdicts for interoperability tracking
ALTER TABLE verdicts
    ADD COLUMN IF NOT EXISTS contract JSONB NOT NULL DEFAULT '{}'::jsonb;

-- Ensure existing rows (if any) have deterministic default values
UPDATE verdicts
SET contract = json_build_object(
        'decision', decision,
        'votes', votes,
        'dissent', dissent,
        'remediation', remediation,
        'constitutional_refs', constitutional_refs
    )
WHERE contract = '{}'::jsonb;
-- Multimodal RAG Schema - Phase 1
-- Adds document ingest, segmentation, embedding, and search audit infrastructure

-- Documents (root for all ingested media)
CREATE TABLE IF NOT EXISTS documents (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  uri TEXT NOT NULL,
  sha256 TEXT NOT NULL UNIQUE,
  kind VARCHAR(50) NOT NULL,
  created_at TIMESTAMP DEFAULT now(),
  project_scope VARCHAR(255),
  version INTEGER DEFAULT 1,
  pipeline_version TEXT,
  toolchain TEXT,
  model_artifacts JSONB,
  CONSTRAINT valid_kind CHECK (kind IN ('video','slides','diagram','transcript'))
);
CREATE INDEX idx_documents_project ON documents(project_scope);
CREATE INDEX idx_documents_sha256 ON documents(sha256);

-- Segments (time/space slices within documents)
CREATE TABLE IF NOT EXISTS segments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  type VARCHAR(50) NOT NULL,
  t0 FLOAT,
  t1 FLOAT,
  bbox JSONB,
  content_hash TEXT,
  quality_score FLOAT,
  stability_score FLOAT,
  CONSTRAINT valid_segment_type CHECK (type IN ('slide','speech','diagram','scene'))
);
CREATE INDEX idx_segments_doc ON segments(doc_id);
CREATE INDEX idx_segments_type ON segments(type);

-- Blocks (semantic units within segments)
CREATE TABLE IF NOT EXISTS blocks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  role VARCHAR(50) NOT NULL,
  text TEXT,
  bbox JSONB,
  ocr_confidence FLOAT,
  CONSTRAINT valid_role CHECK (role IN ('title','bullet','code','table','figure','caption'))
);
CREATE INDEX idx_blocks_segment ON blocks(segment_id);

-- Embedding model registry (config-driven dimensions & metrics)
CREATE TABLE IF NOT EXISTS embedding_models (
  id TEXT PRIMARY KEY,
  modality TEXT NOT NULL,
  dim INTEGER NOT NULL,
  metric TEXT NOT NULL DEFAULT 'cosine',
  active BOOLEAN DEFAULT TRUE,
  CONSTRAINT valid_modality CHECK (modality IN ('text','image','audio')),
  CONSTRAINT valid_metric CHECK (metric IN ('cosine','ip','l2'))
);

-- Per-block vectors (one row per block-model pair)
CREATE TABLE IF NOT EXISTS block_vectors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
  model_id TEXT NOT NULL REFERENCES embedding_models(id),
  modality TEXT NOT NULL,
  vec VECTOR,
  CONSTRAINT valid_vec_modality CHECK (modality IN ('text','image','audio')),
  UNIQUE(block_id, model_id)
);

-- Speech turns (aligned with document timestamps)
CREATE TABLE IF NOT EXISTS speech_turns (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  speaker_id VARCHAR(255),
  provider TEXT,
  t0 FLOAT NOT NULL,
  t1 FLOAT NOT NULL,
  text TEXT NOT NULL,
  confidence FLOAT
);
CREATE INDEX idx_speech_turns_doc ON speech_turns(doc_id);
CREATE INDEX idx_speech_turns_time ON speech_turns(t0, t1);

-- Speech word timings (fine-grained temporal anchors)
CREATE TABLE IF NOT EXISTS speech_words (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  turn_id UUID NOT NULL REFERENCES speech_turns(id) ON DELETE CASCADE,
  t0 FLOAT NOT NULL,
  t1 FLOAT NOT NULL,
  token TEXT NOT NULL
);
CREATE INDEX idx_speech_words_turn ON speech_words(turn_id);

-- Diagram entities (graph nodes/edges/labels)
CREATE TABLE IF NOT EXISTS diagram_entities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  entity_type VARCHAR(50) NOT NULL,
  normalized_name TEXT,
  attributes JSONB,
  embedding_model_id TEXT REFERENCES embedding_models(id),
  embedding VECTOR,
  CONSTRAINT valid_entity_type CHECK (entity_type IN ('node','edge','label'))
);

CREATE TABLE IF NOT EXISTS diagram_edges (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  src UUID NOT NULL REFERENCES diagram_entities(id) ON DELETE CASCADE,
  dst UUID NOT NULL REFERENCES diagram_entities(id) ON DELETE CASCADE,
  label TEXT
);

-- Named entities (PII-aware)
CREATE TABLE IF NOT EXISTS entities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  type TEXT NOT NULL,
  norm TEXT NOT NULL,
  span_ref TEXT,
  pii BOOLEAN DEFAULT FALSE,
  hash TEXT
);

-- Provenance (fine-grained source tracking)
CREATE TABLE IF NOT EXISTS provenance (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source_uri TEXT NOT NULL,
  sha256 TEXT,
  t0 FLOAT,
  t1 FLOAT,
  spatial_ref JSONB,
  content_ref TEXT,
  accessed_at TIMESTAMP DEFAULT now()
);

-- Search audit logs (ranking, fusion, feature traces)
CREATE TABLE IF NOT EXISTS search_logs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  query TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT now(),
  results JSONB,
  features JSONB
);

-- Integrity constraints via triggers
CREATE OR REPLACE FUNCTION validate_segment_time_inclusion() RETURNS TRIGGER AS $$
DECLARE
  segment_record RECORD;
BEGIN
  -- Get the parent segment
  SELECT * INTO segment_record FROM segments WHERE id = NEW.segment_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'Segment % does not exist', NEW.segment_id;
  END IF;

  -- Validate time bounds if segment has temporal data
  IF segment_record.t0 IS NOT NULL AND segment_record.t1 IS NOT NULL THEN
    -- Block timestamps must be within segment bounds
    IF NEW.t0 IS NOT NULL AND (NEW.t0 < segment_record.t0 OR NEW.t0 > segment_record.t1) THEN
      RAISE EXCEPTION 'Block start time % is outside segment time bounds [%s, %s]',
        NEW.t0, segment_record.t0, segment_record.t1;
    END IF;

    IF NEW.t1 IS NOT NULL AND (NEW.t1 < segment_record.t0 OR NEW.t1 > segment_record.t1) THEN
      RAISE EXCEPTION 'Block end time % is outside segment time bounds [%s, %s]',
        NEW.t1, segment_record.t0, segment_record.t1;
    END IF;

    -- Ensure block time ordering
    IF NEW.t0 IS NOT NULL AND NEW.t1 IS NOT NULL AND NEW.t0 >= NEW.t1 THEN
      RAISE EXCEPTION 'Block start time % must be before end time %', NEW.t0, NEW.t1;
    END IF;
  END IF;

  -- Validate bbox consistency if both segment and block have bbox
  IF segment_record.bbox IS NOT NULL AND NEW.bbox IS NOT NULL THEN
    -- TODO: Implement comprehensive spatial relationship validation for multimodal content
    -- - [ ] Support different geometric containment types (fully contained, overlapping, adjacent)
    -- - [ ] Implement multi-dimensional bbox validation (2D, 3D, temporal)
    -- - [ ] Add spatial relationship types (contains, intersects, touches, within)
    -- - [ ] Support different coordinate systems and transformations
    -- - [ ] Implement spatial indexing and query optimization
    -- - [ ] Add spatial relationship consistency checking across modalities
    -- - [ ] Support spatial constraint validation and error reporting
    IF NOT validate_bbox_containment(segment_record.bbox, NEW.bbox) THEN
      RAISE WARNING 'Block bbox may extend outside segment bbox';
    END IF;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- TODO: Implement comprehensive spatial geometry validation functions
-- - [ ] Support complex geometric shapes beyond rectangles (polygons, circles, irregular shapes)
-- - [ ] Implement proper spatial reference systems and coordinate transformations
-- - [ ] Add geometric operations (intersection, union, difference, buffer)
-- - [ ] Support different dimensionality (2D, 3D, 4D with time)
-- - [ ] Implement spatial indexing and query optimization
-- - [ ] Add geometric validation and topological consistency checks
-- - [ ] Support different geometric data formats (WKT, GeoJSON, PostGIS)
CREATE OR REPLACE FUNCTION validate_bbox_containment(parent_bbox JSONB, child_bbox JSONB)
RETURNS BOOLEAN AS $$
DECLARE
  parent_x1 FLOAT;
  parent_y1 FLOAT;
  parent_x2 FLOAT;
  parent_y2 FLOAT;
  child_x1 FLOAT;
  child_y1 FLOAT;
  child_x2 FLOAT;
  child_y2 FLOAT;
BEGIN
  -- Extract bbox coordinates (assuming [x1,y1,x2,y2] format)
  parent_x1 := (parent_bbox->>0)::FLOAT;
  parent_y1 := (parent_bbox->>1)::FLOAT;
  parent_x2 := (parent_bbox->>2)::FLOAT;
  parent_y2 := (parent_bbox->>3)::FLOAT;

  child_x1 := (child_bbox->>0)::FLOAT;
  child_y1 := (child_bbox->>1)::FLOAT;
  child_x2 := (child_bbox->>2)::FLOAT;
  child_y2 := (child_bbox->>3)::FLOAT;

  -- Check if child bbox is contained within parent bbox
  RETURN child_x1 >= parent_x1 AND child_y1 >= parent_y1 AND
         child_x2 <= parent_x2 AND child_y2 <= parent_y2;
EXCEPTION
  WHEN OTHERS THEN
    -- Return true on parsing errors to avoid blocking inserts
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Validate speech turn temporal ordering
CREATE OR REPLACE FUNCTION validate_speech_turn_timing() RETURNS TRIGGER AS $$
BEGIN
  -- Ensure speech turn has valid time ordering
  IF NEW.t0 >= NEW.t1 THEN
    RAISE EXCEPTION 'Speech turn start time % must be before end time %', NEW.t0, NEW.t1;
  END IF;

  -- Ensure reasonable duration (not too long for a single turn)
  IF NEW.t1 - NEW.t0 > 300.0 THEN -- 5 minutes max
    RAISE WARNING 'Speech turn duration %.1f seconds seems unusually long', NEW.t1 - NEW.t0;
  END IF;

  -- Ensure non-negative timing
  IF NEW.t0 < 0 OR NEW.t1 < 0 THEN
    RAISE EXCEPTION 'Speech turn timing cannot be negative: [%, %]', NEW.t0, NEW.t1;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate vector dimensions match embedding model
CREATE OR REPLACE FUNCTION validate_vector_dimensions() RETURNS TRIGGER AS $$
DECLARE
  expected_dim INTEGER;
BEGIN
  -- Get expected dimension from embedding model
  SELECT dim INTO expected_dim
  FROM embedding_models
  WHERE id = NEW.model_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'Embedding model % does not exist', NEW.model_id;
  END IF;

  -- Validate vector dimension matches model expectation
  IF array_length(NEW.vec, 1) != expected_dim THEN
    RAISE EXCEPTION 'Vector dimension % does not match embedding model % dimension %',
      array_length(NEW.vec, 1), NEW.model_id, expected_dim;
  END IF;

  -- Validate modality consistency
  IF NEW.modality NOT IN ('text', 'image', 'audio') THEN
    RAISE EXCEPTION 'Invalid modality %', NEW.modality;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate diagram entity relationships
CREATE OR REPLACE FUNCTION validate_diagram_entity_relationships() RETURNS TRIGGER AS $$
DECLARE
  segment_kind VARCHAR(50);
BEGIN
  -- Ensure diagram entities belong to diagram segments
  SELECT s.type INTO segment_kind
  FROM segments s
  WHERE s.id = NEW.segment_id;

  IF segment_kind != 'diagram' THEN
    RAISE EXCEPTION 'Diagram entities can only belong to diagram segments, not %', segment_kind;
  END IF;

  -- Validate entity type
  IF NEW.entity_type NOT IN ('node', 'edge', 'label') THEN
    RAISE EXCEPTION 'Invalid diagram entity type %', NEW.entity_type;
  END IF;

  -- Validate embedding consistency if provided
  IF NEW.embedding IS NOT NULL AND NEW.embedding_model_id IS NOT NULL THEN
    -- Check that embedding dimension matches model
    DECLARE
      expected_dim INTEGER;
    BEGIN
      SELECT dim INTO expected_dim
      FROM embedding_models
      WHERE id = NEW.embedding_model_id;

      IF array_length(NEW.embedding, 1) != expected_dim THEN
        RAISE EXCEPTION 'Diagram entity embedding dimension % does not match model % dimension %',
          array_length(NEW.embedding, 1), NEW.embedding_model_id, expected_dim;
      END IF;
    END;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate document uniqueness constraints
CREATE OR REPLACE FUNCTION validate_document_uniqueness() RETURNS TRIGGER AS $$
DECLARE
  existing_count INTEGER;
BEGIN
  -- Check for duplicate URI + SHA256 combinations
  SELECT COUNT(*) INTO existing_count
  FROM documents
  WHERE uri = NEW.uri AND sha256 = NEW.sha256 AND id != COALESCE(NEW.id, '00000000-0000-0000-0000-000000000000'::UUID);

  IF existing_count > 0 THEN
    RAISE EXCEPTION 'Document with URI % and SHA256 % already exists', NEW.uri, NEW.sha256;
  END IF;

  -- Validate SHA256 format (64 hex characters)
  IF NEW.sha256 !~ '^[a-f0-9]{64}$' THEN
    RAISE EXCEPTION 'Invalid SHA256 format: %', NEW.sha256;
  END IF;

  -- Validate document kind
  IF NEW.kind NOT IN ('video', 'slides', 'diagram', 'transcript') THEN
    RAISE EXCEPTION 'Invalid document kind %', NEW.kind;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_validate_segment_time
  BEFORE INSERT OR UPDATE ON blocks
  FOR EACH ROW EXECUTE FUNCTION validate_segment_time_inclusion();

CREATE TRIGGER trg_validate_speech_turn_timing
  BEFORE INSERT OR UPDATE ON speech_turns
  FOR EACH ROW EXECUTE FUNCTION validate_speech_turn_timing();

CREATE TRIGGER trg_validate_vector_dimensions
  BEFORE INSERT OR UPDATE ON block_vectors
  FOR EACH ROW EXECUTE FUNCTION validate_vector_dimensions();

CREATE TRIGGER trg_validate_diagram_entity_relationships
  BEFORE INSERT OR UPDATE ON diagram_entities
  FOR EACH ROW EXECUTE FUNCTION validate_diagram_entity_relationships();

CREATE TRIGGER trg_validate_document_uniqueness
  BEFORE INSERT OR UPDATE ON documents
  FOR EACH ROW EXECUTE FUNCTION validate_document_uniqueness();

-- Bootstrap default embedding models (can be extended via config)
INSERT INTO embedding_models (id, modality, dim, metric, active)
VALUES 
  ('e5-small-v2', 'text', 1536, 'cosine', TRUE),
  ('clip-vit-b32', 'image', 512, 'cosine', TRUE)
ON CONFLICT (id) DO NOTHING;
-- @darianrosebrook
-- Migration: Enable pgvector and create HNSW indices for multimodal RAG
-- Version: 007
-- Date: October 18, 2025
-- Description: Enable vector similarity search via pgvector extension with HNSW indices

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create HNSW index for e5-small-v2 embeddings (semantic text search)
-- Cosine similarity for normalized vectors
CREATE INDEX IF NOT EXISTS idx_block_vectors_e5_small_v2_hnsw
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id = 'e5-small-v2';

-- Create HNSW index for CLIP-ViT-B/32 embeddings (image/diagram search)
-- Inner product similarity for normalized CLIP embeddings
CREATE INDEX IF NOT EXISTS idx_block_vectors_clip_vit_b32_hnsw
  ON block_vectors USING hnsw (vec vector_ip_ops)
  WHERE model_id = 'clip-vit-b32';

-- Create HNSW index for multilingual embeddings (e5-multilingual-large)
-- Cosine similarity
CREATE INDEX IF NOT EXISTS idx_block_vectors_e5_multilingual_hnsw
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id = 'e5-multilingual-large';

-- Create HNSW index for all other models (default cosine similarity)
-- This catches any new models added to the registry
CREATE INDEX IF NOT EXISTS idx_block_vectors_generic_hnsw
  ON block_vectors USING hnsw (vec vector_cosine_ops)
  WHERE model_id NOT IN ('e5-small-v2', 'clip-vit-b32', 'e5-multilingual-large');

-- Create composite index for queries by model_id + modality (for filtering)
CREATE INDEX IF NOT EXISTS idx_block_vectors_model_modality
  ON block_vectors (model_id, modality);

-- Create index for project_scope filtering (row-level visibility)
CREATE INDEX IF NOT EXISTS idx_segments_project_scope
  ON segments (project_scope);

-- Create index for document lookup by sha256 (deduplication)
CREATE INDEX IF NOT EXISTS idx_documents_sha256
  ON documents (sha256);

-- Create index for search logs audit trail
CREATE INDEX IF NOT EXISTS idx_search_logs_created_at
  ON search_logs (created_at DESC);

-- Add HNSW index parameters via comment (documentation)
COMMENT ON INDEX idx_block_vectors_e5_small_v2_hnsw IS 
  'HNSW index for e5-small-v2 embeddings (768 dimensions, cosine similarity)';

COMMENT ON INDEX idx_block_vectors_clip_vit_b32_hnsw IS 
  'HNSW index for CLIP-ViT-B/32 embeddings (512 dimensions, inner product)';

-- Verify pgvector is enabled
SELECT 
  extname,
  extversion
FROM pg_extension
WHERE extname = 'vector';
-- Add CAWS violations table for compliance tracking
-- This table stores CAWS compliance violations detected by workers

CREATE TABLE IF NOT EXISTS caws_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    violation_code TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    description TEXT NOT NULL,
    file_path TEXT,
    line_number INTEGER,
    column_number INTEGER,
    rule_id TEXT NOT NULL,
    constitutional_reference TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'resolved', 'waived', 'dismissed')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Indexes for efficient querying
    INDEX idx_caws_violations_task_id (task_id),
    INDEX idx_caws_violations_rule_id (rule_id),
    INDEX idx_caws_violations_status (status),
    INDEX idx_caws_violations_created_at (created_at),
    INDEX idx_caws_violations_severity (severity),

    -- Foreign key constraint (assuming tasks table exists)
    CONSTRAINT fk_caws_violations_task_id
        FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,

    -- Ensure resolved_at is set when status is resolved
    CONSTRAINT chk_resolved_at_when_resolved
        CHECK ((status = 'resolved' AND resolved_at IS NOT NULL) OR status != 'resolved')
);

-- Add comments for documentation
COMMENT ON TABLE caws_violations IS 'Stores CAWS compliance violations detected by workers for audit and tracking purposes';
COMMENT ON COLUMN caws_violations.violation_code IS 'The specific CAWS rule that was violated';
COMMENT ON COLUMN caws_violations.constitutional_reference IS 'Reference to constitutional document supporting the rule';
COMMENT ON COLUMN caws_violations.metadata IS 'Additional structured metadata about the violation';
-- Migration: External Knowledge Base Schema (Wikidata + WordNet)
-- Version: 009
-- Description: Add external knowledge entities with model-agnostic vectors, cross-references, and provenance
-- Author: @darianrosebrook
-- Date: 2025-10-19
--
-- This migration creates:
-- - External knowledge entities table (Wikidata + WordNet)
-- - Model-agnostic vector storage per entity
-- - Cross-reference relationships between knowledge sources
-- - Usage tracking and decay mechanisms
-- - Helper functions for knowledge queries

BEGIN;

-- ============================================================================
-- EMBEDDING MODEL REGISTRY (shared with multimodal if present)
-- ============================================================================

CREATE TABLE IF NOT EXISTS embedding_models (
  id TEXT PRIMARY KEY,           -- e.g., 'e5-small-v2', 'kb-text-default'
  modality TEXT NOT NULL,        -- 'text'|'image'|'audio'
  dim INTEGER NOT NULL,
  metric TEXT NOT NULL DEFAULT 'cosine',
  active BOOLEAN DEFAULT TRUE,
  CONSTRAINT valid_modality CHECK (modality IN ('text','image','audio')),
  CONSTRAINT valid_metric CHECK (metric IN ('cosine','ip','l2'))
);

-- Insert default KB text model
INSERT INTO embedding_models (id, modality, dim, metric, active)
VALUES ('kb-text-default', 'text', 768, 'cosine', true)
ON CONFLICT (id) DO NOTHING;

-- ============================================================================
-- EXTERNAL KNOWLEDGE ENTITIES
-- ============================================================================

CREATE TABLE external_knowledge_entities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source VARCHAR(20) NOT NULL CHECK (source IN ('wikidata','wordnet')),
  entity_key VARCHAR(255) NOT NULL,      -- QID/LID/synset key
  canonical_name TEXT NOT NULL,          -- surface form (by lang policy)
  lang VARCHAR(16),                      -- BCP47 tag, e.g., 'en'
  entity_type VARCHAR(50),               -- 'lexeme','item','synset',...
  properties JSONB NOT NULL,             -- source-specific structure
  confidence NUMERIC(3,2) DEFAULT 1.00,
  usage_count INTEGER DEFAULT 0,
  usage_decay FLOAT DEFAULT 1.0,         -- multiplicative decay factor
  last_accessed TIMESTAMPTZ,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  dump_version TEXT,                     -- e.g., 'wikidata-2025-09-24'
  toolchain TEXT,                        -- versions of parsers/embedders
  license TEXT,                          -- 'CC0', 'WordNet-3.1'
  CONSTRAINT valid_confidence CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CONSTRAINT valid_usage_decay CHECK (usage_decay >= 0.0)
);

-- Unique constraint on source + entity_key
CREATE UNIQUE INDEX uq_eke_source_key ON external_knowledge_entities(source, entity_key);

-- Indexes for performance
CREATE INDEX idx_eke_source ON external_knowledge_entities(source);
CREATE INDEX idx_eke_key ON external_knowledge_entities(entity_key);
CREATE INDEX idx_eke_canonical ON external_knowledge_entities(canonical_name);
CREATE INDEX idx_eke_lang ON external_knowledge_entities(lang) WHERE lang IS NOT NULL;
CREATE INDEX idx_eke_usage ON external_knowledge_entities(usage_count DESC);
CREATE INDEX idx_eke_decay ON external_knowledge_entities(usage_decay DESC);

-- Trigram index for fuzzy matching (requires pg_trgm extension)
CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX idx_eke_name_trgm ON external_knowledge_entities 
USING gin (canonical_name gin_trgm_ops);

-- ============================================================================
-- MODEL-AGNOSTIC VECTOR STORAGE
-- ============================================================================

CREATE TABLE knowledge_vectors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  entity_id UUID NOT NULL REFERENCES external_knowledge_entities(id) ON DELETE CASCADE,
  model_id TEXT NOT NULL REFERENCES embedding_models(id),
  vec VECTOR,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  UNIQUE(entity_id, model_id)
);

-- Index for entity lookup
CREATE INDEX idx_kv_entity ON knowledge_vectors(entity_id);
CREATE INDEX idx_kv_model ON knowledge_vectors(model_id);

-- HNSW index for default KB text model (cosine similarity)
-- Note: Additional HNSW indexes should be created per active model
CREATE INDEX idx_kv_kb_text_default_cos ON knowledge_vectors 
USING hnsw (vec vector_cosine_ops)
WITH (m = 16, ef_construction = 64)
WHERE model_id = 'kb-text-default' AND vec IS NOT NULL;

-- ============================================================================
-- CROSS-REFERENCE RELATIONSHIPS
-- ============================================================================

CREATE TABLE knowledge_relationships (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source_entity_id UUID NOT NULL REFERENCES external_knowledge_entities(id) ON DELETE CASCADE,
  target_entity_id UUID NOT NULL REFERENCES external_knowledge_entities(id) ON DELETE CASCADE,
  relationship_type VARCHAR(50) NOT NULL,    -- 'synonym','hypernym','translation','equivalent'
  confidence NUMERIC(3,2) DEFAULT 0.80,
  metadata JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW(),
  CONSTRAINT valid_rel_confidence CHECK (confidence >= 0.0 AND confidence <= 1.0),
  CONSTRAINT no_self_reference CHECK (source_entity_id != target_entity_id)
);

-- Indexes for relationship queries
CREATE INDEX idx_kr_src ON knowledge_relationships(source_entity_id);
CREATE INDEX idx_kr_tgt ON knowledge_relationships(target_entity_id);
CREATE INDEX idx_kr_type ON knowledge_relationships(relationship_type);
CREATE INDEX idx_kr_confidence ON knowledge_relationships(confidence DESC);

-- Index for bidirectional lookups
CREATE INDEX idx_kr_both ON knowledge_relationships(source_entity_id, target_entity_id);

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Record usage with decay boost
CREATE OR REPLACE FUNCTION record_knowledge_usage(p_entity_id UUID) 
RETURNS VOID AS $$
BEGIN
  UPDATE external_knowledge_entities
     SET usage_count = usage_count + 1,
         usage_decay = LEAST(usage_decay * 1.02, 10.0),
         last_accessed = NOW()
   WHERE id = p_entity_id;
END;
$$ LANGUAGE plpgsql;

-- Apply decay to stale entities (run periodically)
CREATE OR REPLACE FUNCTION apply_knowledge_decay(p_decay_factor FLOAT DEFAULT 0.95)
RETURNS INTEGER AS $$
DECLARE
  affected_count INTEGER;
BEGIN
  UPDATE external_knowledge_entities
     SET usage_decay = usage_decay * p_decay_factor
   WHERE last_accessed < NOW() - INTERVAL '30 days'
     AND usage_decay > 0.1;
  
  GET DIAGNOSTICS affected_count = ROW_COUNT;
  RETURN affected_count;
END;
$$ LANGUAGE plpgsql;

-- Semantic search for external knowledge
CREATE OR REPLACE FUNCTION kb_semantic_search(
  p_query_vec VECTOR,
  p_model_id TEXT,
  p_source VARCHAR(20) DEFAULT NULL,
  p_limit INTEGER DEFAULT 10,
  p_min_confidence NUMERIC DEFAULT 0.5
)
RETURNS TABLE(
  entity_id UUID,
  source VARCHAR(20),
  entity_key VARCHAR(255),
  canonical_name TEXT,
  entity_type VARCHAR(50),
  properties JSONB,
  similarity FLOAT,
  confidence NUMERIC,
  usage_count INTEGER
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    e.id as entity_id,
    e.source,
    e.entity_key,
    e.canonical_name,
    e.entity_type,
    e.properties,
    (1 - (v.vec <=> p_query_vec))::FLOAT as similarity,
    e.confidence,
    e.usage_count
  FROM knowledge_vectors v
  JOIN external_knowledge_entities e ON v.entity_id = e.id
  WHERE v.model_id = p_model_id
    AND v.vec IS NOT NULL
    AND (p_source IS NULL OR e.source = p_source)
    AND e.confidence >= p_min_confidence
  ORDER BY v.vec <=> p_query_vec
  LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Get related entities via relationships (with depth limit)
CREATE OR REPLACE FUNCTION kb_get_related(
  p_entity_id UUID,
  p_relationship_types VARCHAR(50)[] DEFAULT NULL,
  p_max_depth INTEGER DEFAULT 2
)
RETURNS TABLE(
  entity_id UUID,
  canonical_name TEXT,
  relationship_type VARCHAR(50),
  depth INTEGER,
  confidence NUMERIC
) AS $$
BEGIN
  RETURN QUERY
  WITH RECURSIVE related_entities AS (
    -- Base case: direct relationships
    SELECT 
      e.id as entity_id,
      e.canonical_name,
      r.relationship_type,
      1 as depth,
      r.confidence
    FROM knowledge_relationships r
    JOIN external_knowledge_entities e ON r.target_entity_id = e.id
    WHERE r.source_entity_id = p_entity_id
      AND (p_relationship_types IS NULL OR r.relationship_type = ANY(p_relationship_types))
    
    UNION
    
    -- Recursive case: follow relationships
    SELECT 
      e.id as entity_id,
      e.canonical_name,
      r.relationship_type,
      re.depth + 1 as depth,
      (re.confidence * r.confidence)::NUMERIC(3,2) as confidence
    FROM related_entities re
    JOIN knowledge_relationships r ON r.source_entity_id = re.entity_id
    JOIN external_knowledge_entities e ON r.target_entity_id = e.id
    WHERE re.depth < p_max_depth
      AND (p_relationship_types IS NULL OR r.relationship_type = ANY(p_relationship_types))
  )
  SELECT DISTINCT ON (entity_id)
    entity_id,
    canonical_name,
    relationship_type,
    depth,
    confidence
  FROM related_entities
  ORDER BY entity_id, depth ASC, confidence DESC;
END;
$$ LANGUAGE plpgsql STABLE;

-- Fuzzy search by canonical name
CREATE OR REPLACE FUNCTION kb_fuzzy_search(
  p_query TEXT,
  p_source VARCHAR(20) DEFAULT NULL,
  p_limit INTEGER DEFAULT 10,
  p_similarity_threshold FLOAT DEFAULT 0.3
)
RETURNS TABLE(
  entity_id UUID,
  canonical_name TEXT,
  similarity FLOAT
) AS $$
BEGIN
  RETURN QUERY
  SELECT 
    e.id as entity_id,
    e.canonical_name,
    similarity(e.canonical_name, p_query) as similarity
  FROM external_knowledge_entities e
  WHERE (p_source IS NULL OR e.source = p_source)
    AND similarity(e.canonical_name, p_query) > p_similarity_threshold
  ORDER BY similarity DESC
  LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Get knowledge statistics
CREATE OR REPLACE FUNCTION kb_get_stats()
RETURNS TABLE(
  source VARCHAR(20),
  total_entities BIGINT,
  total_vectors BIGINT,
  total_relationships BIGINT,
  avg_confidence NUMERIC,
  avg_usage_count NUMERIC,
  last_updated TIMESTAMPTZ
) AS $$
BEGIN
  RETURN QUERY
  SELECT
    e.source,
    COUNT(DISTINCT e.id) as total_entities,
    COUNT(DISTINCT v.id) as total_vectors,
    COUNT(DISTINCT r.id) as total_relationships,
    AVG(e.confidence) as avg_confidence,
    AVG(e.usage_count) as avg_usage_count,
    MAX(e.created_at) as last_updated
  FROM external_knowledge_entities e
  LEFT JOIN knowledge_vectors v ON v.entity_id = e.id
  LEFT JOIN knowledge_relationships r ON r.source_entity_id = e.id OR r.target_entity_id = e.id
  GROUP BY e.source;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE external_knowledge_entities IS 
'External knowledge sources (Wikidata lexemes/items and WordNet synsets) with provenance and usage tracking';

COMMENT ON TABLE knowledge_vectors IS 
'Model-agnostic vector embeddings for external knowledge entities, supporting multiple embedding models';

COMMENT ON TABLE knowledge_relationships IS 
'Cross-reference relationships between knowledge entities (synonyms, hypernyms, translations, equivalents)';

COMMENT ON FUNCTION record_knowledge_usage IS 
'Update usage statistics and boost decay factor for frequently accessed entities';

COMMENT ON FUNCTION apply_knowledge_decay IS 
'Apply decay to stale entities not accessed in 30+ days to manage storage';

COMMENT ON FUNCTION kb_semantic_search IS 
'Semantic similarity search over external knowledge using vector embeddings';

COMMENT ON FUNCTION kb_get_related IS 
'Retrieve related entities via relationship graph with configurable depth and relationship types';

COMMENT ON FUNCTION kb_fuzzy_search IS 
'Fuzzy text search over canonical names using trigram similarity';

COMMENT ON FUNCTION kb_get_stats IS 
'Get aggregate statistics for external knowledge sources';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Semantic search for "database" concept
-- SELECT * FROM kb_semantic_search('[0.1, 0.2, ...]'::vector, 'kb-text-default', NULL, 10, 0.6);

-- Example 2: Get WordNet synonyms and hypernyms
-- SELECT * FROM kb_get_related('uuid-here', ARRAY['synonym', 'hypernym'], 2);

-- Example 3: Fuzzy search for "databse" (typo)
-- SELECT * FROM kb_fuzzy_search('databse', 'wordnet', 5, 0.3);

-- Example 4: Record usage after successful disambiguation
-- SELECT record_knowledge_usage('uuid-here');

-- Example 5: Get knowledge base statistics
-- SELECT * FROM kb_get_stats();

-- Example 6: Apply decay to stale entities
-- SELECT apply_knowledge_decay(0.95);

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Additional HNSW indexes should be created for each active embedding model:
--    CREATE INDEX idx_kv_{model_id}_cos ON knowledge_vectors 
--    USING hnsw (vec vector_cosine_ops)
--    WHERE model_id = '{model_id}' AND vec IS NOT NULL;

-- 2. Schedule periodic decay application (e.g., via pg_cron):
--    SELECT cron.schedule('apply-kb-decay', '0 0 * * 0', 'SELECT apply_knowledge_decay(0.95);');

-- 3. Monitor index bloat and rebuild HNSW indexes periodically if needed:
--    REINDEX INDEX CONCURRENTLY idx_kv_kb_text_default_cos;

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

-- Migration 010: Database Integration Schema for V3 Components
-- Adds tables for CAWS validations, task resource history, and analytics cache
-- Author: @darianrosebrook
-- Date: 2025-10-19

BEGIN;

-- ============================================================================
-- CAWS VALIDATIONS TABLE
-- ============================================================================

CREATE TABLE caws_validations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    is_compliant BOOLEAN NOT NULL,
    violations JSONB NOT NULL DEFAULT '[]',
    suggestions JSONB NOT NULL DEFAULT '[]',
    trend VARCHAR(50) CHECK (trend IN ('improving', 'stable', 'declining', 'unknown')),
    validated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for CAWS validations
CREATE INDEX idx_caws_task_id ON caws_validations(task_id);
CREATE INDEX idx_caws_validated_at ON caws_validations(validated_at);
CREATE INDEX idx_caws_compliant ON caws_validations(is_compliant);
CREATE INDEX idx_caws_trend ON caws_validations(trend) WHERE trend IS NOT NULL;

-- ============================================================================
-- TASK RESOURCE HISTORY TABLE
-- ============================================================================

CREATE TABLE task_resource_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    task_type VARCHAR(100),
    cpu_usage_percent FLOAT NOT NULL CHECK (cpu_usage_percent >= 0.0 AND cpu_usage_percent <= 100.0),
    memory_usage_mb INTEGER NOT NULL CHECK (memory_usage_mb >= 0),
    execution_time_ms BIGINT NOT NULL CHECK (execution_time_ms >= 0),
    success BOOLEAN NOT NULL,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for task resource history
CREATE INDEX idx_task_history_task_id ON task_resource_history(task_id);
CREATE INDEX idx_task_history_type ON task_resource_history(task_type) WHERE task_type IS NOT NULL;
CREATE INDEX idx_task_history_recorded_at ON task_resource_history(recorded_at);
CREATE INDEX idx_task_history_success ON task_resource_history(success);
CREATE INDEX idx_task_history_cpu ON task_resource_history(cpu_usage_percent);
CREATE INDEX idx_task_history_memory ON task_resource_history(memory_usage_mb);

-- ============================================================================
-- ANALYTICS CACHE TABLE
-- ============================================================================

CREATE TABLE analytics_cache (
    cache_key VARCHAR(255) PRIMARY KEY,
    cache_value JSONB NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    access_count INTEGER DEFAULT 0 CHECK (access_count >= 0),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for analytics cache
CREATE INDEX idx_analytics_cache_expires ON analytics_cache(expires_at);
CREATE INDEX idx_analytics_cache_accessed ON analytics_cache(last_accessed_at);
CREATE INDEX idx_analytics_cache_access_count ON analytics_cache(access_count DESC);

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to store CAWS validation result
CREATE OR REPLACE FUNCTION store_caws_validation(
    p_task_id UUID,
    p_is_compliant BOOLEAN,
    p_violations JSONB DEFAULT '[]',
    p_suggestions JSONB DEFAULT '[]',
    p_trend VARCHAR(50) DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_validation_id UUID;
BEGIN
    INSERT INTO caws_validations (
        task_id, is_compliant, violations, suggestions, trend
    ) VALUES (
        p_task_id, p_is_compliant, p_violations, p_suggestions, p_trend
    )
    RETURNING id INTO v_validation_id;

    RETURN v_validation_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get CAWS validation history for a task
CREATE OR REPLACE FUNCTION get_caws_validation_history(
    p_task_id UUID,
    p_limit INTEGER DEFAULT 10
)
RETURNS TABLE(
    id UUID,
    is_compliant BOOLEAN,
    violations JSONB,
    suggestions JSONB,
    trend VARCHAR(50),
    validated_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        cv.id,
        cv.is_compliant,
        cv.violations,
        cv.suggestions,
        cv.trend,
        cv.validated_at
    FROM caws_validations cv
    WHERE cv.task_id = p_task_id
    ORDER BY cv.validated_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to record task resource usage
CREATE OR REPLACE FUNCTION record_task_resource_usage(
    p_task_id UUID,
    p_task_type VARCHAR(100) DEFAULT NULL,
    p_cpu_usage_percent FLOAT,
    p_memory_usage_mb INTEGER,
    p_execution_time_ms BIGINT,
    p_success BOOLEAN
)
RETURNS UUID AS $$
DECLARE
    v_record_id UUID;
BEGIN
    INSERT INTO task_resource_history (
        task_id, task_type, cpu_usage_percent, memory_usage_mb,
        execution_time_ms, success
    ) VALUES (
        p_task_id, p_task_type, p_cpu_usage_percent, p_memory_usage_mb,
        p_execution_time_ms, p_success
    )
    RETURNING id INTO v_record_id;

    RETURN v_record_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get historical resource data for task complexity estimation
CREATE OR REPLACE FUNCTION get_task_resource_history(
    p_task_id UUID DEFAULT NULL,
    p_task_type VARCHAR(100) DEFAULT NULL,
    p_limit INTEGER DEFAULT 50
)
RETURNS TABLE(
    task_id UUID,
    task_type VARCHAR(100),
    cpu_usage_percent FLOAT,
    memory_usage_mb INTEGER,
    execution_time_ms BIGINT,
    success BOOLEAN,
    recorded_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        trh.task_id,
        trh.task_type,
        trh.cpu_usage_percent,
        trh.memory_usage_mb,
        trh.execution_time_ms,
        trh.success,
        trh.recorded_at
    FROM task_resource_history trh
    WHERE (p_task_id IS NULL OR trh.task_id = p_task_id)
      AND (p_task_type IS NULL OR trh.task_type = p_task_type)
    ORDER BY trh.recorded_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to cache analytics data with automatic cleanup
CREATE OR REPLACE FUNCTION cache_analytics_data(
    p_cache_key VARCHAR(255),
    p_cache_value JSONB,
    p_ttl_seconds INTEGER DEFAULT 3600
)
RETURNS BOOLEAN AS $$
DECLARE
    v_expires_at TIMESTAMPTZ;
BEGIN
    v_expires_at := NOW() + INTERVAL '1 second' * p_ttl_seconds;

    INSERT INTO analytics_cache (
        cache_key, cache_value, expires_at
    ) VALUES (
        p_cache_key, p_cache_value, v_expires_at
    )
    ON CONFLICT (cache_key) DO UPDATE SET
        cache_value = EXCLUDED.cache_value,
        expires_at = EXCLUDED.expires_at,
        access_count = 0,
        last_accessed_at = NOW();

    RETURN TRUE;
EXCEPTION
    WHEN OTHERS THEN
        RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to get cached analytics data with access tracking
CREATE OR REPLACE FUNCTION get_analytics_cache(
    p_cache_key VARCHAR(255)
)
RETURNS JSONB AS $$
DECLARE
    v_cache_value JSONB;
BEGIN
    -- Get cache value and update access statistics
    UPDATE analytics_cache
    SET
        access_count = access_count + 1,
        last_accessed_at = NOW()
    WHERE cache_key = p_cache_key
      AND expires_at > NOW()
    RETURNING cache_value INTO v_cache_value;

    RETURN v_cache_value;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup expired cache entries
CREATE OR REPLACE FUNCTION cleanup_expired_cache()
RETURNS INTEGER AS $$
DECLARE
    v_deleted_count INTEGER;
BEGIN
    DELETE FROM analytics_cache
    WHERE expires_at <= NOW();

    GET DIAGNOSTICS v_deleted_count = ROW_COUNT;
    RETURN v_deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE caws_validations IS
'CAWS (Coding Agent Workflow System) validation results with compliance tracking and trend analysis';

COMMENT ON TABLE task_resource_history IS
'Historical resource usage data for tasks, used by learning components for performance prediction';

COMMENT ON TABLE analytics_cache IS
'Persistent cache for analytics dashboard data with LRU eviction and access tracking';

COMMENT ON FUNCTION store_caws_validation IS
'Store CAWS validation result with violations, suggestions, and trend analysis';

COMMENT ON FUNCTION get_caws_validation_history IS
'Retrieve CAWS validation history for a specific task with compliance trends';

COMMENT ON FUNCTION record_task_resource_usage IS
'Record resource usage data for task execution (CPU, memory, timing)';

COMMENT ON FUNCTION get_task_resource_history IS
'Retrieve historical resource data for task complexity and performance estimation';

COMMENT ON FUNCTION cache_analytics_data IS
'Cache analytics data with configurable TTL and automatic conflict resolution';

COMMENT ON FUNCTION get_analytics_cache IS
'Retrieve cached analytics data with automatic access tracking';

COMMENT ON FUNCTION cleanup_expired_cache IS
'Remove expired cache entries and return count of deleted items';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Store CAWS validation result
-- SELECT store_caws_validation('task-uuid', true, '[]'::jsonb, '[]'::jsonb, 'improving');

-- Example 2: Get CAWS validation history
-- SELECT * FROM get_caws_validation_history('task-uuid', 5);

-- Example 3: Record task resource usage
-- SELECT record_task_resource_usage('task-uuid', 'learning', 75.5, 512, 1500, true);

-- Example 4: Get task resource history
-- SELECT * FROM get_task_resource_history(NULL, 'learning', 20);

-- Example 5: Cache analytics data
-- SELECT cache_analytics_data('dashboard:metrics:hourly', '{"cpu": 85.2}'::jsonb, 3600);

-- Example 6: Get cached analytics data
-- SELECT get_analytics_cache('dashboard:metrics:hourly');

-- Example 7: Cleanup expired cache
-- SELECT cleanup_expired_cache();

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Schedule periodic cache cleanup (e.g., via pg_cron):
--    SELECT cron.schedule('cleanup-analytics-cache', '0 */6 * * *', 'SELECT cleanup_expired_cache();');

-- 2. Consider partitioning task_resource_history by month if data volume grows:
--    CREATE TABLE task_resource_history_y2025m10 PARTITION OF task_resource_history
--    FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');

-- 3. Monitor index performance and add composite indexes if needed:
--    CREATE INDEX CONCURRENTLY idx_task_history_composite
--    ON task_resource_history(task_type, success, recorded_at DESC);

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
-- Migration 011: Artifacts Storage Schema for Execution Artifacts
-- Adds tables for storing execution artifacts with metadata and versioning
-- Author: @darianrosebrook
-- Date: 2025-10-20

BEGIN;

-- ============================================================================
-- EXECUTION ARTIFACTS TABLE
-- ============================================================================

CREATE TABLE execution_artifacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    session_id UUID,
    execution_id UUID,
    artifact_type VARCHAR(50) NOT NULL CHECK (artifact_type IN ('unit_tests', 'integration_tests', 'e2e_tests', 'linting', 'coverage', 'profiling', 'logs', 'metrics')),
    artifact_data JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    size_bytes BIGINT NOT NULL CHECK (size_bytes >= 0),
    compression_type VARCHAR(20) DEFAULT 'none' CHECK (compression_type IN ('none', 'gzip', 'lz4', 'zstd')),
    checksum VARCHAR(128), -- SHA-256 checksum for integrity
    INDEX idx_execution_artifacts_task_id (task_id),
    INDEX idx_execution_artifacts_created_at (created_at),
    INDEX idx_execution_artifacts_expires_at (expires_at),
    INDEX idx_execution_artifacts_artifact_type (artifact_type)
);

-- ============================================================================
-- ARTIFACT METADATA TABLE
-- ============================================================================

CREATE TABLE artifact_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    execution_id UUID,
    session_id UUID,
    version INTEGER NOT NULL DEFAULT 1,
    artifact_types TEXT[] NOT NULL DEFAULT '{}', -- Array of artifact types
    total_size_bytes BIGINT NOT NULL DEFAULT 0 CHECK (total_size_bytes >= 0),
    compression_ratio REAL DEFAULT 1.0 CHECK (compression_ratio > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    retention_policy VARCHAR(50) DEFAULT 'standard' CHECK (retention_policy IN ('temporary', 'standard', 'permanent', 'audit')),
    metadata JSONB NOT NULL DEFAULT '{}',
    INDEX idx_artifact_metadata_task_id (task_id),
    INDEX idx_artifact_metadata_created_at (created_at),
    INDEX idx_artifact_metadata_expires_at (expires_at)
);

-- ============================================================================
-- ARTIFACT CLEANUP FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION cleanup_expired_artifacts()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER := 0;
BEGIN
    -- Delete expired execution artifacts
    WITH deleted AS (
        DELETE FROM execution_artifacts
        WHERE expires_at IS NOT NULL AND expires_at < NOW()
        RETURNING id
    )
    SELECT COUNT(*) INTO deleted_count FROM deleted;

    -- Delete expired metadata (cascading)
    DELETE FROM artifact_metadata
    WHERE expires_at IS NOT NULL AND expires_at < NOW();

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- ARTIFACT STATISTICS FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION get_artifact_storage_stats()
RETURNS TABLE (
    total_artifacts BIGINT,
    total_size_bytes BIGINT,
    average_size_bytes BIGINT,
    oldest_artifact TIMESTAMPTZ,
    newest_artifact TIMESTAMPTZ,
    expired_artifacts BIGINT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        (SELECT COUNT(*) FROM execution_artifacts) as total_artifacts,
        (SELECT COALESCE(SUM(size_bytes), 0) FROM execution_artifacts) as total_size_bytes,
        (SELECT COALESCE(AVG(size_bytes)::BIGINT, 0) FROM execution_artifacts) as average_size_bytes,
        (SELECT MIN(created_at) FROM execution_artifacts) as oldest_artifact,
        (SELECT MAX(created_at) FROM execution_artifacts) as newest_artifact,
        (SELECT COUNT(*) FROM execution_artifacts WHERE expires_at IS NOT NULL AND expires_at < NOW()) as expired_artifacts;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- ARTIFACT SEARCH FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION find_artifacts_by_task(
    p_task_id UUID,
    p_artifact_types TEXT[] DEFAULT NULL,
    p_limit INTEGER DEFAULT 50
)
RETURNS TABLE (
    artifact_id UUID,
    artifact_type VARCHAR(50),
    created_at TIMESTAMPTZ,
    size_bytes BIGINT,
    metadata JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        ea.id,
        ea.artifact_type,
        ea.created_at,
        ea.size_bytes,
        ea.metadata
    FROM execution_artifacts ea
    WHERE ea.task_id = p_task_id
    AND (p_artifact_types IS NULL OR ea.artifact_type = ANY(p_artifact_types))
    ORDER BY ea.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

CREATE INDEX CONCURRENTLY idx_execution_artifacts_composite
ON execution_artifacts (task_id, artifact_type, created_at DESC);

CREATE INDEX CONCURRENTLY idx_artifact_metadata_composite
ON artifact_metadata (task_id, created_at DESC, retention_policy);

-- ============================================================================
-- PARTITIONING (FOR LARGE SCALE)
-- ============================================================================

-- Create partitioning function for execution_artifacts by month
CREATE OR REPLACE FUNCTION execution_artifacts_partition_key()
RETURNS TRIGGER AS $$
BEGIN
    -- Partition by year-month for better query performance
    EXECUTE format('CREATE TABLE IF NOT EXISTS execution_artifacts_y%sm%s PARTITION OF execution_artifacts FOR VALUES FROM (''%s-%s-01'') TO (''%s-%s-01'')',
        EXTRACT(YEAR FROM NEW.created_at),
        LPAD(EXTRACT(MONTH FROM NEW.created_at)::TEXT, 2, '0'),
        EXTRACT(YEAR FROM NEW.created_at),
        LPAD(EXTRACT(MONTH FROM NEW.created_at)::TEXT, 2, '0'),
        EXTRACT(YEAR FROM NEW.created_at),
        LPAD(EXTRACT(MONTH FROM NEW.created_at + INTERVAL '1 month')::TEXT, 2, '0')
    );
    RETURN NEW;
EXCEPTION
    WHEN duplicate_table THEN
        RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic partitioning (optional, for high-scale deployments)
-- CREATE TRIGGER execution_artifacts_partition_trigger
--     BEFORE INSERT ON execution_artifacts
--     FOR EACH ROW EXECUTE FUNCTION execution_artifacts_partition_key();

COMMENT ON TABLE execution_artifacts IS 'Stores execution artifacts (test results, logs, metrics) with compression and integrity checks';
COMMENT ON TABLE artifact_metadata IS 'Metadata for artifact collections with versioning and retention policies';
COMMENT ON FUNCTION cleanup_expired_artifacts() IS 'Removes expired artifacts based on retention policies';
COMMENT ON FUNCTION get_artifact_storage_stats() IS 'Returns comprehensive storage statistics for artifacts';
COMMENT ON FUNCTION find_artifacts_by_task(UUID, TEXT[], INTEGER) IS 'Finds artifacts for a specific task with optional type filtering';

COMMIT;
-- Migration 012: Artifact Versioning Schema for Execution Artifacts
-- Adds versioning support for artifact storage with change tracking
-- Author: @darianrosebrook
-- Date: 2025-10-20

BEGIN;

-- ============================================================================
-- ARTIFACT VERSIONS TABLE
-- ============================================================================

CREATE TABLE artifact_versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    artifact_id UUID NOT NULL REFERENCES execution_artifacts(id) ON DELETE CASCADE,
    version_number INTEGER NOT NULL,
    version_label VARCHAR(100),
    parent_version_id UUID REFERENCES artifact_versions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    change_summary TEXT,
    change_type VARCHAR(50) CHECK (change_type IN ('created', 'modified', 'merged', 'branched')),
    compatibility_level VARCHAR(20) DEFAULT 'patch' CHECK (compatibility_level IN ('patch', 'minor', 'major', 'breaking')),
    metadata JSONB NOT NULL DEFAULT '{}',
    INDEX idx_artifact_versions_task_id (task_id),
    INDEX idx_artifact_versions_artifact_id (artifact_id),
    INDEX idx_artifact_versions_version_number (version_number),
    INDEX idx_artifact_versions_created_at (created_at)
);

-- ============================================================================
-- VERSION RELATIONSHIPS TABLE
-- ============================================================================

CREATE TABLE version_relationships (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    parent_version_id UUID NOT NULL REFERENCES artifact_versions(id) ON DELETE CASCADE,
    child_version_id UUID NOT NULL REFERENCES artifact_versions(id) ON DELETE CASCADE,
    relationship_type VARCHAR(50) NOT NULL CHECK (relationship_type IN ('derived_from', 'merged_into', 'branched_from', 'superseded_by')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(parent_version_id, child_version_id, relationship_type)
);

-- ============================================================================
-- ARTIFACT VERSION METADATA
-- ============================================================================

CREATE TABLE artifact_version_metadata (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    version_id UUID NOT NULL REFERENCES artifact_versions(id) ON DELETE CASCADE,
    key VARCHAR(255) NOT NULL,
    value JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(version_id, key)
);

-- ============================================================================
-- VERSIONING FUNCTIONS
-- ============================================================================

CREATE OR REPLACE FUNCTION get_latest_version(task_id_param UUID)
RETURNS TABLE (
    version_id UUID,
    artifact_id UUID,
    version_number INTEGER,
    version_label VARCHAR(100),
    created_at TIMESTAMPTZ,
    change_summary TEXT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        av.id,
        av.artifact_id,
        av.version_number,
        av.version_label,
        av.created_at,
        av.change_summary
    FROM artifact_versions av
    WHERE av.task_id = task_id_param
    ORDER BY av.version_number DESC
    LIMIT 1;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION get_version_history(task_id_param UUID, limit_param INTEGER DEFAULT 50)
RETURNS TABLE (
    version_id UUID,
    version_number INTEGER,
    version_label VARCHAR(100),
    created_at TIMESTAMPTZ,
    created_by VARCHAR(255),
    change_summary TEXT,
    change_type VARCHAR(50),
    compatibility_level VARCHAR(20)
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        av.id,
        av.version_number,
        av.version_label,
        av.created_at,
        av.created_by,
        av.change_summary,
        av.change_type,
        av.compatibility_level
    FROM artifact_versions av
    WHERE av.task_id = task_id_param
    ORDER BY av.version_number DESC
    LIMIT limit_param;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION create_new_version(
    task_id_param UUID,
    artifact_id_param UUID,
    change_summary_param TEXT DEFAULT NULL,
    change_type_param VARCHAR(50) DEFAULT 'modified',
    compatibility_level_param VARCHAR(20) DEFAULT 'patch',
    version_label_param VARCHAR(100) DEFAULT NULL,
    created_by_param VARCHAR(255) DEFAULT NULL
) RETURNS UUID AS $$
DECLARE
    new_version_id UUID;
    next_version_number INTEGER;
BEGIN
    -- Get next version number
    SELECT COALESCE(MAX(version_number), 0) + 1
    INTO next_version_number
    FROM artifact_versions
    WHERE task_id = task_id_param;

    -- Create new version
    INSERT INTO artifact_versions (
        task_id,
        artifact_id,
        version_number,
        version_label,
        change_summary,
        change_type,
        compatibility_level,
        created_by
    ) VALUES (
        task_id_param,
        artifact_id_param,
        next_version_number,
        version_label_param,
        change_summary_param,
        change_type_param,
        compatibility_level_param,
        created_by_param
    ) RETURNING id INTO new_version_id;

    RETURN new_version_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

CREATE INDEX CONCURRENTLY idx_version_relationships_parent
ON version_relationships (parent_version_id);

CREATE INDEX CONCURRENTLY idx_version_relationships_child
ON version_relationships (child_version_id);

CREATE INDEX CONCURRENTLY idx_artifact_version_metadata_version
ON artifact_version_metadata (version_id);

-- ============================================================================
-- TRIGGERS FOR AUTOMATIC VERSIONING
-- ============================================================================

CREATE OR REPLACE FUNCTION trigger_new_artifact_version()
RETURNS TRIGGER AS $$
BEGIN
    -- Automatically create version 1 for new artifacts
    PERFORM create_new_version(
        NEW.task_id,
        NEW.id,
        'Initial artifact creation',
        'created',
        'major',
        'v1.0.0',
        'system'
    );

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for automatic versioning (optional, for development)
-- CREATE TRIGGER artifact_versioning_trigger
--     AFTER INSERT ON execution_artifacts
--     FOR EACH ROW EXECUTE FUNCTION trigger_new_artifact_version();

COMMENT ON TABLE artifact_versions IS 'Version control for execution artifacts with change tracking';
COMMENT ON TABLE version_relationships IS 'Relationships between artifact versions (inheritance, merging)';
COMMENT ON TABLE artifact_version_metadata IS 'Additional metadata for artifact versions';
COMMENT ON FUNCTION get_latest_version(UUID) IS 'Get the latest version for a task';
COMMENT ON FUNCTION get_version_history(UUID, INTEGER) IS 'Get version history for a task with optional limit';
COMMENT ON FUNCTION create_new_version(UUID, UUID, TEXT, VARCHAR, VARCHAR, VARCHAR, VARCHAR) IS 'Create a new version for an artifact';

COMMIT;
-- Migration: Historical Claims Schema
-- Version: 013
-- Description: Add historical claims table for claim extraction and verification
-- Author: @darianrosebrook
-- Date: 2025-10-21
--
-- This migration creates:
-- - Historical claims table for storing verified claims
-- - Claim verification statuses and types
-- - Cross-references and metadata storage
-- - Performance indexes for claim lookups

BEGIN;

-- ============================================================================
-- CLAIM VERIFICATION STATUS ENUM
-- ============================================================================

CREATE TYPE claim_verification_status AS ENUM (
    'unverified',
    'pending_verification',
    'verified_true',
    'verified_false',
    'partially_verified',
    'conflicting_evidence',
    'needs_more_data'
);

-- ============================================================================
-- CLAIM TYPE ENUM
-- ============================================================================

CREATE TYPE claim_type AS ENUM (
    'factual_statement',
    'causal_relationship',
    'temporal_sequence',
    'quantitative_claim',
    'comparative_claim',
    'definitional_claim',
    'methodological_claim',
    'ethical_claim',
    'hypothetical_claim'
);

-- ============================================================================
-- HISTORICAL CLAIMS TABLE
-- ============================================================================

CREATE TABLE historical_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    claim_text TEXT NOT NULL,
    normalized_text TEXT, -- Normalized version for fuzzy matching
    claim_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash for deduplication

    -- Verification status
    verification_status claim_verification_status DEFAULT 'unverified',
    confidence_score NUMERIC(3,2), -- 0.00 to 1.00

    -- Source tracking
    source_count INTEGER DEFAULT 0,
    primary_source TEXT,
    source_references TEXT[], -- Array of source identifiers

    -- Temporal tracking
    first_seen_at TIMESTAMPTZ DEFAULT NOW(),
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Classification
    claim_type claim_type,
    domain_tags TEXT[], -- e.g., ['software', 'ai', 'testing']

    -- Relationships
    related_entities TEXT[], -- Entity names mentioned in claim
    cross_references UUID[], -- IDs of related claims
    parent_claim_id UUID REFERENCES historical_claims(id), -- For claim hierarchies

    -- Evidence and validation
    validation_metadata JSONB, -- Validation results, confidence factors
    evidence_summary JSONB, -- Summary of supporting/rejecting evidence

    -- Usage tracking
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT valid_confidence CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    CONSTRAINT valid_source_count CHECK (source_count >= 0),
    CONSTRAINT valid_access_count CHECK (access_count >= 0)
);

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- Primary lookup indexes
CREATE INDEX idx_historical_claims_hash ON historical_claims(claim_hash);
CREATE INDEX idx_historical_claims_text ON historical_claims USING gin(to_tsvector('english', claim_text));
CREATE INDEX idx_historical_claims_normalized ON historical_claims USING gin(to_tsvector('english', normalized_text)) WHERE normalized_text IS NOT NULL;
CREATE INDEX idx_historical_claims_status ON historical_claims(verification_status);
CREATE INDEX idx_historical_claims_type ON historical_claims(claim_type);
CREATE INDEX idx_historical_claims_confidence ON historical_claims(confidence_score DESC);

-- Temporal indexes
CREATE INDEX idx_historical_claims_created ON historical_claims(created_at DESC);
CREATE INDEX idx_historical_claims_verified ON historical_claims(last_verified_at DESC);
CREATE INDEX idx_historical_claims_accessed ON historical_claims(last_accessed_at DESC);

-- Relationship indexes
CREATE INDEX idx_historical_claims_parent ON historical_claims(parent_claim_id) WHERE parent_claim_id IS NOT NULL;
CREATE INDEX idx_historical_claims_entities ON historical_claims USING gin(related_entities) WHERE related_entities IS NOT NULL;
CREATE INDEX idx_historical_claims_tags ON historical_claims USING gin(domain_tags) WHERE domain_tags IS NOT NULL;
CREATE INDEX idx_historical_claims_references ON historical_claims USING gin(cross_references) WHERE cross_references IS NOT NULL;

-- ============================================================================
-- CLAIM SIMILARITY SEARCH FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION find_similar_claims(
    p_claim_text TEXT,
    p_similarity_threshold FLOAT DEFAULT 0.7,
    p_limit INTEGER DEFAULT 10,
    p_min_confidence NUMERIC DEFAULT 0.0
)
RETURNS TABLE(
    id UUID,
    claim_text TEXT,
    similarity_score FLOAT,
    confidence_score NUMERIC,
    verification_status claim_verification_status,
    claim_type claim_type
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        hc.id,
        hc.claim_text,
        (1 - (hc.normalized_text <=> p_claim_text))::FLOAT as similarity_score,
        hc.confidence_score,
        hc.verification_status,
        hc.claim_type
    FROM historical_claims hc
    WHERE hc.confidence_score >= p_min_confidence
        AND hc.normalized_text IS NOT NULL
        AND (1 - (hc.normalized_text <=> p_claim_text)) >= p_similarity_threshold
    ORDER BY (hc.normalized_text <=> p_claim_text)
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- CLAIM VERIFICATION UPDATE FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION update_claim_verification(
    p_claim_id UUID,
    p_new_status claim_verification_status,
    p_new_confidence NUMERIC DEFAULT NULL,
    p_validation_metadata JSONB DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    updated_rows INTEGER;
BEGIN
    UPDATE historical_claims
    SET
        verification_status = p_new_status,
        confidence_score = COALESCE(p_new_confidence, confidence_score),
        validation_metadata = COALESCE(p_validation_metadata, validation_metadata),
        last_verified_at = NOW(),
        updated_at = NOW()
    WHERE id = p_claim_id;

    GET DIAGNOSTICS updated_rows = ROW_COUNT;
    RETURN updated_rows > 0;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- CLAIM ACCESS TRACKING
-- ============================================================================

CREATE OR REPLACE FUNCTION record_claim_access(p_claim_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE historical_claims
    SET
        access_count = access_count + 1,
        last_accessed_at = NOW()
    WHERE id = p_claim_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE historical_claims IS
'Storage for historical claims with verification status, confidence scores, and cross-references';

COMMENT ON TYPE claim_verification_status IS
'Enumeration of possible claim verification states from unverified to fully verified';

COMMENT ON TYPE claim_type IS
'Classification of claim types for better organization and retrieval';

COMMENT ON FUNCTION find_similar_claims IS
'Find claims similar to input text using vector similarity search';

COMMENT ON FUNCTION update_claim_verification IS
'Update claim verification status with metadata tracking';

COMMENT ON FUNCTION record_claim_access IS
'Track claim usage for popularity and recency analysis';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Find similar claims
-- SELECT * FROM find_similar_claims('Machine learning models require large datasets', 0.6, 5);

-- Example 2: Update claim verification
-- SELECT update_claim_verification('uuid-here'::uuid, 'verified_true', 0.95);

-- Example 3: Record claim access
-- SELECT record_claim_access('uuid-here'::uuid);

-- Example 4: Get claims by verification status
-- SELECT * FROM historical_claims WHERE verification_status = 'verified_true' ORDER BY confidence_score DESC LIMIT 10;

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Consider adding trigram indexes for fuzzy text matching on claim_text
--    CREATE EXTENSION IF NOT EXISTS pg_trgm;
--    CREATE INDEX idx_claims_text_trgm ON historical_claims USING gin(claim_text gin_trgm_ops);

-- 2. For high-volume claim ingestion, consider partitioning by created_at:
--    CREATE TABLE historical_claims_y2025 PARTITION OF historical_claims FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

-- 3. Set up periodic cleanup of low-confidence, unverified claims:
--    DELETE FROM historical_claims WHERE verification_status = 'unverified' AND created_at < NOW() - INTERVAL '90 days';

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
-- Migration 014: Core P0 Persistence Schema
-- Adds foundational tables for task execution, audit trails, chat sessions, and saved queries
-- Author: @darianrosebrook
-- Date: 2025-10-20

BEGIN;

-- ============================================================================
-- TASKS TABLE - Core task execution state
-- ============================================================================

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spec JSONB NOT NULL,
    state VARCHAR(50) NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'executing', 'completed', 'failed', 'canceled', 'canceling', 'paused')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    metadata JSONB DEFAULT '{}'::jsonb,
    acceptance_criteria TEXT[] DEFAULT ARRAY[]::TEXT[]
);

CREATE INDEX idx_tasks_state ON tasks(state);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at DESC);
COMMENT ON TABLE tasks IS 'Core task execution records with state transitions';

-- ============================================================================
-- AUDIT LOGS TABLE - Decision and action audit trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(255) NOT NULL,
    actor VARCHAR(255),
    resource_id UUID,
    resource_type VARCHAR(50),
    change_summary JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
COMMENT ON TABLE audit_logs IS 'Immutable audit trail for all decisions, actions, and changes';

-- ============================================================================
-- CHAT SESSIONS TABLE - User chat session lifecycle
-- ============================================================================

CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_chat_sessions_created_at ON chat_sessions(created_at DESC);
CREATE INDEX idx_chat_sessions_ended_at ON chat_sessions(ended_at) WHERE ended_at IS NOT NULL;
COMMENT ON TABLE chat_sessions IS 'Chat session lifecycle management';

-- ============================================================================
-- CHAT MESSAGES TABLE - Individual messages in a session
-- ============================================================================

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at DESC);
COMMENT ON TABLE chat_messages IS 'Individual chat messages with role tracking';

-- ============================================================================
-- SAVED QUERIES TABLE - User-saved database queries
-- ============================================================================

CREATE TABLE IF NOT EXISTS saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    name VARCHAR(255) NOT NULL,
    query_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_saved_queries_user_id ON saved_queries(user_id);
CREATE INDEX idx_saved_queries_name ON saved_queries(name);
CREATE INDEX idx_saved_queries_created_at ON saved_queries(created_at DESC);
COMMENT ON TABLE saved_queries IS 'Saved database queries for dashboard exploration';

-- ============================================================================
-- WAIVERS TABLE - Quality gate bypass approvals
-- ============================================================================

CREATE TABLE IF NOT EXISTS waivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    reason VARCHAR(100) NOT NULL CHECK (reason IN (
        'emergency_hotfix', 'legacy_integration', 'experimental_feature',
        'third_party_constraint', 'performance_critical', 'security_patch',
        'infrastructure_limitation', 'other'
    )),
    description TEXT NOT NULL,
    gates TEXT[] NOT NULL, -- Array of quality gates being waived
    approved_by VARCHAR(255) NOT NULL,
    impact_level VARCHAR(20) NOT NULL CHECK (impact_level IN ('low', 'medium', 'high', 'critical')),
    mitigation_plan TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'revoked')),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_waivers_status ON waivers(status);
CREATE INDEX idx_waivers_expires_at ON waivers(expires_at);
CREATE INDEX idx_waivers_approved_by ON waivers(approved_by);
CREATE INDEX idx_waivers_impact_level ON waivers(impact_level);
CREATE INDEX idx_waivers_reason ON waivers(reason);
COMMENT ON TABLE waivers IS 'Quality gate waivers with approval workflow and expiration';

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to update task state and log audit event
CREATE OR REPLACE FUNCTION update_task_state(
    p_task_id UUID,
    p_new_state VARCHAR(50),
    p_actor VARCHAR(255) DEFAULT NULL,
    p_reason JSONB DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    v_old_state VARCHAR(50);
BEGIN
    -- Get current state
    SELECT state INTO v_old_state FROM tasks WHERE id = p_task_id;

    IF v_old_state IS NULL THEN
        RAISE EXCEPTION 'Task % not found', p_task_id;
    END IF;

    -- Update task state and timestamp
    UPDATE tasks
    SET state = p_new_state, updated_at = NOW()
    WHERE id = p_task_id;

    -- Log audit event
    INSERT INTO audit_logs (action, actor, resource_id, resource_type, change_summary)
    VALUES (
        'task_state_changed',
        COALESCE(p_actor, 'system'),
        p_task_id,
        'task',
        jsonb_build_object(
            'old_state', v_old_state,
            'new_state', p_new_state,
            'reason', p_reason
        )
    );

    RETURN TRUE;
EXCEPTION
    WHEN OTHERS THEN
        RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to log audit event
CREATE OR REPLACE FUNCTION log_audit_event(
    p_action VARCHAR(255),
    p_actor VARCHAR(255),
    p_resource_id UUID DEFAULT NULL,
    p_resource_type VARCHAR(50) DEFAULT NULL,
    p_change_summary JSONB DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
BEGIN
    INSERT INTO audit_logs (
        action, actor, resource_id, resource_type, change_summary
    ) VALUES (
        p_action,
        p_actor,
        p_resource_id,
        p_resource_type,
        COALESCE(p_change_summary, '{}'::jsonb)
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get audit trail for a resource
CREATE OR REPLACE FUNCTION get_audit_trail(
    p_resource_id UUID,
    p_limit INTEGER DEFAULT 50
)
RETURNS TABLE(
    id UUID,
    action VARCHAR(255),
    actor VARCHAR(255),
    change_summary JSONB,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        al.id,
        al.action,
        al.actor,
        al.change_summary,
        al.created_at
    FROM audit_logs al
    WHERE al.resource_id = p_resource_id
    ORDER BY al.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to retrieve chat history for a session
CREATE OR REPLACE FUNCTION get_chat_history(
    p_session_id UUID,
    p_limit INTEGER DEFAULT 100
)
RETURNS TABLE(
    id UUID,
    role VARCHAR(50),
    content TEXT,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        cm.id,
        cm.role,
        cm.content,
        cm.created_at
    FROM chat_messages cm
    WHERE cm.session_id = p_session_id
    ORDER BY cm.created_at ASC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON FUNCTION update_task_state(UUID, VARCHAR(50), VARCHAR(255), JSONB) IS
'Update task state and automatically log audit event with reason';

COMMENT ON FUNCTION log_audit_event(VARCHAR(255), VARCHAR(255), UUID, VARCHAR(50), JSONB) IS
'Log an audit event for any action or state change';

COMMENT ON FUNCTION get_audit_trail(UUID, INTEGER) IS
'Retrieve audit trail for a specific resource (task, waiver, etc.)';

COMMENT ON FUNCTION get_chat_history(UUID, INTEGER) IS
'Retrieve chat message history for a session in chronological order';

-- Function to create a waiver with validation
CREATE OR REPLACE FUNCTION create_waiver(
    p_title VARCHAR(255),
    p_reason VARCHAR(100),
    p_description TEXT,
    p_gates TEXT[],
    p_approved_by VARCHAR(255),
    p_impact_level VARCHAR(20),
    p_mitigation_plan TEXT,
    p_expires_at TIMESTAMPTZ,
    p_metadata JSONB DEFAULT '{}'::jsonb
)
RETURNS UUID AS $$
DECLARE
    v_waiver_id UUID;
BEGIN
    -- Validate reason
    IF p_reason NOT IN (
        'emergency_hotfix', 'legacy_integration', 'experimental_feature',
        'third_party_constraint', 'performance_critical', 'security_patch',
        'infrastructure_limitation', 'other'
    ) THEN
        RAISE EXCEPTION 'Invalid waiver reason: %', p_reason;
    END IF;
    
    -- Validate impact level
    IF p_impact_level NOT IN ('low', 'medium', 'high', 'critical') THEN
        RAISE EXCEPTION 'Invalid impact level: %', p_impact_level;
    END IF;
    
    -- Validate expiration is in the future
    IF p_expires_at <= NOW() THEN
        RAISE EXCEPTION 'Waiver expiration must be in the future';
    END IF;
    
    INSERT INTO waivers (
        title, reason, description, gates, approved_by, impact_level,
        mitigation_plan, expires_at, metadata
    )
    VALUES (
        p_title, p_reason, p_description, p_gates, p_approved_by, p_impact_level,
        p_mitigation_plan, p_expires_at, p_metadata
    )
    RETURNING id INTO v_waiver_id;
    
    -- Log the waiver creation
    PERFORM log_audit_event(
        'waiver_created',
        p_approved_by,
        v_waiver_id,
        'waiver',
        jsonb_build_object(
            'title', p_title,
            'reason', p_reason,
            'impact_level', p_impact_level,
            'expires_at', p_expires_at
        )
    );
    
    RETURN v_waiver_id;
END;
$$ LANGUAGE plpgsql;

-- Function to check if a waiver is active for specific gates
CREATE OR REPLACE FUNCTION is_waiver_active(
    p_gates TEXT[],
    p_check_time TIMESTAMPTZ DEFAULT NOW()
)
RETURNS BOOLEAN AS $$
DECLARE
    v_active_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO v_active_count
    FROM waivers
    WHERE status = 'active'
      AND expires_at > p_check_time
      AND gates && p_gates; -- Array overlap operator
    
    RETURN v_active_count > 0;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_waiver(VARCHAR(255), VARCHAR(100), TEXT, TEXT[], VARCHAR(255), VARCHAR(20), TEXT, TIMESTAMPTZ, JSONB) IS
'Create a new waiver with validation and automatic audit logging';

COMMENT ON FUNCTION is_waiver_active(TEXT[], TIMESTAMPTZ) IS
'Check if any active waivers exist for the specified gates';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Create a new task
-- INSERT INTO tasks (spec) VALUES ('{"title": "My Task", "description": "Test"}'::jsonb);

-- Example 2: Update task state with audit
-- SELECT update_task_state('task-uuid', 'executing', 'user@example.com', '{"reason": "Started by user"}'::jsonb);

-- Example 3: Get audit trail for a task
-- SELECT * FROM get_audit_trail('task-uuid', 10);

-- Example 4: Create a chat session
-- INSERT INTO chat_sessions DEFAULT VALUES RETURNING id;

-- Example 5: Add message to chat
-- INSERT INTO chat_messages (session_id, role, content) VALUES ('session-uuid', 'user', 'Hello');

-- Example 6: Get chat history
-- SELECT * FROM get_chat_history('session-uuid', 50);

-- Example 7: Save a query
-- INSERT INTO saved_queries (user_id, name, query_text) VALUES ('user-uuid', 'My Query', 'SELECT * FROM tasks WHERE state = ''completed''');

-- Example 8: Create a waiver for emergency hotfix
-- SELECT create_waiver(
--     'Emergency Security Patch',
--     'emergency_hotfix',
--     'Critical security vulnerability requires immediate deployment',
--     ARRAY['test_coverage', 'mutation_testing'],
--     'security-team@company.com',
--     'critical',
--     'Deploy with enhanced monitoring and immediate rollback plan',
--     NOW() + INTERVAL '24 hours'
-- );

-- Example 9: Check if waiver is active for specific gates
-- SELECT is_waiver_active(ARRAY['test_coverage', 'mutation_testing']);

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Consider partitioning audit_logs by month for large volumes:
--    CREATE TABLE audit_logs_y2025m10 PARTITION OF audit_logs
--    FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');

-- 2. Set up periodic cleanup of old chat sessions:
--    SELECT cron.schedule('cleanup-old-chats', '0 3 * * *',
--      'DELETE FROM chat_sessions WHERE ended_at < NOW() - INTERVAL ''30 days''');

-- 3. Monitor audit_logs growth and consider archival strategy:
--    CREATE TABLE audit_logs_archive PARTITION OF audit_logs
--    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
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
-- ============================================================================
-- PARALLEL WORKERS LEARNING SYSTEM MIGRATION
-- ============================================================================
-- Create comprehensive database schema for parallel workers learning system
-- This enables persistent learning, pattern recognition, and optimization
--
-- Tables created:
-- - learning_execution_records: Raw execution data for analysis
-- - learning_worker_profiles: Worker performance and specialization tracking
-- - learning_success_patterns: Identified successful execution patterns
-- - learning_failure_patterns: Identified failure patterns and causes
-- - learning_optimal_configs: Optimal configuration recommendations
-- - learning_config_recommendations: Pattern-specific configuration suggestions
-- - learning_optimization_events: Learning system optimization tracking
-- ============================================================================

-- ============================================================================
-- EXECUTION RECORDS TABLE
-- ============================================================================
-- Stores raw execution data for learning and analysis
CREATE TABLE IF NOT EXISTS learning_execution_records (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id             UUID NOT NULL,
    worker_id           UUID NOT NULL,
    specialty           TEXT NOT NULL, -- Worker specialty (enum serialized)
    subtask_id          UUID NOT NULL,
    success             BOOLEAN NOT NULL,
    execution_time_ms   BIGINT NOT NULL,
    quality_score       REAL,
    error_category      TEXT,
    task_pattern_type   TEXT NOT NULL, -- CompilationErrors, RefactoringOperations, etc.
    task_pattern_hash   BIGINT NOT NULL, -- Hash for pattern deduplication
    metrics_data        JSONB NOT NULL DEFAULT '{}'::jsonb, -- Full execution metrics
    outcome             TEXT NOT NULL CHECK (outcome IN ('Success', 'Failure', 'Timeout', 'Cancelled')),
    learning_mode       TEXT NOT NULL CHECK (learning_mode IN ('Learn', 'DoNotLearn', 'Shadow')),
    timestamp           TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT positive_execution_time CHECK (execution_time_ms > 0),
    CONSTRAINT valid_quality_score CHECK (quality_score IS NULL OR (quality_score >= 0.0 AND quality_score <= 1.0))
);

-- Indexes for efficient querying
CREATE INDEX idx_learning_execution_records_task_id ON learning_execution_records (task_id);
CREATE INDEX idx_learning_execution_records_worker_id ON learning_execution_records (worker_id);
CREATE INDEX idx_learning_execution_records_timestamp ON learning_execution_records (timestamp DESC);
CREATE INDEX idx_learning_execution_records_pattern ON learning_execution_records (task_pattern_type, task_pattern_hash);
CREATE INDEX idx_learning_execution_records_success ON learning_execution_records (success);
CREATE INDEX idx_learning_execution_records_outcome ON learning_execution_records (outcome);

-- ============================================================================
-- WORKER PERFORMANCE PROFILES TABLE
-- ============================================================================
-- Tracks worker performance and specialization over time
CREATE TABLE IF NOT EXISTS learning_worker_profiles (
    id                          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id                   UUID NOT NULL UNIQUE,
    specialty                   TEXT NOT NULL,
    total_executions            INTEGER NOT NULL DEFAULT 0,
    success_rate                REAL NOT NULL CHECK (success_rate >= 0.0 AND success_rate <= 1.0),
    average_execution_time_ms   BIGINT NOT NULL,
    average_quality_score       REAL CHECK (average_quality_score >= 0.0 AND average_quality_score <= 1.0),
    resource_efficiency         JSONB NOT NULL DEFAULT '{}'::jsonb,
    specialization_strength     JSONB NOT NULL DEFAULT '{}'::jsonb,
    last_updated                TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT positive_executions CHECK (total_executions >= 0),
    CONSTRAINT positive_avg_time CHECK (average_execution_time_ms > 0)
);

-- Indexes
CREATE UNIQUE INDEX idx_learning_worker_profiles_worker_id ON learning_worker_profiles (worker_id);
CREATE INDEX idx_learning_worker_profiles_specialty ON learning_worker_profiles (specialty);
CREATE INDEX idx_learning_worker_profiles_updated ON learning_worker_profiles (last_updated DESC);

-- ============================================================================
-- SUCCESS PATTERNS TABLE
-- ============================================================================
-- Stores identified successful execution patterns
CREATE TABLE IF NOT EXISTS learning_success_patterns (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_type        TEXT NOT NULL,
    pattern_hash        BIGINT NOT NULL,
    pattern_data        JSONB NOT NULL,
    confidence_score    REAL NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    execution_count     INTEGER NOT NULL DEFAULT 1,
    avg_improvement     REAL NOT NULL,
    last_seen           TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT positive_execution_count CHECK (execution_count > 0)
);

-- Indexes
CREATE UNIQUE INDEX idx_learning_success_patterns_unique ON learning_success_patterns (pattern_type, pattern_hash);
CREATE INDEX idx_learning_success_patterns_confidence ON learning_success_patterns (confidence_score DESC);
CREATE INDEX idx_learning_success_patterns_last_seen ON learning_success_patterns (last_seen DESC);

-- ============================================================================
-- FAILURE PATTERNS TABLE
-- ============================================================================
-- Stores identified failure patterns and root causes
CREATE TABLE IF NOT EXISTS learning_failure_patterns (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_type        TEXT NOT NULL,
    pattern_hash        BIGINT NOT NULL,
    pattern_data        JSONB NOT NULL,
    root_cause_category TEXT NOT NULL,
    root_cause_details  JSONB NOT NULL DEFAULT '{}'::jsonb,
    confidence_score    REAL NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    occurrence_count    INTEGER NOT NULL DEFAULT 1,
    avg_severity        REAL NOT NULL CHECK (avg_severity >= 0.0 AND avg_severity <= 1.0),
    mitigation_strategy JSONB,
    last_seen           TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT positive_occurrence_count CHECK (occurrence_count > 0)
);

-- Indexes
CREATE UNIQUE INDEX idx_learning_failure_patterns_unique ON learning_failure_patterns (pattern_type, pattern_hash);
CREATE INDEX idx_learning_failure_patterns_root_cause ON learning_failure_patterns (root_cause_category);
CREATE INDEX idx_learning_failure_patterns_confidence ON learning_failure_patterns (confidence_score DESC);
CREATE INDEX idx_learning_failure_patterns_last_seen ON learning_failure_patterns (last_seen DESC);

-- ============================================================================
-- OPTIMAL CONFIGURATIONS TABLE
-- ============================================================================
-- Stores optimal configuration settings for different scenarios
CREATE TABLE IF NOT EXISTS learning_optimal_configs (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_type        TEXT NOT NULL,
    pattern_hash        BIGINT NOT NULL,
    config_key          TEXT NOT NULL,
    config_value        JSONB NOT NULL,
    confidence_score    REAL NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    performance_impact  REAL NOT NULL,
    validation_count    INTEGER NOT NULL DEFAULT 1,
    last_validated      TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT positive_validation_count CHECK (validation_count > 0)
);

-- Indexes
CREATE UNIQUE INDEX idx_learning_optimal_configs_unique ON learning_optimal_configs (pattern_type, pattern_hash, config_key);
CREATE INDEX idx_learning_optimal_configs_confidence ON learning_optimal_configs (confidence_score DESC);
CREATE INDEX idx_learning_optimal_configs_impact ON learning_optimal_configs (performance_impact DESC);
CREATE INDEX idx_learning_optimal_configs_last_validated ON learning_optimal_configs (last_validated DESC);

-- ============================================================================
-- CONFIGURATION RECOMMENDATIONS TABLE
-- ============================================================================
-- Stores pattern-specific configuration recommendations
CREATE TABLE IF NOT EXISTS learning_config_recommendations (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    pattern_type        TEXT NOT NULL,
    pattern_hash        BIGINT NOT NULL,
    recommendations     JSONB NOT NULL,
    overall_confidence  REAL NOT NULL CHECK (overall_confidence >= 0.0 AND overall_confidence <= 1.0),
    expected_improvement REAL NOT NULL,
    validation_status   TEXT NOT NULL CHECK (validation_status IN ('proposed', 'validated', 'deprecated')),
    last_updated        TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT positive_expected_improvement CHECK (expected_improvement >= 0.0)
);

-- Indexes
CREATE UNIQUE INDEX idx_learning_config_recommendations_unique ON learning_config_recommendations (pattern_type, pattern_hash);
CREATE INDEX idx_learning_config_recommendations_confidence ON learning_config_recommendations (overall_confidence DESC);
CREATE INDEX idx_learning_config_recommendations_status ON learning_config_recommendations (validation_status);
CREATE INDEX idx_learning_config_recommendations_updated ON learning_config_recommendations (last_updated DESC);

-- ============================================================================
-- OPTIMIZATION EVENTS TABLE
-- ============================================================================
-- Tracks learning system optimization events and decisions
CREATE TABLE IF NOT EXISTS learning_optimization_events (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type          TEXT NOT NULL CHECK (event_type IN ('pattern_discovered', 'config_optimized', 'worker_reassigned', 'drift_detected', 'learning_updated')),
    event_data          JSONB NOT NULL,
    impact_score        REAL CHECK (impact_score >= -1.0 AND impact_score <= 1.0),
    affected_patterns   JSONB NOT NULL DEFAULT '[]'::jsonb,
    timestamp           TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- Constraints
    CONSTRAINT valid_impact_score CHECK (impact_score IS NULL OR (impact_score >= -1.0 AND impact_score <= 1.0))
);

-- Indexes
CREATE INDEX idx_learning_optimization_events_type ON learning_optimization_events (event_type);
CREATE INDEX idx_learning_optimization_events_timestamp ON learning_optimization_events (timestamp DESC);
CREATE INDEX idx_learning_optimization_events_impact ON learning_optimization_events (impact_score DESC);

-- ============================================================================
-- TABLE COMMENTS
-- ============================================================================

COMMENT ON TABLE learning_execution_records IS 'Raw execution data for learning system analysis and pattern recognition';
COMMENT ON TABLE learning_worker_profiles IS 'Worker performance profiles and specialization tracking';
COMMENT ON TABLE learning_success_patterns IS 'Identified successful execution patterns with confidence scores';
COMMENT ON TABLE learning_failure_patterns IS 'Identified failure patterns with root cause analysis';
COMMENT ON TABLE learning_optimal_configs IS 'Optimal configuration settings for different task patterns';
COMMENT ON TABLE learning_config_recommendations IS 'Pattern-specific configuration recommendations';
COMMENT ON TABLE learning_optimization_events IS 'Learning system optimization events and decisions';

-- ============================================================================
-- DATA RETENTION POLICIES
-- ============================================================================

-- Create function for cleanup of old learning data
CREATE OR REPLACE FUNCTION cleanup_learning_data(
    p_execution_retention_days INTEGER DEFAULT 90,
    p_pattern_retention_days INTEGER DEFAULT 180,
    p_event_retention_days INTEGER DEFAULT 30
)
RETURNS TABLE (
    executions_deleted BIGINT,
    patterns_deleted BIGINT,
    events_deleted BIGINT
)
LANGUAGE SQL
AS $$
    WITH execution_cleanup AS (
        DELETE FROM learning_execution_records
        WHERE timestamp < now() - INTERVAL '1 day' * p_execution_retention_days
        RETURNING id
    ),
    pattern_cleanup AS (
        DELETE FROM learning_success_patterns
        WHERE last_seen < now() - INTERVAL '1 day' * p_pattern_retention_days
        RETURNING id
    ),
    failure_pattern_cleanup AS (
        DELETE FROM learning_failure_patterns
        WHERE last_seen < now() - INTERVAL '1 day' * p_pattern_retention_days
        RETURNING id
    ),
    event_cleanup AS (
        DELETE FROM learning_optimization_events
        WHERE timestamp < now() - INTERVAL '1 day' * p_event_retention_days
        RETURNING id
    )
    SELECT
        (SELECT COUNT(*) FROM execution_cleanup) as executions_deleted,
        ((SELECT COUNT(*) FROM pattern_cleanup) + (SELECT COUNT(*) FROM failure_pattern_cleanup)) as patterns_deleted,
        (SELECT COUNT(*) FROM event_cleanup) as events_deleted;
$$;

COMMENT ON FUNCTION cleanup_learning_data(INTEGER, INTEGER, INTEGER) IS 'Clean up old learning data based on retention policies. Returns counts of deleted records.';

-- ============================================================================
-- ANALYTICS VIEWS
-- ============================================================================

-- View for worker performance analytics
CREATE OR REPLACE VIEW learning_worker_performance_analytics AS
SELECT
    wp.worker_id,
    wp.specialty,
    wp.total_executions,
    wp.success_rate,
    wp.average_execution_time_ms,
    wp.average_quality_score,
    COUNT(er.id) as recent_executions,
    AVG(er.execution_time_ms) as recent_avg_time,
    AVG(er.quality_score) as recent_avg_quality,
    MAX(er.timestamp) as last_execution
FROM learning_worker_profiles wp
LEFT JOIN learning_execution_records er ON wp.worker_id = er.worker_id
    AND er.timestamp > now() - INTERVAL '7 days'
GROUP BY wp.worker_id, wp.specialty, wp.total_executions, wp.success_rate,
         wp.average_execution_time_ms, wp.average_quality_score;

-- View for pattern effectiveness
CREATE OR REPLACE VIEW learning_pattern_effectiveness AS
SELECT
    sp.pattern_type,
    sp.pattern_hash,
    sp.confidence_score,
    sp.execution_count,
    sp.avg_improvement,
    fp.root_cause_category,
    fp.occurrence_count as failure_count,
    fp.avg_severity as failure_severity,
    (sp.execution_count::float / NULLIF(fp.occurrence_count, 0)) as success_to_failure_ratio
FROM learning_success_patterns sp
FULL OUTER JOIN learning_failure_patterns fp
    ON sp.pattern_type = fp.pattern_type AND sp.pattern_hash = fp.pattern_hash;

-- ============================================================================
-- PERFORMANCE & MONITORING
-- ============================================================================

-- Create partial indexes for common queries
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_learning_exec_recent
    ON learning_execution_records (timestamp DESC)
    WHERE timestamp > now() - INTERVAL '30 days';

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_learning_exec_success_recent
    ON learning_execution_records (success, timestamp DESC)
    WHERE timestamp > now() - INTERVAL '30 days';

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

-- Note: This migration establishes the complete learning system persistence layer.
-- The schema is designed to support:
-- - Scalable execution record storage with efficient querying
-- - Worker performance tracking and specialization analysis
-- - Pattern recognition and confidence scoring
-- - Configuration optimization and recommendation systems
-- - Learning system monitoring and analytics
--
-- Future migrations may add partitioning, additional indexes, or schema extensions
-- based on production usage patterns and performance requirements.
-- Performance optimization indexes for Agent Agency V3
-- Created: 2025-01-20
-- Purpose: Improve query performance based on observed usage patterns

-- Task status filtering (frequently used in dashboards and monitoring)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_status ON tasks (status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_risk_tier ON tasks (risk_tier);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_created_at ON tasks (created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_priority ON tasks (priority DESC) WHERE priority IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_assigned_worker ON tasks (assigned_worker_id) WHERE assigned_worker_id IS NOT NULL;

-- Task execution performance monitoring
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_task_id ON task_executions (task_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_worker_id ON task_executions (worker_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_status ON task_executions (status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_started_at ON task_executions (execution_started_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_completed_at ON task_executions (execution_completed_at DESC) WHERE execution_completed_at IS NOT NULL;

-- Council verdict lookups (critical for decision making)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_council_verdicts_task_id ON council_verdicts (task_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_council_verdicts_created_at ON council_verdicts (created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_council_verdicts_consensus_score ON council_verdicts (consensus_score DESC);

-- Judge evaluation performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judge_evaluations_verdict_id ON judge_evaluations (verdict_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judge_evaluations_judge_id ON judge_evaluations (judge_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judge_evaluations_evaluation_time ON judge_evaluations (evaluation_time_ms DESC);

-- Knowledge base search optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_relevance_score ON knowledge_entries (relevance_score DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_created_at ON knowledge_entries (created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_content_type ON knowledge_entries (content_type) WHERE content_type IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_source ON knowledge_entries (source);

-- Vector search optimization (for pgvector)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_embedding ON knowledge_entries USING ivfflat (embedding_vector vector_cosine_ops) WITH (lists = 100);

-- Judge availability and performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judges_is_active ON judges (is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judges_weight ON judges (weight DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judges_optimization_target ON judges (optimization_target);

-- Worker management
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workers_is_active ON workers (is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workers_worker_type ON workers (worker_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workers_specialty ON workers (specialty) WHERE specialty IS NOT NULL;

-- Debate session tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_debate_sessions_task_id ON debate_sessions (task_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_debate_sessions_status ON debate_sessions (status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_debate_sessions_resolved_at ON debate_sessions (resolved_at DESC) WHERE resolved_at IS NOT NULL;

-- Composite indexes for common query patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_status_created ON tasks (status, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_status_started ON task_executions (status, execution_started_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_relevance_created ON knowledge_entries (relevance_score DESC, created_at DESC);

-- Partial indexes for active records only
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_active_tasks ON tasks (id, title, status, priority) WHERE status NOT IN ('completed', 'cancelled');
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_active_executions ON task_executions (id, task_id, status, execution_started_at) WHERE status NOT IN ('completed', 'failed');

-- Text search optimization (if using full-text search)
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_content_fts ON knowledge_entries USING gin(to_tsvector('english', content));
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_description_fts ON tasks USING gin(to_tsvector('english', description));

-- JSONB indexes for metadata searches (if needed)
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_metadata ON tasks USING gin(metadata) WHERE metadata IS NOT NULL;
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_metadata ON knowledge_entries USING gin(metadata) WHERE metadata IS NOT NULL;

COMMENT ON INDEX idx_tasks_status IS 'Optimizes task status filtering in dashboards and monitoring';
COMMENT ON INDEX idx_tasks_created_at IS 'Optimizes task listing by creation time (most common sort)';
COMMENT ON INDEX idx_task_executions_started_at IS 'Optimizes execution timeline queries';
COMMENT ON INDEX idx_council_verdicts_task_id IS 'Critical index for verdict lookups during task processing';
COMMENT ON INDEX idx_judge_evaluations_evaluation_time IS 'Optimizes judge performance analysis';
COMMENT ON INDEX idx_knowledge_entries_relevance_score IS 'Optimizes knowledge base search ranking';
COMMENT ON INDEX idx_knowledge_entries_embedding IS 'Optimizes vector similarity search using pgvector';
