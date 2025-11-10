-- Add Search and Organization Features for Chat
-- Migration: 013_add_chat_search_and_organization.sql
-- Author: @darianrosebrook
-- Purpose: Add full-text search, folders, tags, and organization features

-- ===========================================
-- FULL-TEXT SEARCH SUPPORT
-- ===========================================

-- Add full-text search column to chat_sessions for title search
-- PostgreSQL full-text search uses tsvector for efficient searching
ALTER TABLE chat_sessions 
ADD COLUMN IF NOT EXISTS title_search_vector tsvector;

-- Add full-text search column to chat_messages for content search
ALTER TABLE chat_messages 
ADD COLUMN IF NOT EXISTS content_search_vector tsvector;

-- Create GIN indexes for full-text search (GIN is optimal for tsvector)
CREATE INDEX IF NOT EXISTS idx_chat_sessions_title_search 
ON chat_sessions USING GIN(title_search_vector);

CREATE INDEX IF NOT EXISTS idx_chat_messages_content_search 
ON chat_messages USING GIN(content_search_vector);

-- Function to update title search vector
CREATE OR REPLACE FUNCTION update_chat_session_title_search()
RETURNS TRIGGER AS $$
BEGIN
    NEW.title_search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.title, '')), 'A');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Function to update message content search vector
CREATE OR REPLACE FUNCTION update_chat_message_content_search()
RETURNS TRIGGER AS $$
BEGIN
    NEW.content_search_vector := 
        setweight(to_tsvector('english', COALESCE(NEW.content, '')), 'A');
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Triggers to automatically update search vectors
CREATE TRIGGER trigger_update_session_title_search
    BEFORE INSERT OR UPDATE OF title ON chat_sessions
    FOR EACH ROW
    EXECUTE FUNCTION update_chat_session_title_search();

CREATE TRIGGER trigger_update_message_content_search
    BEFORE INSERT OR UPDATE OF content ON chat_messages
    FOR EACH ROW
    EXECUTE FUNCTION update_chat_message_content_search();

-- Update existing rows
UPDATE chat_sessions 
SET title_search_vector = setweight(to_tsvector('english', COALESCE(title, '')), 'A')
WHERE title_search_vector IS NULL;

UPDATE chat_messages 
SET content_search_vector = setweight(to_tsvector('english', COALESCE(content, '')), 'A')
WHERE content_search_vector IS NULL;

-- ===========================================
-- FOLDERS FOR ORGANIZATION
-- ===========================================

-- Create folders table for organizing chat sessions
CREATE TABLE IF NOT EXISTS chat_folders (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID,
    tenant_id UUID,
    name VARCHAR(255) NOT NULL,
    parent_folder_id UUID REFERENCES chat_folders(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    metadata JSONB DEFAULT '{}',
    UNIQUE(workspace_id, name, parent_folder_id)
);

-- Indexes for folders
CREATE INDEX IF NOT EXISTS idx_chat_folders_workspace ON chat_folders(workspace_id);
CREATE INDEX IF NOT EXISTS idx_chat_folders_tenant ON chat_folders(tenant_id);
CREATE INDEX IF NOT EXISTS idx_chat_folders_parent ON chat_folders(parent_folder_id);
CREATE INDEX IF NOT EXISTS idx_chat_folders_name ON chat_folders(name);

-- Add folder_id to chat_sessions
ALTER TABLE chat_sessions 
ADD COLUMN IF NOT EXISTS folder_id UUID REFERENCES chat_folders(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_chat_sessions_folder ON chat_sessions(folder_id);

-- ===========================================
-- TAGS FOR ORGANIZATION
-- ===========================================

-- Create tags table
CREATE TABLE IF NOT EXISTS chat_tags (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID,
    tenant_id UUID,
    name VARCHAR(100) NOT NULL,
    color VARCHAR(7), -- Hex color code
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(workspace_id, name)
);

-- Indexes for tags
CREATE INDEX IF NOT EXISTS idx_chat_tags_workspace ON chat_tags(workspace_id);
CREATE INDEX IF NOT EXISTS idx_chat_tags_tenant ON chat_tags(tenant_id);
CREATE INDEX IF NOT EXISTS idx_chat_tags_name ON chat_tags(name);

-- Junction table for many-to-many relationship between sessions and tags
CREATE TABLE IF NOT EXISTS chat_session_tags (
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    tag_id UUID NOT NULL REFERENCES chat_tags(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (session_id, tag_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_session_tags_session ON chat_session_tags(session_id);
CREATE INDEX IF NOT EXISTS idx_chat_session_tags_tag ON chat_session_tags(tag_id);

-- ===========================================
-- PINNED SESSIONS
-- ===========================================

-- Add pinned flag to chat_sessions
ALTER TABLE chat_sessions 
ADD COLUMN IF NOT EXISTS pinned BOOLEAN DEFAULT FALSE;

CREATE INDEX IF NOT EXISTS idx_chat_sessions_pinned ON chat_sessions(pinned, updated_at DESC);

-- ===========================================
-- SEARCH FUNCTIONS
-- ===========================================

-- Search chat sessions by title and message content
CREATE OR REPLACE FUNCTION search_chat_sessions(
    p_workspace_id UUID,
    p_search_text TEXT,
    p_archived BOOLEAN DEFAULT FALSE,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE (
    id UUID,
    workspace_id UUID,
    tenant_id UUID,
    title VARCHAR(500),
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    last_message_at TIMESTAMPTZ,
    message_count INTEGER,
    metadata JSONB,
    archived BOOLEAN,
    pinned BOOLEAN,
    folder_id UUID,
    relevance REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT
        cs.id,
        cs.workspace_id,
        cs.tenant_id,
        cs.title,
        cs.created_at,
        cs.updated_at,
        cs.last_message_at,
        cs.message_count,
        cs.metadata,
        cs.archived,
        cs.pinned,
        cs.folder_id,
        -- Calculate relevance score
        (
            -- Title match (higher weight)
            ts_rank(cs.title_search_vector, plainto_tsquery('english', p_search_text)) * 2.0 +
            -- Message content match (lower weight)
            COALESCE((
                SELECT MAX(ts_rank(cm.content_search_vector, plainto_tsquery('english', p_search_text)))
                FROM chat_messages cm
                WHERE cm.session_id = cs.id
            ), 0.0)
        ) AS relevance
    FROM chat_sessions cs
    LEFT JOIN chat_messages cm ON cm.session_id = cs.id
    WHERE cs.workspace_id = p_workspace_id
      AND cs.archived = p_archived
      AND (
          -- Search in title
          cs.title_search_vector @@ plainto_tsquery('english', p_search_text)
          OR
          -- Search in message content
          cm.content_search_vector @@ plainto_tsquery('english', p_search_text)
      )
    ORDER BY relevance DESC, cs.updated_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- Search chat messages within a session
CREATE OR REPLACE FUNCTION search_chat_messages(
    p_session_id UUID,
    p_search_text TEXT,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE (
    id UUID,
    session_id UUID,
    role VARCHAR(50),
    content TEXT,
    created_at TIMESTAMPTZ,
    sequence_number INTEGER,
    relevance REAL
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        cm.id,
        cm.session_id,
        cm.role,
        cm.content,
        cm.created_at,
        cm.sequence_number,
        ts_rank(cm.content_search_vector, plainto_tsquery('english', p_search_text)) AS relevance
    FROM chat_messages cm
    WHERE cm.session_id = p_session_id
      AND cm.content_search_vector @@ plainto_tsquery('english', p_search_text)
    ORDER BY relevance DESC, cm.sequence_number ASC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- FILTER FUNCTIONS
-- ===========================================

-- Filter sessions by tags
CREATE OR REPLACE FUNCTION filter_sessions_by_tags(
    p_workspace_id UUID,
    p_tag_names TEXT[],
    p_archived BOOLEAN DEFAULT FALSE,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE (
    id UUID,
    workspace_id UUID,
    title VARCHAR(500),
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    message_count INTEGER,
    archived BOOLEAN
) AS $$
BEGIN
    RETURN QUERY
    SELECT DISTINCT
        cs.id,
        cs.workspace_id,
        cs.title,
        cs.created_at,
        cs.updated_at,
        cs.message_count,
        cs.archived
    FROM chat_sessions cs
    INNER JOIN chat_session_tags cst ON cst.session_id = cs.id
    INNER JOIN chat_tags ct ON ct.id = cst.tag_id
    WHERE cs.workspace_id = p_workspace_id
      AND cs.archived = p_archived
      AND ct.name = ANY(p_tag_names)
    ORDER BY cs.updated_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- Filter sessions by date range
CREATE OR REPLACE FUNCTION filter_sessions_by_date_range(
    p_workspace_id UUID,
    p_start_date TIMESTAMPTZ,
    p_end_date TIMESTAMPTZ,
    p_archived BOOLEAN DEFAULT FALSE,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE (
    id UUID,
    workspace_id UUID,
    title VARCHAR(500),
    created_at TIMESTAMPTZ,
    updated_at TIMESTAMPTZ,
    message_count INTEGER
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        cs.id,
        cs.workspace_id,
        cs.title,
        cs.created_at,
        cs.updated_at,
        cs.message_count
    FROM chat_sessions cs
    WHERE cs.workspace_id = p_workspace_id
      AND cs.archived = p_archived
      AND cs.updated_at BETWEEN p_start_date AND p_end_date
    ORDER BY cs.updated_at DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- BULK OPERATIONS
-- ===========================================

-- Bulk archive sessions
CREATE OR REPLACE FUNCTION bulk_archive_sessions(
    p_session_ids UUID[]
) RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    UPDATE chat_sessions
    SET archived = TRUE,
        archived_at = NOW(),
        updated_at = NOW()
    WHERE id = ANY(p_session_ids)
      AND archived = FALSE;
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

-- Bulk delete sessions
CREATE OR REPLACE FUNCTION bulk_delete_sessions(
    p_session_ids UUID[]
) RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM chat_sessions
    WHERE id = ANY(p_session_ids);
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Bulk move sessions to folder
CREATE OR REPLACE FUNCTION bulk_move_sessions_to_folder(
    p_session_ids UUID[],
    p_folder_id UUID
) RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    UPDATE chat_sessions
    SET folder_id = p_folder_id,
        updated_at = NOW()
    WHERE id = ANY(p_session_ids);
    
    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

-- Bulk add tags to sessions
CREATE OR REPLACE FUNCTION bulk_add_tags_to_sessions(
    p_session_ids UUID[],
    p_tag_ids UUID[]
) RETURNS INTEGER AS $$
DECLARE
    inserted_count INTEGER;
BEGIN
    INSERT INTO chat_session_tags (session_id, tag_id)
    SELECT DISTINCT s.id, t.id
    FROM unnest(p_session_ids) AS s(id)
    CROSS JOIN unnest(p_tag_ids) AS t(id)
    ON CONFLICT (session_id, tag_id) DO NOTHING;
    
    GET DIAGNOSTICS inserted_count = ROW_COUNT;
    RETURN inserted_count;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- COMMENTS
-- ===========================================

COMMENT ON COLUMN chat_sessions.title_search_vector IS 
'Full-text search vector for title search';

COMMENT ON COLUMN chat_messages.content_search_vector IS 
'Full-text search vector for message content search';

COMMENT ON TABLE chat_folders IS 
'Folders for organizing chat sessions hierarchically';

COMMENT ON TABLE chat_tags IS 
'Tags for categorizing chat sessions';

COMMENT ON TABLE chat_session_tags IS 
'Many-to-many relationship between sessions and tags';

COMMENT ON FUNCTION search_chat_sessions IS 
'Search chat sessions by title and message content with relevance ranking';

COMMENT ON FUNCTION search_chat_messages IS 
'Search messages within a session with relevance ranking';

COMMENT ON FUNCTION filter_sessions_by_tags IS 
'Filter sessions by tag names';

COMMENT ON FUNCTION filter_sessions_by_date_range IS 
'Filter sessions by date range';

COMMENT ON FUNCTION bulk_archive_sessions IS 
'Archive multiple sessions at once';

COMMENT ON FUNCTION bulk_delete_sessions IS 
'Delete multiple sessions at once';

COMMENT ON FUNCTION bulk_move_sessions_to_folder IS 
'Move multiple sessions to a folder';

COMMENT ON FUNCTION bulk_add_tags_to_sessions IS 
'Add multiple tags to multiple sessions';

-- ===========================================
-- MIGRATION LOG
-- ===========================================

INSERT INTO migration_log (version, description, applied_at)
VALUES ('013', 'add_chat_search_and_organization', NOW())
ON CONFLICT (version) DO NOTHING;


