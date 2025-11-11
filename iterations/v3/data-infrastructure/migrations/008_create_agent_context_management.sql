-- Create agent context management schema
-- Migration: 008_create_agent_context_management.sql

-- ===========================================
-- AGENT CONTEXTS
-- ===========================================

-- Store agent context data for preservation and working memory
CREATE TABLE IF NOT EXISTS agent_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    context_type VARCHAR(255) NOT NULL,
    content BYTEA NOT NULL, -- Compressed or raw content data
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    access_count BIGINT NOT NULL DEFAULT 0,
    size_bytes BIGINT NOT NULL DEFAULT 0,
    archive_location TEXT,
    compression_enabled BOOLEAN DEFAULT FALSE,
    folding_strategy VARCHAR(50),
    folded_at TIMESTAMPTZ,
    workspace_id UUID,
    session_id UUID,
    metadata_search JSONB DEFAULT '{}', -- For full-text search indexing
    created_at_index TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_agent_contexts_context_type ON agent_contexts(context_type);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_last_accessed_at ON agent_contexts(last_accessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_access_count ON agent_contexts(access_count DESC);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_workspace_id ON agent_contexts(workspace_id);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_session_id ON agent_contexts(session_id);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_folded_at ON agent_contexts(folded_at);

-- GIN index for JSONB metadata search
CREATE INDEX IF NOT EXISTS idx_agent_contexts_metadata_search ON agent_contexts USING GIN(metadata_search);

-- Composite index for lifecycle management queries
CREATE INDEX IF NOT EXISTS idx_agent_contexts_lifecycle ON agent_contexts(last_accessed_at, access_count, size_bytes);

-- ===========================================
-- FOLDED CONTEXTS
-- ===========================================

-- Store folded context data (compressed, summarized, archived, or deleted)
CREATE TABLE IF NOT EXISTS folded_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    context_id UUID NOT NULL REFERENCES agent_contexts(id) ON DELETE CASCADE,
    fold_type VARCHAR(50) NOT NULL CHECK (fold_type IN ('compressed', 'summarized', 'archived', 'deleted')),
    fold_data TEXT NOT NULL,
    folded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    original_size_bytes BIGINT,
    folded_size_bytes BIGINT,
    compression_ratio FLOAT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(context_id)
);

-- Indexes for folded contexts
CREATE INDEX IF NOT EXISTS idx_folded_contexts_context_id ON folded_contexts(context_id);
CREATE INDEX IF NOT EXISTS idx_folded_contexts_fold_type ON folded_contexts(fold_type);
CREATE INDEX IF NOT EXISTS idx_folded_contexts_folded_at ON folded_contexts(folded_at DESC);

-- ===========================================
-- CONTEXT ACCESS HISTORY
-- ===========================================

-- Track context access patterns for analytics
CREATE TABLE IF NOT EXISTS context_access_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    context_id UUID NOT NULL REFERENCES agent_contexts(id) ON DELETE CASCADE,
    accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    access_type VARCHAR(50) DEFAULT 'read', -- 'read', 'write', 'update', 'delete'
    workspace_id UUID,
    session_id UUID,
    metadata JSONB DEFAULT '{}'
);

-- Indexes for access history
CREATE INDEX IF NOT EXISTS idx_context_access_history_context_id ON context_access_history(context_id);
CREATE INDEX IF NOT EXISTS idx_context_access_history_accessed_at ON context_access_history(accessed_at DESC);
CREATE INDEX IF NOT EXISTS idx_context_access_history_workspace_id ON context_access_history(workspace_id);

-- ===========================================
-- CONTEXT STATISTICS VIEW
-- ===========================================

-- View for context statistics
CREATE OR REPLACE VIEW context_statistics AS
SELECT
    COUNT(*) as total_contexts,
    COALESCE(SUM(ac.size_bytes), 0) as total_storage_size,
    COUNT(*) FILTER (WHERE ac.folded_at IS NULL) as working_memory_contexts,
    COUNT(*) FILTER (WHERE ac.folded_at IS NOT NULL) as folded_contexts,
    COALESCE(AVG(ac.size_bytes), 0) as average_context_size,
    COUNT(*) FILTER (WHERE ac.last_accessed_at > NOW() - INTERVAL '1 hour') as recent_accesses,
    COUNT(*) FILTER (WHERE fc.fold_type = 'compressed') as compressed_count,
    COUNT(*) FILTER (WHERE fc.fold_type = 'summarized') as summarized_count,
    COUNT(*) FILTER (WHERE fc.fold_type = 'archived') as archived_count
FROM agent_contexts ac
LEFT JOIN folded_contexts fc ON ac.id = fc.context_id;

-- ===========================================
-- HELPER FUNCTIONS
-- ===========================================

-- Function to get current storage usage
CREATE OR REPLACE FUNCTION get_current_storage_usage()
RETURNS BIGINT AS $$
BEGIN
    RETURN COALESCE(SUM(size_bytes), 0) FROM agent_contexts;
END;
$$ LANGUAGE plpgsql;

-- Function to find contexts needing folding based on access patterns
CREATE OR REPLACE FUNCTION find_contexts_needing_folding(
    max_age_hours INTEGER DEFAULT 24,
    min_access_count BIGINT DEFAULT 0
)
RETURNS TABLE (context_id UUID) AS $$
BEGIN
    RETURN QUERY
    SELECT ac.id
    FROM agent_contexts ac
    WHERE ac.folded_at IS NULL
      AND ac.last_accessed_at < NOW() - (max_age_hours || ' hours')::INTERVAL
      AND ac.access_count <= min_access_count
    ORDER BY ac.last_accessed_at ASC
    LIMIT 100;
END;
$$ LANGUAGE plpgsql;

-- Function to cleanup old access history
CREATE OR REPLACE FUNCTION cleanup_old_access_history(retention_days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM context_access_history
    WHERE accessed_at < NOW() - (retention_days || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;




