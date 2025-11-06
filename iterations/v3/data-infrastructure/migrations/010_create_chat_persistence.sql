-- Create chat persistence schema
-- Migration: 010_create_chat_persistence.sql
-- Author: @darianrosebrook

-- ===========================================
-- CHAT SESSIONS
-- ===========================================

-- Store chat sessions for conversation tracking
CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID,
    tenant_id UUID,
    title VARCHAR(500),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_message_at TIMESTAMPTZ,
    message_count INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}',
    archived BOOLEAN DEFAULT FALSE,
    archived_at TIMESTAMPTZ
);

-- Indexes for chat sessions
CREATE INDEX IF NOT EXISTS idx_chat_sessions_workspace ON chat_sessions(workspace_id);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_tenant ON chat_sessions(tenant_id);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_updated_at ON chat_sessions(updated_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_archived ON chat_sessions(archived, updated_at DESC);

-- ===========================================
-- CHAT MESSAGES
-- ===========================================

-- Store individual chat messages
CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    edited_at TIMESTAMPTZ,
    token_count INTEGER,
    model_used VARCHAR(255),
    parent_message_id UUID REFERENCES chat_messages(id) ON DELETE SET NULL,
    sequence_number INTEGER NOT NULL DEFAULT 0
);

-- Indexes for chat messages
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX IF NOT EXISTS idx_chat_messages_created_at ON chat_messages(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_chat_messages_role ON chat_messages(role);
CREATE INDEX IF NOT EXISTS idx_chat_messages_sequence ON chat_messages(session_id, sequence_number);
CREATE INDEX IF NOT EXISTS idx_chat_messages_parent ON chat_messages(parent_message_id);

-- GIN index for JSONB metadata search
CREATE INDEX IF NOT EXISTS idx_chat_messages_metadata ON chat_messages USING GIN(metadata);

-- Composite index for session queries
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_sequence ON chat_messages(session_id, sequence_number DESC);

-- ===========================================
-- CHAT CONTEXT REFERENCE
-- ===========================================

-- Link chat messages to offloaded contexts
CREATE TABLE IF NOT EXISTS chat_context_links (
    message_id UUID NOT NULL REFERENCES chat_messages(id) ON DELETE CASCADE,
    context_id UUID NOT NULL REFERENCES agent_contexts(id) ON DELETE CASCADE,
    link_type VARCHAR(50) NOT NULL CHECK (link_type IN ('input', 'output', 'reference')),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    PRIMARY KEY (message_id, context_id)
);

CREATE INDEX IF NOT EXISTS idx_chat_context_links_message ON chat_context_links(message_id);
CREATE INDEX IF NOT EXISTS idx_chat_context_links_context ON chat_context_links(context_id);

-- ===========================================
-- TRIGGERS
-- ===========================================

-- Update session timestamp and message count on new message
CREATE OR REPLACE FUNCTION update_chat_session_on_message()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE chat_sessions
    SET updated_at = NOW(),
        last_message_at = NEW.created_at,
        message_count = message_count + 1
    WHERE id = NEW.session_id;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_chat_session
    AFTER INSERT ON chat_messages
    FOR EACH ROW
    EXECUTE FUNCTION update_chat_session_on_message();

-- ===========================================
-- FUNCTIONS
-- ===========================================

-- Get recent messages for a session
CREATE OR REPLACE FUNCTION get_chat_messages(
    p_session_id UUID,
    p_limit INTEGER DEFAULT 50,
    p_offset INTEGER DEFAULT 0
) RETURNS TABLE (
    id UUID,
    role VARCHAR(50),
    content TEXT,
    created_at TIMESTAMPTZ,
    metadata JSONB
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        cm.id,
        cm.role,
        cm.content,
        cm.created_at,
        cm.metadata
    FROM chat_messages cm
    WHERE cm.session_id = p_session_id
    ORDER BY cm.sequence_number ASC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- COMMENTS
-- ===========================================

COMMENT ON TABLE chat_sessions IS 'Chat conversation sessions with workspace/tenant isolation';
COMMENT ON TABLE chat_messages IS 'Individual chat messages with role tracking and threading';
COMMENT ON TABLE chat_context_links IS 'Links between chat messages and offloaded contexts';

-- Log migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('010', 'create_chat_persistence', NOW())
ON CONFLICT (version) DO NOTHING;

