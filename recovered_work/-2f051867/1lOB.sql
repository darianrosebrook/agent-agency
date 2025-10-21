-- Performance optimization indexes for Agent Agency V3
-- Created: 2025-01-20
-- Purpose: Improve query performance based on observed usage patterns

-- Task status filtering (frequently used in dashboards and monitoring)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_status ON tasks (status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_risk_tier ON tasks (risk_tier);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_created_at ON tasks (created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_priority ON tasks (priority DESC) WHERE priority IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_assigned_worker ON tasks (assigned_worker_id) WHERE assigned_worker_id IS NOT NULL;

-- Task execution performance monitoring
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_task_id ON task_executions (task_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_worker_id ON task_executions (worker_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_status ON task_executions (status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_started_at ON task_executions (execution_started_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_completed_at ON task_executions (execution_completed_at DESC) WHERE execution_completed_at IS NOT NULL;

-- Council verdict lookups (critical for decision making)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_council_verdicts_task_id ON council_verdicts (task_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_council_verdicts_created_at ON council_verdicts (created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_council_verdicts_consensus_score ON council_verdicts (consensus_score DESC);

-- Judge evaluation performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judge_evaluations_verdict_id ON judge_evaluations (verdict_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judge_evaluations_judge_id ON judge_evaluations (judge_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judge_evaluations_evaluation_time ON judge_evaluations (evaluation_time_ms DESC);

-- Knowledge base search optimization
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_relevance_score ON knowledge_entries (relevance_score DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_created_at ON knowledge_entries (created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_content_type ON knowledge_entries (content_type) WHERE content_type IS NOT NULL;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_source ON knowledge_entries (source);

-- Vector search optimization (for pgvector)
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_embedding ON knowledge_entries USING ivfflat (embedding_vector vector_cosine_ops) WITH (lists = 100);

-- Judge availability and performance
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judges_is_active ON judges (is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judges_weight ON judges (weight DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_judges_optimization_target ON judges (optimization_target);

-- Worker management
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workers_is_active ON workers (is_active) WHERE is_active = true;
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workers_worker_type ON workers (worker_type);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_workers_specialty ON workers (specialty) WHERE specialty IS NOT NULL;

-- Debate session tracking
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_debate_sessions_task_id ON debate_sessions (task_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_debate_sessions_status ON debate_sessions (status);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_debate_sessions_resolved_at ON debate_sessions (resolved_at DESC) WHERE resolved_at IS NOT NULL;

-- Composite indexes for common query patterns
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_status_created ON tasks (status, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_task_executions_status_started ON task_executions (status, execution_started_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_entries_relevance_created ON knowledge_entries (relevance_score DESC, created_at DESC);

-- Partial indexes for active records only
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_active_tasks ON tasks (id, title, status, priority) WHERE status NOT IN ('completed', 'cancelled');
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_active_executions ON task_executions (id, task_id, status, execution_started_at) WHERE status NOT IN ('completed', 'failed');

-- Text search optimization (if using full-text search)
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_content_fts ON knowledge_entries USING gin(to_tsvector('english', content));
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_description_fts ON tasks USING gin(to_tsvector('english', description));

-- JSONB indexes for metadata searches (if needed)
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_tasks_metadata ON tasks USING gin(metadata) WHERE metadata IS NOT NULL;
-- CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_knowledge_metadata ON knowledge_entries USING gin(metadata) WHERE metadata IS NOT NULL;

COMMENT ON INDEX idx_tasks_status IS 'Optimizes task status filtering in dashboards and monitoring';
COMMENT ON INDEX idx_tasks_created_at IS 'Optimizes task listing by creation time (most common sort)';
COMMENT ON INDEX idx_task_executions_started_at IS 'Optimizes execution timeline queries';
COMMENT ON INDEX idx_council_verdicts_task_id IS 'Critical index for verdict lookups during task processing';
COMMENT ON INDEX idx_judge_evaluations_evaluation_time IS 'Optimizes judge performance analysis';
COMMENT ON INDEX idx_knowledge_entries_relevance_score IS 'Optimizes knowledge base search ranking';
COMMENT ON INDEX idx_knowledge_entries_embedding IS 'Optimizes vector similarity search using pgvector';
