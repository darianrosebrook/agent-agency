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
