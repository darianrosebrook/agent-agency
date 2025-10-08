-- Performance Optimizations for Multi-Tenant Memory System
-- Migration: 002_add_performance_optimizations.sql
-- Description: Additional indexes, partitioning, and performance optimizations

BEGIN;

-- ============================================================================
-- ADDITIONAL INDEXES FOR PERFORMANCE
-- ============================================================================

-- Composite indexes for common query patterns
CREATE INDEX idx_contextual_memories_tenant_relevance ON contextual_memories(tenant_id, relevance_score DESC);
CREATE INDEX idx_contextual_memories_tenant_created ON contextual_memories(tenant_id, created_at DESC);
CREATE INDEX idx_contextual_memories_tenant_access ON contextual_memories(tenant_id, access_count DESC);

-- Partial indexes for active records
CREATE INDEX idx_active_contextual_memories ON contextual_memories(tenant_id, created_at DESC)
WHERE expires_at IS NULL OR expires_at > NOW();

CREATE INDEX idx_active_offloaded_contexts ON offloaded_contexts(tenant_id, created_at DESC)
WHERE expires_at IS NULL OR expires_at > NOW();

-- JSONB indexes for metadata queries
CREATE INDEX idx_contextual_memories_metadata ON contextual_memories USING gin(metadata);
CREATE INDEX idx_offloaded_contexts_metadata ON offloaded_contexts USING gin(metadata);
CREATE INDEX idx_audit_log_details ON audit_log USING gin(details);

-- Full-text search indexes
CREATE INDEX idx_contextual_memories_content_fts ON contextual_memories
USING gin(to_tsvector('english', content::text));

-- ============================================================================
-- PARTITIONING STRATEGIES
-- ============================================================================

-- Partition audit_log by month for better performance
-- Note: This requires PostgreSQL 10+ with declarative partitioning

/*
-- Create partition table (uncomment if using partitioning)
CREATE TABLE audit_log_y2025m01 PARTITION OF audit_log
    FOR VALUES FROM ('2025-01-01') TO ('2025-02-01');

CREATE TABLE audit_log_y2025m02 PARTITION OF audit_log
    FOR VALUES FROM ('2025-02-01') TO ('2025-03-01');

-- Add more partitions as needed...
*/

-- ============================================================================
-- MATERIALIZED VIEWS FOR ANALYTICS
-- ============================================================================

-- Materialized view for tenant memory statistics (refresh periodically)
CREATE MATERIALIZED VIEW tenant_memory_stats AS
SELECT
    t.tenant_id,
    t.project_id,
    COUNT(cm.id) as memory_count,
    AVG(cm.relevance_score) as avg_relevance,
    MAX(cm.created_at) as latest_memory,
    COUNT(oc.id) as offloaded_count,
    AVG(oc.compression_ratio) as avg_compression,
    SUM(cm.access_count) as total_accesses,
    COUNT(CASE WHEN cm.expires_at < NOW() THEN 1 END) as expired_memories
FROM tenants t
LEFT JOIN contextual_memories cm ON t.id = cm.tenant_id
LEFT JOIN offloaded_contexts oc ON t.id = oc.tenant_id
GROUP BY t.id, t.tenant_id, t.project_id;

-- Materialized view for system performance metrics
CREATE MATERIALIZED VIEW system_performance_summary AS
SELECT
    DATE_TRUNC('hour', timestamp) as hour,
    metric_type,
    AVG(value) as avg_value,
    MIN(value) as min_value,
    MAX(value) as max_value,
    COUNT(*) as sample_count
FROM performance_metrics
WHERE timestamp > NOW() - INTERVAL '30 days'
GROUP BY DATE_TRUNC('hour', timestamp), metric_type
ORDER BY hour DESC, metric_type;

-- Create indexes on materialized views
CREATE INDEX idx_tenant_memory_stats_tenant ON tenant_memory_stats(tenant_id);
CREATE INDEX idx_system_performance_hour ON system_performance_summary(hour);

-- ============================================================================
-- ADDITIONAL CONSTRAINTS AND VALIDATIONS
-- ============================================================================

-- Add check constraints for data integrity
ALTER TABLE contextual_memories
ADD CONSTRAINT valid_memory_id_format CHECK (memory_id ~ '^[a-zA-Z0-9_-]+$');

ALTER TABLE offloaded_contexts
ADD CONSTRAINT valid_context_id_format CHECK (context_id ~ '^[a-zA-Z0-9_-]+$');

-- Add foreign key constraints with better naming
ALTER TABLE tenant_access_policies
ADD CONSTRAINT fk_tenant_access_policies_tenant
FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

ALTER TABLE tenant_sharing_rules
ADD CONSTRAINT fk_tenant_sharing_rules_source
FOREIGN KEY (source_tenant_id) REFERENCES tenants(id) ON DELETE CASCADE,
ADD CONSTRAINT fk_tenant_sharing_rules_target
FOREIGN KEY (target_tenant_id) REFERENCES tenants(id) ON DELETE CASCADE;

-- ============================================================================
-- UTILITY FUNCTIONS
-- ============================================================================

-- Function to get tenant memory usage
CREATE OR REPLACE FUNCTION get_tenant_memory_usage(p_tenant_id UUID)
RETURNS TABLE (
    memory_count BIGINT,
    total_size_bytes BIGINT,
    avg_relevance DECIMAL,
    offloaded_count BIGINT,
    compression_savings DECIMAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        COUNT(cm.id) as memory_count,
        SUM(LENGTH(cm.content::text)) as total_size_bytes,
        AVG(cm.relevance_score) as avg_relevance,
        COUNT(oc.id) as offloaded_count,
        COALESCE(AVG(oc.compression_ratio), 0) as compression_savings
    FROM tenants t
    LEFT JOIN contextual_memories cm ON t.id = cm.tenant_id
    LEFT JOIN offloaded_contexts oc ON t.id = oc.tenant_id
    WHERE t.id = p_tenant_id
    GROUP BY t.id;
END;
$$ LANGUAGE plpgsql;

-- Function to clean expired memories
CREATE OR REPLACE FUNCTION clean_expired_memories()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    -- Delete expired contextual memories
    DELETE FROM contextual_memories
    WHERE expires_at IS NOT NULL AND expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;

    -- Delete expired offloaded contexts
    DELETE FROM offloaded_contexts
    WHERE expires_at IS NOT NULL AND expires_at < NOW();

    -- Log cleanup operation
    INSERT INTO audit_log (operation, resource_type, details)
    VALUES ('cleanup_expired', 'system', jsonb_build_object('deleted_memories', deleted_count));

    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate tenant reputation for federated learning
CREATE OR REPLACE FUNCTION calculate_tenant_reputation(p_tenant_id UUID)
RETURNS DECIMAL(3,2) AS $$
DECLARE
    reputation DECIMAL(3,2);
BEGIN
    SELECT
        CASE
            WHEN fp.total_contributions = 0 THEN 0.5
            ELSE LEAST(1.0, GREATEST(0.0,
                0.3 +  -- Base reputation
                (fp.total_contributions::DECIMAL / 100.0) * 0.4 +  -- Contribution bonus
                (fp.reputation_score - 0.5) * 0.3  -- Existing reputation factor
            ))
        END INTO reputation
    FROM federated_participants fp
    WHERE fp.tenant_id = p_tenant_id;

    -- Return default if no participant record exists
    IF reputation IS NULL THEN
        RETURN 0.5;
    END IF;

    RETURN reputation;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- DATA CLEANUP POLICIES
-- ============================================================================

-- Create a function to archive old audit logs (example retention policy)
CREATE OR REPLACE FUNCTION archive_old_audit_logs(days_old INTEGER DEFAULT 365)
RETURNS INTEGER AS $$
DECLARE
    archived_count INTEGER;
BEGIN
    -- In a real implementation, this would move records to an archive table
    -- For now, we'll just count them
    SELECT COUNT(*) INTO archived_count
    FROM audit_log
    WHERE timestamp < NOW() - INTERVAL '1 day' * days_old;

    -- Log the archival operation
    INSERT INTO audit_log (operation, resource_type, details)
    VALUES ('archive_audit_logs', 'system',
        jsonb_build_object('archived_count', archived_count, 'days_old', days_old));

    RETURN archived_count;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- PERFORMANCE MONITORING FUNCTIONS
-- ============================================================================

-- Function to log performance metrics
CREATE OR REPLACE FUNCTION log_performance_metric(
    p_tenant_id UUID,
    p_metric_type VARCHAR(100),
    p_metric_name VARCHAR(255),
    p_value DECIMAL,
    p_unit VARCHAR(50) DEFAULT NULL,
    p_context JSONB DEFAULT '{}'
)
RETURNS UUID AS $$
DECLARE
    metric_id UUID;
BEGIN
    INSERT INTO performance_metrics (
        tenant_id, metric_type, metric_name, value, unit, context
    ) VALUES (
        p_tenant_id, p_metric_type, p_metric_name, p_value, p_unit, p_context
    )
    RETURNING id INTO metric_id;

    RETURN metric_id;
END;
$$ LANGUAGE plpgsql;

-- Function to log system health
CREATE OR REPLACE FUNCTION log_system_health(
    p_component VARCHAR(100),
    p_metric_name VARCHAR(255),
    p_value DECIMAL,
    p_status VARCHAR(50) DEFAULT 'healthy',
    p_details JSONB DEFAULT '{}'
)
RETURNS UUID AS $$
DECLARE
    health_id UUID;
BEGIN
    INSERT INTO system_health (
        component, metric_name, value, status, details
    ) VALUES (
        p_component, p_metric_name, p_value, p_status, p_details
    )
    RETURNING id INTO health_id;

    RETURN health_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- SCHEDULED MAINTENANCE (Example cron jobs)
-- ============================================================================

-- Note: These would typically be run by a scheduler like pg_cron

-- Refresh materialized views periodically
CREATE OR REPLACE FUNCTION refresh_analytics_views()
RETURNS VOID AS $$
BEGIN
    REFRESH MATERIALIZED VIEW CONCURRENTLY tenant_memory_stats;
    REFRESH MATERIALIZED VIEW CONCURRENTLY system_performance_summary;

    -- Log the refresh operation
    PERFORM log_system_health('database', 'analytics_refresh', 1.0, 'healthy',
        '{"refreshed_views": ["tenant_memory_stats", "system_performance_summary"]}');
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- RECORD MIGRATION
-- ============================================================================

INSERT INTO schema_migrations (version, name, checksum) VALUES
('002', 'add_performance_optimizations', 'performance_optimizations_v1');

COMMIT;
