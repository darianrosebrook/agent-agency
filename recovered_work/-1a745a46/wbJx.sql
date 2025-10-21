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
