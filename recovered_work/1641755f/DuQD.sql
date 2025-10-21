-- Migration 014: Core P0 Persistence Schema
-- Adds foundational tables for task execution, audit trails, chat sessions, and saved queries
-- Author: @darianrosebrook
-- Date: 2025-10-20

BEGIN;

-- ============================================================================
-- TASKS TABLE - Core task execution state
-- ============================================================================

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    spec JSONB NOT NULL,
    state VARCHAR(50) NOT NULL DEFAULT 'pending' CHECK (state IN ('pending', 'executing', 'completed', 'failed', 'canceled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255),
    metadata JSONB DEFAULT '{}'::jsonb,
    acceptance_criteria TEXT[] DEFAULT ARRAY[]::TEXT[]
);

CREATE INDEX idx_tasks_state ON tasks(state);
CREATE INDEX idx_tasks_created_at ON tasks(created_at DESC);
CREATE INDEX idx_tasks_updated_at ON tasks(updated_at DESC);
COMMENT ON TABLE tasks IS 'Core task execution records with state transitions';

-- ============================================================================
-- AUDIT LOGS TABLE - Decision and action audit trail
-- ============================================================================

CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    action VARCHAR(255) NOT NULL,
    actor VARCHAR(255),
    resource_id UUID,
    resource_type VARCHAR(50),
    change_summary JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_audit_logs_resource ON audit_logs(resource_id);
CREATE INDEX idx_audit_logs_action ON audit_logs(action);
CREATE INDEX idx_audit_logs_actor ON audit_logs(actor);
CREATE INDEX idx_audit_logs_created_at ON audit_logs(created_at DESC);
COMMENT ON TABLE audit_logs IS 'Immutable audit trail for all decisions, actions, and changes';

-- ============================================================================
-- CHAT SESSIONS TABLE - User chat session lifecycle
-- ============================================================================

CREATE TABLE IF NOT EXISTS chat_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_chat_sessions_created_at ON chat_sessions(created_at DESC);
CREATE INDEX idx_chat_sessions_ended_at ON chat_sessions(ended_at) WHERE ended_at IS NOT NULL;
COMMENT ON TABLE chat_sessions IS 'Chat session lifecycle management';

-- ============================================================================
-- CHAT MESSAGES TABLE - Individual messages in a session
-- ============================================================================

CREATE TABLE IF NOT EXISTS chat_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL REFERENCES chat_sessions(id) ON DELETE CASCADE,
    role VARCHAR(50) NOT NULL CHECK (role IN ('user', 'assistant', 'system')),
    content TEXT NOT NULL,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_chat_messages_session_id ON chat_messages(session_id);
CREATE INDEX idx_chat_messages_created_at ON chat_messages(created_at DESC);
COMMENT ON TABLE chat_messages IS 'Individual chat messages with role tracking';

-- ============================================================================
-- SAVED QUERIES TABLE - User-saved database queries
-- ============================================================================

CREATE TABLE IF NOT EXISTS saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID,
    name VARCHAR(255) NOT NULL,
    query_text TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, name)
);

CREATE INDEX idx_saved_queries_user_id ON saved_queries(user_id);
CREATE INDEX idx_saved_queries_name ON saved_queries(name);
CREATE INDEX idx_saved_queries_created_at ON saved_queries(created_at DESC);
COMMENT ON TABLE saved_queries IS 'Saved database queries for dashboard exploration';

-- ============================================================================
-- WAIVERS TABLE - Quality gate bypass approvals
-- ============================================================================

CREATE TABLE IF NOT EXISTS waivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(255) NOT NULL,
    reason VARCHAR(100) NOT NULL CHECK (reason IN (
        'emergency_hotfix', 'legacy_integration', 'experimental_feature',
        'third_party_constraint', 'performance_critical', 'security_patch',
        'infrastructure_limitation', 'other'
    )),
    description TEXT NOT NULL,
    gates TEXT[] NOT NULL, -- Array of quality gates being waived
    approved_by VARCHAR(255) NOT NULL,
    impact_level VARCHAR(20) NOT NULL CHECK (impact_level IN ('low', 'medium', 'high', 'critical')),
    mitigation_plan TEXT NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'revoked')),
    metadata JSONB DEFAULT '{}'::jsonb
);

CREATE INDEX idx_waivers_status ON waivers(status);
CREATE INDEX idx_waivers_expires_at ON waivers(expires_at);
CREATE INDEX idx_waivers_approved_by ON waivers(approved_by);
CREATE INDEX idx_waivers_impact_level ON waivers(impact_level);
CREATE INDEX idx_waivers_reason ON waivers(reason);
COMMENT ON TABLE waivers IS 'Quality gate waivers with approval workflow and expiration';

-- ============================================================================
-- HELPER FUNCTIONS
-- ============================================================================

-- Function to update task state and log audit event
CREATE OR REPLACE FUNCTION update_task_state(
    p_task_id UUID,
    p_new_state VARCHAR(50),
    p_actor VARCHAR(255) DEFAULT NULL,
    p_reason JSONB DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    v_old_state VARCHAR(50);
BEGIN
    -- Get current state
    SELECT state INTO v_old_state FROM tasks WHERE id = p_task_id;

    IF v_old_state IS NULL THEN
        RAISE EXCEPTION 'Task % not found', p_task_id;
    END IF;

    -- Update task state and timestamp
    UPDATE tasks
    SET state = p_new_state, updated_at = NOW()
    WHERE id = p_task_id;

    -- Log audit event
    INSERT INTO audit_logs (action, actor, resource_id, resource_type, change_summary)
    VALUES (
        'task_state_changed',
        COALESCE(p_actor, 'system'),
        p_task_id,
        'task',
        jsonb_build_object(
            'old_state', v_old_state,
            'new_state', p_new_state,
            'reason', p_reason
        )
    );

    RETURN TRUE;
EXCEPTION
    WHEN OTHERS THEN
        RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Function to log audit event
CREATE OR REPLACE FUNCTION log_audit_event(
    p_action VARCHAR(255),
    p_actor VARCHAR(255),
    p_resource_id UUID DEFAULT NULL,
    p_resource_type VARCHAR(50) DEFAULT NULL,
    p_change_summary JSONB DEFAULT NULL
)
RETURNS UUID AS $$
DECLARE
    v_audit_id UUID;
BEGIN
    INSERT INTO audit_logs (
        action, actor, resource_id, resource_type, change_summary
    ) VALUES (
        p_action,
        p_actor,
        p_resource_id,
        p_resource_type,
        COALESCE(p_change_summary, '{}'::jsonb)
    )
    RETURNING id INTO v_audit_id;

    RETURN v_audit_id;
END;
$$ LANGUAGE plpgsql;

-- Function to get audit trail for a resource
CREATE OR REPLACE FUNCTION get_audit_trail(
    p_resource_id UUID,
    p_limit INTEGER DEFAULT 50
)
RETURNS TABLE(
    id UUID,
    action VARCHAR(255),
    actor VARCHAR(255),
    change_summary JSONB,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        al.id,
        al.action,
        al.actor,
        al.change_summary,
        al.created_at
    FROM audit_logs al
    WHERE al.resource_id = p_resource_id
    ORDER BY al.created_at DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to retrieve chat history for a session
CREATE OR REPLACE FUNCTION get_chat_history(
    p_session_id UUID,
    p_limit INTEGER DEFAULT 100
)
RETURNS TABLE(
    id UUID,
    role VARCHAR(50),
    content TEXT,
    created_at TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        cm.id,
        cm.role,
        cm.content,
        cm.created_at
    FROM chat_messages cm
    WHERE cm.session_id = p_session_id
    ORDER BY cm.created_at ASC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON FUNCTION update_task_state(UUID, VARCHAR(50), VARCHAR(255), JSONB) IS
'Update task state and automatically log audit event with reason';

COMMENT ON FUNCTION log_audit_event(VARCHAR(255), VARCHAR(255), UUID, VARCHAR(50), JSONB) IS
'Log an audit event for any action or state change';

COMMENT ON FUNCTION get_audit_trail(UUID, INTEGER) IS
'Retrieve audit trail for a specific resource (task, waiver, etc.)';

COMMENT ON FUNCTION get_chat_history(UUID, INTEGER) IS
'Retrieve chat message history for a session in chronological order';

-- Function to create a waiver with validation
CREATE OR REPLACE FUNCTION create_waiver(
    p_title VARCHAR(255),
    p_reason VARCHAR(100),
    p_description TEXT,
    p_gates TEXT[],
    p_approved_by VARCHAR(255),
    p_impact_level VARCHAR(20),
    p_mitigation_plan TEXT,
    p_expires_at TIMESTAMPTZ,
    p_metadata JSONB DEFAULT '{}'::jsonb
)
RETURNS UUID AS $$
DECLARE
    v_waiver_id UUID;
BEGIN
    -- Validate reason
    IF p_reason NOT IN (
        'emergency_hotfix', 'legacy_integration', 'experimental_feature',
        'third_party_constraint', 'performance_critical', 'security_patch',
        'infrastructure_limitation', 'other'
    ) THEN
        RAISE EXCEPTION 'Invalid waiver reason: %', p_reason;
    END IF;
    
    -- Validate impact level
    IF p_impact_level NOT IN ('low', 'medium', 'high', 'critical') THEN
        RAISE EXCEPTION 'Invalid impact level: %', p_impact_level;
    END IF;
    
    -- Validate expiration is in the future
    IF p_expires_at <= NOW() THEN
        RAISE EXCEPTION 'Waiver expiration must be in the future';
    END IF;
    
    INSERT INTO waivers (
        title, reason, description, gates, approved_by, impact_level,
        mitigation_plan, expires_at, metadata
    )
    VALUES (
        p_title, p_reason, p_description, p_gates, p_approved_by, p_impact_level,
        p_mitigation_plan, p_expires_at, p_metadata
    )
    RETURNING id INTO v_waiver_id;
    
    -- Log the waiver creation
    PERFORM log_audit_event(
        'waiver_created',
        p_approved_by,
        v_waiver_id,
        'waiver',
        jsonb_build_object(
            'title', p_title,
            'reason', p_reason,
            'impact_level', p_impact_level,
            'expires_at', p_expires_at
        )
    );
    
    RETURN v_waiver_id;
END;
$$ LANGUAGE plpgsql;

-- Function to check if a waiver is active for specific gates
CREATE OR REPLACE FUNCTION is_waiver_active(
    p_gates TEXT[],
    p_check_time TIMESTAMPTZ DEFAULT NOW()
)
RETURNS BOOLEAN AS $$
DECLARE
    v_active_count INTEGER;
BEGIN
    SELECT COUNT(*)
    INTO v_active_count
    FROM waivers
    WHERE status = 'active'
      AND expires_at > p_check_time
      AND gates && p_gates; -- Array overlap operator
    
    RETURN v_active_count > 0;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION create_waiver(VARCHAR(255), VARCHAR(100), TEXT, TEXT[], VARCHAR(255), VARCHAR(20), TEXT, TIMESTAMPTZ, JSONB) IS
'Create a new waiver with validation and automatic audit logging';

COMMENT ON FUNCTION is_waiver_active(TEXT[], TIMESTAMPTZ) IS
'Check if any active waivers exist for the specified gates';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Create a new task
-- INSERT INTO tasks (spec) VALUES ('{"title": "My Task", "description": "Test"}'::jsonb);

-- Example 2: Update task state with audit
-- SELECT update_task_state('task-uuid', 'executing', 'user@example.com', '{"reason": "Started by user"}'::jsonb);

-- Example 3: Get audit trail for a task
-- SELECT * FROM get_audit_trail('task-uuid', 10);

-- Example 4: Create a chat session
-- INSERT INTO chat_sessions DEFAULT VALUES RETURNING id;

-- Example 5: Add message to chat
-- INSERT INTO chat_messages (session_id, role, content) VALUES ('session-uuid', 'user', 'Hello');

-- Example 6: Get chat history
-- SELECT * FROM get_chat_history('session-uuid', 50);

-- Example 7: Save a query
-- INSERT INTO saved_queries (user_id, name, query_text) VALUES ('user-uuid', 'My Query', 'SELECT * FROM tasks WHERE state = ''completed''');

-- Example 8: Create a waiver for emergency hotfix
-- SELECT create_waiver(
--     'Emergency Security Patch',
--     'emergency_hotfix',
--     'Critical security vulnerability requires immediate deployment',
--     ARRAY['test_coverage', 'mutation_testing'],
--     'security-team@company.com',
--     'critical',
--     'Deploy with enhanced monitoring and immediate rollback plan',
--     NOW() + INTERVAL '24 hours'
-- );

-- Example 9: Check if waiver is active for specific gates
-- SELECT is_waiver_active(ARRAY['test_coverage', 'mutation_testing']);

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Consider partitioning audit_logs by month for large volumes:
--    CREATE TABLE audit_logs_y2025m10 PARTITION OF audit_logs
--    FOR VALUES FROM ('2025-10-01') TO ('2025-11-01');

-- 2. Set up periodic cleanup of old chat sessions:
--    SELECT cron.schedule('cleanup-old-chats', '0 3 * * *',
--      'DELETE FROM chat_sessions WHERE ended_at < NOW() - INTERVAL ''30 days''');

-- 3. Monitor audit_logs growth and consider archival strategy:
--    CREATE TABLE audit_logs_archive PARTITION OF audit_logs
--    FOR VALUES FROM ('2024-01-01') TO ('2025-01-01');

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
