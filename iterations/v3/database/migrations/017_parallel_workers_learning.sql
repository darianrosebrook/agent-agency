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
