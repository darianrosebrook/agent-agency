-- Database Query Optimization Migration
-- Migration: 012_optimize_chat_queries.sql
-- Author: @darianrosebrook
-- Purpose: Add performance indexes, optimize queries, and improve pagination

BEGIN;

-- ===========================================
-- ADDITIONAL INDEXES FOR COMMON QUERY PATTERNS
-- ===========================================

-- Composite index for workspace + archived + updated_at queries (most common)
-- This covers: list_workspace_sessions WHERE workspace_id = X AND archived = Y ORDER BY updated_at DESC
CREATE INDEX IF NOT EXISTS idx_chat_sessions_workspace_archived_updated 
ON chat_sessions(workspace_id, archived, updated_at DESC)
WHERE workspace_id IS NOT NULL;

-- Composite index for tenant + archived + updated_at queries
CREATE INDEX IF NOT EXISTS idx_chat_sessions_tenant_archived_updated 
ON chat_sessions(tenant_id, archived, updated_at DESC)
WHERE tenant_id IS NOT NULL;

-- Index for created_at queries (for date range filtering)
CREATE INDEX IF NOT EXISTS idx_chat_sessions_created_at 
ON chat_sessions(created_at DESC);

-- Index for last_message_at (for sorting by most recent activity)
CREATE INDEX IF NOT EXISTS idx_chat_sessions_last_message_at 
ON chat_sessions(last_message_at DESC NULLS LAST);

-- Composite index for session_id + sequence_number (already exists but ensure it's optimal)
-- This is critical for get_session_messages queries
-- Note: idx_chat_messages_session_sequence already exists, but we'll verify it's optimal

-- Index for message count queries (for filtering by activity level)
CREATE INDEX IF NOT EXISTS idx_chat_sessions_message_count 
ON chat_sessions(message_count DESC);

-- ===========================================
-- OPTIMIZE SEQUENCE NUMBER QUERIES
-- ===========================================

-- Create a function to get next sequence number atomically
-- This uses advisory locks to ensure atomicity across concurrent requests
CREATE OR REPLACE FUNCTION get_next_sequence_number(p_session_id UUID)
RETURNS INTEGER AS $$
DECLARE
    next_seq INTEGER;
    lock_id BIGINT;
BEGIN
    -- Use session_id hash as advisory lock ID to serialize per-session
    -- This ensures atomic sequence number generation even with concurrent requests
    lock_id := hashtext(p_session_id::TEXT);
    
    -- Acquire advisory lock for this session
    PERFORM pg_advisory_xact_lock(lock_id);
    
    -- Get next sequence number atomically
    SELECT COALESCE(MAX(sequence_number), 0) + 1
    INTO next_seq
    FROM chat_messages
    WHERE session_id = p_session_id;
    
    RETURN next_seq;
END;
$$ LANGUAGE plpgsql;

-- Better approach: Use a sequence per session (more scalable)
-- However, this requires application changes, so we'll optimize the current approach
-- by ensuring the index is used efficiently

-- ===========================================
-- OPTIMIZE PAGINATION QUERIES
-- ===========================================

-- Create a function to get total count efficiently
-- This uses COUNT(*) with proper index usage
CREATE OR REPLACE FUNCTION get_chat_messages_count(p_session_id UUID)
RETURNS INTEGER AS $$
DECLARE
    msg_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO msg_count
    FROM chat_messages
    WHERE session_id = p_session_id;
    
    RETURN msg_count;
END;
$$ LANGUAGE plpgsql;

-- Create a function to get chat sessions count for pagination
CREATE OR REPLACE FUNCTION get_chat_sessions_count(
    p_workspace_id UUID,
    p_archived BOOLEAN DEFAULT FALSE
) RETURNS INTEGER AS $$
DECLARE
    session_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO session_count
    FROM chat_sessions
    WHERE workspace_id = p_workspace_id
      AND archived = p_archived;
    
    RETURN session_count;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- OPTIMIZE MESSAGE RETRIEVAL WITH CURSOR-BASED PAGINATION
-- ===========================================

-- Cursor-based pagination is more efficient than OFFSET for large datasets
-- This function uses sequence_number as cursor (already indexed)
CREATE OR REPLACE FUNCTION get_chat_messages_cursor(
    p_session_id UUID,
    p_cursor INTEGER DEFAULT 0,
    p_limit INTEGER DEFAULT 50
) RETURNS TABLE (
    id UUID,
    session_id UUID,
    role VARCHAR(50),
    content TEXT,
    metadata JSONB,
    created_at TIMESTAMPTZ,
    edited_at TIMESTAMPTZ,
    token_count INTEGER,
    model_used VARCHAR(255),
    sequence_number INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        cm.id,
        cm.session_id,
        cm.role,
        cm.content,
        cm.metadata,
        cm.created_at,
        cm.edited_at,
        cm.token_count,
        cm.model_used,
        cm.sequence_number
    FROM chat_messages cm
    WHERE cm.session_id = p_session_id
      AND cm.sequence_number > p_cursor
    ORDER BY cm.sequence_number ASC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- ANALYZE TABLES FOR QUERY PLANNER
-- ===========================================

-- Update table statistics for better query planning
ANALYZE chat_sessions;
ANALYZE chat_messages;
ANALYZE chat_context_links;

-- ===========================================
-- QUERY PERFORMANCE MONITORING VIEWS
-- ===========================================

-- Create a view to monitor slow queries (requires pg_stat_statements extension)
-- This is informational and helps identify optimization opportunities
CREATE OR REPLACE VIEW chat_query_stats AS
SELECT 
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch,
    n_tup_ins,
    n_tup_upd,
    n_tup_del,
    n_live_tup,
    n_dead_tup,
    last_vacuum,
    last_autovacuum,
    last_analyze,
    last_autoanalyze
FROM pg_stat_user_tables
WHERE schemaname = current_schema()
  AND tablename IN ('chat_sessions', 'chat_messages', 'chat_context_links');

-- ===========================================
-- INDEX USAGE MONITORING
-- ===========================================

-- View to monitor index usage
CREATE OR REPLACE VIEW chat_index_usage AS
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan,
    idx_tup_read,
    idx_tup_fetch
FROM pg_stat_user_indexes
WHERE schemaname = current_schema()
  AND tablename IN ('chat_sessions', 'chat_messages', 'chat_context_links')
ORDER BY idx_scan DESC;

-- ===========================================
-- COMMENTS
-- ===========================================

COMMENT ON INDEX idx_chat_sessions_workspace_archived_updated IS 
'Composite index for efficient workspace session listing with archived filter and date sorting';

COMMENT ON INDEX idx_chat_sessions_tenant_archived_updated IS 
'Composite index for efficient tenant session listing with archived filter and date sorting';

COMMENT ON FUNCTION get_next_sequence_number(UUID) IS 
'Atomically get next sequence number for a chat session';

COMMENT ON FUNCTION get_chat_messages_count(UUID) IS 
'Get total message count for a session (for pagination)';

COMMENT ON FUNCTION get_chat_sessions_count(UUID, BOOLEAN) IS 
'Get total session count for a workspace (for pagination)';

COMMENT ON FUNCTION get_chat_messages_cursor(UUID, INTEGER, INTEGER) IS 
'Cursor-based pagination for chat messages (more efficient than OFFSET)';

COMMENT ON VIEW chat_query_stats IS 
'Monitor query statistics for chat tables';

COMMENT ON VIEW chat_index_usage IS 
'Monitor index usage for chat tables';

-- ===========================================
-- MIGRATION LOG
-- ===========================================

INSERT INTO migration_log (version, description, applied_at)
VALUES ('012', 'optimize_chat_queries', NOW())
ON CONFLICT (version) DO NOTHING;

COMMIT;

