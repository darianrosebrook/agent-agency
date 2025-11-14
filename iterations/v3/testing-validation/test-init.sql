-- Initial migration for testing-validation
-- Creates basic test tables for E2E testing

CREATE TABLE IF NOT EXISTS test_research (
    id SERIAL PRIMARY KEY,
    topic TEXT NOT NULL,
    content TEXT,
    citations JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS test_code_changes (
    id SERIAL PRIMARY KEY,
    file_path TEXT NOT NULL,
    old_content TEXT,
    new_content TEXT,
    change_type TEXT,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS test_agent_runs (
    id SERIAL PRIMARY KEY,
    agent_type TEXT NOT NULL,
    task_description TEXT,
    start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    status TEXT,
    result TEXT,
    metadata JSONB
);

CREATE INDEX idx_test_research_topic ON test_research(topic);
CREATE INDEX idx_test_code_changes_file_path ON test_code_changes(file_path);
CREATE INDEX idx_test_agent_runs_agent_type ON test_agent_runs(agent_type);
CREATE INDEX idx_test_agent_runs_status ON test_agent_runs(status);
-- Migration V2: Add API Integration Tables
-- Creates tables needed for API integration tests:
-- - execution_plans: For task management endpoints
-- - saved_queries: For query management endpoints  
-- - audit_logs: For audit logging endpoints

-- Execution plans table (matches data-infrastructure schema)
CREATE TABLE IF NOT EXISTS execution_plans (
    id UUID PRIMARY KEY,
    session_id UUID NOT NULL,
    working_spec_id VARCHAR(255) NOT NULL,
    title VARCHAR(500) NOT NULL,
    overview TEXT,
    state VARCHAR(50) NOT NULL DEFAULT 'draft',
    milestones JSONB NOT NULL DEFAULT '[]'::jsonb,
    dependency_graph JSONB NOT NULL DEFAULT '{}'::jsonb,
    change_budget JSONB NOT NULL DEFAULT '{}'::jsonb,
    quality_gates JSONB NOT NULL DEFAULT '{}'::jsonb,
    evidence_requirements JSONB NOT NULL DEFAULT '[]'::jsonb,
    active_waivers JSONB NOT NULL DEFAULT '[]'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    approved_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for execution_plans
CREATE INDEX IF NOT EXISTS idx_execution_plans_session_id ON execution_plans(session_id);
CREATE INDEX IF NOT EXISTS idx_execution_plans_state ON execution_plans(state);
CREATE INDEX IF NOT EXISTS idx_execution_plans_working_spec_id ON execution_plans(working_spec_id);

-- Saved queries table (matches data-infrastructure schema)
CREATE TABLE IF NOT EXISTS saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query_sql TEXT NOT NULL,
    parameters TEXT,
    created_by VARCHAR(255) NOT NULL DEFAULT 'system',
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for saved_queries
CREATE INDEX IF NOT EXISTS idx_saved_queries_name ON saved_queries(name);
CREATE INDEX IF NOT EXISTS idx_saved_queries_created_by ON saved_queries(created_by);
CREATE INDEX IF NOT EXISTS idx_saved_queries_is_public ON saved_queries(is_public);

-- Audit logs table (matches data-infrastructure schema)
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for audit_logs
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_data ON audit_logs USING GIN(event_data);










