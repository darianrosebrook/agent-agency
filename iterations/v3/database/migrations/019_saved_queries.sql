-- ============================================================================
-- SAVED QUERIES MIGRATION
-- ============================================================================
-- Create saved queries table for storing reusable database queries
-- This addresses P0 requirement for query persistence and reuse
--
-- Schema includes:
-- - id: Unique identifier for each saved query
-- - name: Human-readable name for the query
-- - description: Optional description of what the query does
-- - query_sql: The actual SQL query text
-- - parameters: JSON schema for query parameters
-- - created_by: User/system that created the query
-- - created_at: Timestamp of creation
-- - updated_at: Timestamp of last modification
-- - tags: Array of tags for categorization
-- ============================================================================

-- Create the saved_queries table
CREATE TABLE IF NOT EXISTS saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    query_sql TEXT NOT NULL,
    parameters JSONB DEFAULT '{}'::jsonb,
    created_by VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    tags TEXT[] DEFAULT ARRAY[]::TEXT[]
);

-- Create indexes for efficient querying
CREATE INDEX idx_saved_queries_name ON saved_queries (name);
CREATE INDEX idx_saved_queries_created_by ON saved_queries (created_by);
CREATE INDEX idx_saved_queries_tags ON saved_queries USING GIN (tags);
CREATE INDEX idx_saved_queries_created_at ON saved_queries (created_at DESC);

-- Create a partial index for frequently accessed queries (updated recently)
CREATE INDEX idx_saved_queries_recent ON saved_queries (updated_at DESC)
WHERE updated_at > now() - interval '30 days';

-- Add table comment
COMMENT ON TABLE saved_queries IS 'Reusable database queries with metadata for query management and sharing';

-- Add column comments for clarity
COMMENT ON COLUMN saved_queries.name IS 'Human-readable unique name for the saved query';
COMMENT ON COLUMN saved_queries.description IS 'Optional description of what the query does and when to use it';
COMMENT ON COLUMN saved_queries.query_sql IS 'The actual SQL query text with parameter placeholders ($1, $2, etc.)';
COMMENT ON COLUMN saved_queries.parameters IS 'JSON schema defining expected parameters (names, types, defaults)';
COMMENT ON COLUMN saved_queries.created_by IS 'User or system identifier that created this query';
COMMENT ON COLUMN saved_queries.tags IS 'Array of tags for categorization and search (e.g., analytics, reports, maintenance)';

-- ============================================================================
-- DATA INTEGRITY & SECURITY
-- ============================================================================

-- Create a function to validate query parameters against schema
CREATE OR REPLACE FUNCTION validate_saved_query_parameters()
RETURNS TRIGGER AS $$
BEGIN
    -- Basic validation: ensure parameters is valid JSON
    IF NOT jsonb_typeof(NEW.parameters) = 'object' THEN
        RAISE EXCEPTION 'parameters must be a valid JSON object';
    END IF;

    -- Check for required parameter validation fields
    -- This is optional validation - queries can work without strict parameter validation

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger for parameter validation
CREATE TRIGGER validate_saved_query_parameters_trigger
    BEFORE INSERT OR UPDATE ON saved_queries
    FOR EACH ROW EXECUTE FUNCTION validate_saved_query_parameters();

-- Create a function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_saved_query_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update timestamp
CREATE TRIGGER update_saved_query_timestamp_trigger
    BEFORE UPDATE ON saved_queries
    FOR EACH ROW EXECUTE FUNCTION update_saved_query_timestamp();

-- ============================================================================
-- UTILITY FUNCTIONS
-- ============================================================================

-- Create a function to find queries by tags
CREATE OR REPLACE FUNCTION find_queries_by_tags(
    search_tags TEXT[],
    match_all BOOLEAN DEFAULT false
)
RETURNS TABLE (
    id UUID,
    name VARCHAR(255),
    description TEXT,
    tags TEXT[]
)
LANGUAGE SQL
STABLE
AS $$
    SELECT
        sq.id,
        sq.name,
        sq.description,
        sq.tags
    FROM saved_queries sq
    WHERE
        CASE
            WHEN match_all THEN sq.tags @> search_tags
            ELSE sq.tags && search_tags
        END;
$$;

COMMENT ON FUNCTION find_queries_by_tags(TEXT[], BOOLEAN) IS 'Find saved queries that match given tags. match_all=true requires all tags to match, false requires any tag to match';

-- ============================================================================
-- INITIAL DATA
-- ============================================================================

-- Insert some useful default queries for the system
INSERT INTO saved_queries (name, description, query_sql, parameters, created_by, tags) VALUES
(
    'task_summary_last_24h',
    'Summary of tasks executed in the last 24 hours',
    'SELECT
        COUNT(*) as total_tasks,
        COUNT(CASE WHEN status = ''completed'' THEN 1 END) as completed_tasks,
        COUNT(CASE WHEN status = ''failed'' THEN 1 END) as failed_tasks,
        AVG(EXTRACT(EPOCH FROM (updated_at - created_at))) as avg_duration_seconds
     FROM tasks
     WHERE created_at >= now() - interval ''24 hours''',
    '{}'::jsonb,
    'system',
    ARRAY['analytics', 'tasks', 'summary']
),
(
    'audit_events_by_category',
    'Count of audit events grouped by category and action',
    'SELECT
        category,
        action,
        COUNT(*) as event_count,
        MIN(ts) as first_event,
        MAX(ts) as last_event
     FROM task_audit_logs
     WHERE ts >= $1
     GROUP BY category, action
     ORDER BY event_count DESC',
    '{"start_time": {"type": "string", "format": "date-time", "description": "Start time for filtering events"}}'::jsonb,
    'system',
    ARRAY['analytics', 'audit', 'events']
);

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

-- Note: This migration creates the foundation for saved query management.
-- The initial data includes useful system queries for monitoring and analytics.
