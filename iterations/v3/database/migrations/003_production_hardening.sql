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


