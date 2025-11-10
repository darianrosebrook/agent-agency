-- Migration 017: Add Composite Indexes for Query Optimization
-- Adds composite indexes for frequently used query patterns to improve performance
-- Based on analysis of common WHERE clauses and ORDER BY patterns

-- ===========================================
-- SESSIONS TABLE OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE user_id = $1 AND is_active = true
-- Used by: get_user_sessions, deactivate_user_sessions
CREATE INDEX IF NOT EXISTS idx_sessions_user_id_is_active 
ON sessions(user_id, is_active) 
WHERE is_active = true;

-- Composite index for: WHERE token_hash = $1 AND expires_at > NOW() AND is_active = true
-- Used by: session validation in auth middleware
CREATE INDEX IF NOT EXISTS idx_sessions_token_hash_expires_active 
ON sessions(token_hash, expires_at, is_active) 
WHERE is_active = true AND expires_at > NOW();

-- Composite index for: WHERE expires_at < NOW() AND is_active = true
-- Used by: cleanup_expired_sessions
CREATE INDEX IF NOT EXISTS idx_sessions_expires_active 
ON sessions(expires_at, is_active) 
WHERE is_active = true;

-- ===========================================
-- PASSWORD RESET TOKENS OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE token_hash = $1 AND expires_at > NOW() AND used_at IS NULL
-- Used by: validate_password_reset_token
CREATE INDEX IF NOT EXISTS idx_password_reset_tokens_token_expires_used 
ON password_reset_tokens(token_hash, expires_at, used_at) 
WHERE expires_at > NOW() AND used_at IS NULL;

-- ===========================================
-- TASKS TABLE OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE status = $1 (commonly used with ORDER BY created_at DESC)
-- Used by: get_tasks_by_status, task monitoring
CREATE INDEX IF NOT EXISTS idx_tasks_status_created_at 
ON tasks(status, created_at DESC);

-- Composite index for: WHERE status = $1 AND assigned_worker_id = $2
-- Used by: get_worker_tasks
CREATE INDEX IF NOT EXISTS idx_tasks_status_worker 
ON tasks(status, assigned_worker_id) 
WHERE assigned_worker_id IS NOT NULL;

-- Composite index for: WHERE status = 'running' (most common status filter)
-- Partial index for active tasks
CREATE INDEX IF NOT EXISTS idx_tasks_running 
ON tasks(status, created_at DESC) 
WHERE status = 'running';

-- ===========================================
-- TASK EXECUTIONS OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE task_id = $1 ORDER BY started_at DESC
-- Used by: get_executions_by_task
CREATE INDEX IF NOT EXISTS idx_task_executions_task_started 
ON task_executions(task_id, execution_started_at DESC);

-- Composite index for: WHERE task_id = $1 AND status = $2
-- Used by: get_task_executions_by_status
CREATE INDEX IF NOT EXISTS idx_task_executions_task_status 
ON task_executions(task_id, status);

-- Composite index for: WHERE worker_id = $1 AND status = $2 ORDER BY execution_started_at DESC
-- Used by: get_worker_executions
CREATE INDEX IF NOT EXISTS idx_task_executions_worker_status_started 
ON task_executions(worker_id, status, execution_started_at DESC) 
WHERE worker_id IS NOT NULL;

-- ===========================================
-- JUDGE EVALUATIONS OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE judge_id = $1 ORDER BY created_at DESC
-- Used by: get_judge_evaluations
CREATE INDEX IF NOT EXISTS idx_judge_evaluations_judge_created 
ON judge_evaluations(judge_id, created_at DESC);

-- Composite index for: WHERE task_id = $1 ORDER BY created_at DESC
-- Used by: get_task_evaluations
CREATE INDEX IF NOT EXISTS idx_judge_evaluations_task_created 
ON judge_evaluations(verdict_id, created_at DESC);

-- ===========================================
-- COUNCIL VERDICTS OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE task_id = $1 ORDER BY created_at DESC
-- Used by: get_verdicts_by_task
CREATE INDEX IF NOT EXISTS idx_council_verdicts_task_created 
ON council_verdicts(task_id, created_at DESC);

-- Composite index for: WHERE task_id = $1 ORDER BY consensus_score DESC
-- Used by: get_best_verdict
CREATE INDEX IF NOT EXISTS idx_council_verdicts_task_consensus 
ON council_verdicts(task_id, consensus_score DESC);

-- ===========================================
-- SAVED QUERIES OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE (created_by = $1 OR is_public = true) ORDER BY updated_at DESC
-- Used by: list_saved_queries
CREATE INDEX IF NOT EXISTS idx_saved_queries_user_public_updated 
ON saved_queries(created_by, is_public, updated_at DESC);

-- Partial index for public queries
CREATE INDEX IF NOT EXISTS idx_saved_queries_public_updated 
ON saved_queries(updated_at DESC) 
WHERE is_public = true;

-- ===========================================
-- PROVENANCE ENTRIES OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE task_id = $1 ORDER BY created_at DESC
-- Used by: get_provenance_by_task
CREATE INDEX IF NOT EXISTS idx_provenance_entries_task_created 
ON provenance_entries(task_id, created_at DESC);

-- Composite index for: WHERE task_id = $1 AND action = $2
-- Used by: get_provenance_by_action
CREATE INDEX IF NOT EXISTS idx_provenance_entries_task_action 
ON provenance_entries(task_id, action);

-- ===========================================
-- AUDIT TRAIL OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE entity_type = $1 AND entity_id = $2 ORDER BY timestamp DESC
-- Used by: get_audit_trail_by_entity
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_entity_timestamp 
ON audit_trail_entries(entity_type, entity_id, created_at DESC);

-- Composite index for: WHERE user_id = $1 ORDER BY timestamp DESC
-- Used by: get_audit_trail_by_user
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_user_timestamp 
ON audit_trail_entries(user_id, created_at DESC) 
WHERE user_id IS NOT NULL;

-- ===========================================
-- CHAT SESSIONS OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE workspace_id = $1 AND archived = false ORDER BY updated_at DESC
-- Used by: list_chat_sessions
CREATE INDEX IF NOT EXISTS idx_chat_sessions_workspace_archived_updated 
ON chat_sessions(workspace_id, archived, updated_at DESC) 
WHERE archived = false;

-- Composite index for: WHERE tenant_id = $1 AND archived = false ORDER BY updated_at DESC
-- Used by: list_tenant_chat_sessions
CREATE INDEX IF NOT EXISTS idx_chat_sessions_tenant_archived_updated 
ON chat_sessions(tenant_id, archived, updated_at DESC) 
WHERE archived = false AND tenant_id IS NOT NULL;

-- ===========================================
-- CHAT MESSAGES OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE session_id = $1 ORDER BY created_at ASC
-- Used by: get_chat_messages
CREATE INDEX IF NOT EXISTS idx_chat_messages_session_created 
ON chat_messages(session_id, created_at ASC);

-- ===========================================
-- PERFORMANCE METRICS OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE entity_type = $1 AND entity_id = $2 ORDER BY recorded_at DESC
-- Used by: get_metrics_by_entity
CREATE INDEX IF NOT EXISTS idx_performance_metrics_entity_recorded 
ON performance_metrics(entity_type, entity_id, recorded_at DESC);

-- ===========================================
-- CAWS COMPLIANCE OPTIMIZATIONS
-- ===========================================

-- Composite index for: WHERE task_id = $1 ORDER BY recorded_at DESC
-- Used by: get_compliance_by_task
CREATE INDEX IF NOT EXISTS idx_caws_compliance_task_recorded 
ON caws_compliance(task_id, recorded_at DESC);

-- Composite index for: WHERE recorded_at >= $1 GROUP BY compliance_status
-- Used by: get_compliance_stats
CREATE INDEX IF NOT EXISTS idx_caws_compliance_recorded_status 
ON caws_compliance(recorded_at, compliance_status);

-- ===========================================
-- LOG MIGRATION
-- ===========================================

-- Log the migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('017', 'Add composite indexes for query optimization', NOW())
ON CONFLICT (version) DO NOTHING;

