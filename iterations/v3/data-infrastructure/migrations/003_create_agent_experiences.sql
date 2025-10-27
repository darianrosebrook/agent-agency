-- Create agent experiences table for memory system foundation
-- Migration: 003_create_agent_experiences.sql

-- Create the base agent_experiences table that memory_embeddings references
CREATE TABLE IF NOT EXISTS agent_experiences (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id VARCHAR(255) NOT NULL,
    task_id VARCHAR(255),
    input TEXT NOT NULL,
    output TEXT,
    context JSONB DEFAULT '{}',
    outcome JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    execution_time_ms INTEGER,
    success BOOLEAN DEFAULT FALSE,
    error_message TEXT,
    workspace_id UUID NULL, -- NULL = global experience, UUID = workspace-scoped
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_agent_experiences_agent_id ON agent_experiences(agent_id);
CREATE INDEX IF NOT EXISTS idx_agent_experiences_task_id ON agent_experiences(task_id);
CREATE INDEX IF NOT EXISTS idx_agent_experiences_timestamp ON agent_experiences(timestamp);
CREATE INDEX IF NOT EXISTS idx_agent_experiences_workspace ON agent_experiences(workspace_id);
CREATE INDEX IF NOT EXISTS idx_agent_experiences_success ON agent_experiences(success);
CREATE INDEX IF NOT EXISTS idx_agent_experiences_context ON agent_experiences USING gin(context);
CREATE INDEX IF NOT EXISTS idx_agent_experiences_outcome ON agent_experiences USING gin(outcome);

-- Create migration log table if it doesn't exist
CREATE TABLE IF NOT EXISTS migration_log (
    version VARCHAR(10) PRIMARY KEY,
    description TEXT NOT NULL,
    applied_at TIMESTAMPTZ DEFAULT NOW()
);

-- Log the migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('003', 'Create agent experiences table', NOW())
ON CONFLICT (version) DO NOTHING;
