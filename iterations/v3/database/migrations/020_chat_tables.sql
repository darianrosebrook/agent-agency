-- ============================================================================
-- CHAT TABLES MIGRATION
-- ============================================================================
-- Create chat_sessions and chat_messages tables for P0 WebSocket chat
-- This addresses P0 requirement: "Chat sessions persist across reconnects"
--
-- Schema includes:
-- - chat_sessions: Session metadata and WebSocket connection info
-- - chat_messages: Individual messages within sessions
-- - Proper indexing for real-time message retrieval
-- ============================================================================

-- Create chat_sessions table
CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id VARCHAR(255) NOT NULL UNIQUE, -- WebSocket session identifier
    user_id VARCHAR(255), -- Optional user identifier
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'expired')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_message_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb, -- Connection info, user agent, etc.
    expires_at TIMESTAMPTZ DEFAULT (now() + interval '24 hours')
);

-- Create chat_messages table
CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    message_type VARCHAR(20) NOT NULL DEFAULT 'message' CHECK (message_type IN ('message', 'system', 'error', 'typing')),
    content TEXT NOT NULL,
    sender VARCHAR(255), -- 'user', 'assistant', or specific user ID
    metadata JSONB DEFAULT '{}'::jsonb, -- Message metadata (emojis, attachments, etc.)
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    sequence_number BIGINT GENERATED ALWAYS AS IDENTITY -- For ordering within session
);

-- Create indexes for performance
CREATE INDEX idx_chat_sessions_session_id ON chat_sessions (session_id);
CREATE INDEX idx_chat_sessions_status ON chat_sessions (status);
CREATE INDEX idx_chat_sessions_user_id ON chat_sessions (user_id);
CREATE INDEX idx_chat_sessions_expires_at ON chat_sessions (expires_at);

CREATE INDEX idx_chat_messages_session_id ON chat_messages (session_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages (created_at DESC);
CREATE INDEX idx_chat_messages_sequence_number ON chat_messages (session_id, sequence_number);

-- Create a composite index for efficient message retrieval
CREATE INDEX idx_chat_sessions_messages ON chat_messages (session_id, created_at DESC, sequence_number DESC);

-- Add table comments
COMMENT ON TABLE chat_sessions IS 'WebSocket chat sessions with connection metadata and lifecycle management';
COMMENT ON TABLE chat_messages IS 'Individual chat messages within sessions with sequencing and metadata';

-- Add column comments
COMMENT ON COLUMN chat_sessions.session_id IS 'Unique WebSocket session identifier for reconnection';
COMMENT ON COLUMN chat_sessions.user_id IS 'Optional user identifier for multi-user chat support';
COMMENT ON COLUMN chat_sessions.metadata IS 'Connection metadata (IP, user agent, browser info)';
COMMENT ON COLUMN chat_sessions.expires_at IS 'Session expiration time (default 24 hours)';

COMMENT ON COLUMN chat_messages.message_type IS 'Type of message: message (user/assistant), system, error, typing';
COMMENT ON COLUMN chat_messages.sender IS 'Message sender identifier';
COMMENT ON COLUMN chat_messages.sequence_number IS 'Sequential message number within session for ordering';
COMMENT ON COLUMN chat_messages.metadata IS 'Message metadata (reactions, attachments, formatting)';

-- ============================================================================
-- DATA INTEGRITY & CLEANUP
-- ============================================================================

-- Create a function to clean up expired sessions
CREATE OR REPLACE FUNCTION cleanup_expired_chat_sessions()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM chat_sessions WHERE expires_at < now();
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION cleanup_expired_chat_sessions() IS 'Clean up expired chat sessions, returns number of sessions deleted';

-- Create a function to get recent messages for a session
CREATE OR REPLACE FUNCTION get_recent_chat_messages(
    p_session_id UUID,
    p_limit INTEGER DEFAULT 50,
    p_since TIMESTAMPTZ DEFAULT NULL
)
RETURNS TABLE (
    id UUID,
    message_type VARCHAR(20),
    content TEXT,
    sender VARCHAR(255),
    metadata JSONB,
    created_at TIMESTAMPTZ,
    sequence_number BIGINT
)
LANGUAGE SQL
STABLE
AS $$
    SELECT
        cm.id,
        cm.message_type,
        cm.content,
        cm.sender,
        cm.metadata,
        cm.created_at,
        cm.sequence_number
    FROM chat_messages cm
    WHERE cm.session_id = p_session_id
      AND (p_since IS NULL OR cm.created_at >= p_since)
    ORDER BY cm.sequence_number DESC
    LIMIT p_limit;
$$;

COMMENT ON FUNCTION get_recent_chat_messages(UUID, INTEGER, TIMESTAMPTZ) IS 'Get recent messages for a chat session with optional time filtering';

-- ============================================================================
-- INITIAL DATA & TESTING
-- ============================================================================

-- Insert a sample session for testing
INSERT INTO chat_sessions (session_id, user_id, metadata) VALUES
('test-session-001', 'test-user', '{"browser": "Chrome", "platform": "Web"}'::jsonb);

-- Insert sample messages
INSERT INTO chat_messages (session_id, message_type, content, sender, metadata) VALUES
(
    (SELECT id FROM chat_sessions WHERE session_id = 'test-session-001'),
    'system',
    'Chat session initialized',
    'system',
    '{"event": "session_start"}'::jsonb
),
(
    (SELECT id FROM chat_sessions WHERE session_id = 'test-session-001'),
    'message',
    'Hello! How can I help you today?',
    'assistant',
    '{"sentiment": "positive"}'::jsonb
);

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================

-- Note: This migration creates the foundation for WebSocket chat functionality.
-- The chat system supports session persistence, message sequencing, and
-- real-time communication through the P0 WebSocket implementation.
