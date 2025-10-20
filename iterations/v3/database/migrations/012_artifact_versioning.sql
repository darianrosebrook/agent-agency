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
