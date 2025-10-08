-- Migration: Create Core Database Schema
-- Created: 2025-01-08T11:00:00.000Z

-- Up migration

-- Enable necessary extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

CREATE EXTENSION IF NOT EXISTS "vector";

-- Create tenants table for multi-tenancy
CREATE TABLE tenants (
    id VARCHAR(255) PRIMARY KEY,
    project_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,
    isolation_level VARCHAR(50) NOT NULL CHECK (isolation_level IN ('strict', 'shared', 'federated')),
    access_policies JSONB DEFAULT '[]'::jsonb,
    sharing_rules JSONB DEFAULT '[]'::jsonb,
    data_retention JSONB NOT NULL,
    encryption_enabled BOOLEAN DEFAULT false,
    audit_logging BOOLEAN DEFAULT true,
    config JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create Row Level Security policies for tenants
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;

-- Create agents table
CREATE TABLE agents (
    id VARCHAR(255) PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    type VARCHAR(50) NOT NULL CHECK (type IN ('worker', 'coordinator', 'specialist')),
    capabilities TEXT[] DEFAULT '{}',
    status VARCHAR(50) NOT NULL CHECK (status IN ('active', 'inactive', 'maintenance')),
    config JSONB DEFAULT '{}'::jsonb,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for agents
CREATE INDEX idx_agents_tenant_id ON agents (tenant_id);

CREATE INDEX idx_agents_status ON agents (status);

CREATE INDEX idx_agents_type ON agents(type);

CREATE INDEX idx_agents_capabilities ON agents USING GIN (capabilities);

-- Enable RLS for agents
ALTER TABLE agents ENABLE ROW LEVEL SECURITY;

-- Create tasks table
CREATE TABLE tasks (
    id VARCHAR(255) PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id VARCHAR(255) REFERENCES agents(id) ON DELETE SET NULL,
    type VARCHAR(100) NOT NULL,
    status VARCHAR(50) NOT NULL CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    description TEXT,
    priority VARCHAR(20) NOT NULL DEFAULT 'normal' CHECK (priority IN ('low', 'normal', 'high')),
    payload JSONB DEFAULT '{}'::jsonb,
    result JSONB,
    error TEXT,
    requirements TEXT[] DEFAULT '{}',
    max_retries INTEGER DEFAULT 3,
    retry_count INTEGER DEFAULT 0,
    timeout INTEGER,
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Create indexes for tasks
CREATE INDEX idx_tasks_tenant_id ON tasks (tenant_id);

CREATE INDEX idx_tasks_agent_id ON tasks (agent_id);

CREATE INDEX idx_tasks_status ON tasks (status);

CREATE INDEX idx_tasks_priority ON tasks (priority);

CREATE INDEX idx_tasks_created_at ON tasks (created_at);

CREATE INDEX idx_tasks_requirements ON tasks USING GIN (requirements);

-- Enable RLS for tasks
ALTER TABLE tasks ENABLE ROW LEVEL SECURITY;

-- Create experiences table for agent learning
CREATE TABLE experiences (
    id VARCHAR(255) PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    agent_id VARCHAR(255) REFERENCES agents(id) ON DELETE SET NULL,
    task_id VARCHAR(255) REFERENCES tasks(id) ON DELETE SET NULL,
    type VARCHAR(100) NOT NULL,
    content JSONB NOT NULL,
    outcome VARCHAR(20) NOT NULL CHECK (outcome IN ('success', 'failure', 'partial')),
    relevance_score DECIMAL(3,2) NOT NULL CHECK (relevance_score >= 0 AND relevance_score <= 1),
    context_match JSONB NOT NULL,
    reasoning_path JSONB,
    temporal_relevance JSONB NOT NULL,
    weight DECIMAL(5,4) DEFAULT 1.0,
    embedding vector(384), -- Default embedding dimension
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create indexes for experiences
CREATE INDEX idx_experiences_tenant_id ON experiences (tenant_id);

CREATE INDEX idx_experiences_agent_id ON experiences (agent_id);

CREATE INDEX idx_experiences_task_id ON experiences (task_id);

CREATE INDEX idx_experiences_outcome ON experiences (outcome);

CREATE INDEX idx_experiences_relevance_score ON experiences (relevance_score);

CREATE INDEX idx_experiences_type ON experiences(type);

CREATE INDEX idx_experiences_created_at ON experiences (created_at);

-- Create vector similarity search index
CREATE INDEX idx_experiences_embedding ON experiences USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);

-- Enable RLS for experiences
ALTER TABLE experiences ENABLE ROW LEVEL SECURITY;

-- Create entities table for knowledge graph
CREATE TABLE entities (
    id VARCHAR(255) PRIMARY KEY,
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    type VARCHAR(100) NOT NULL,
    name VARCHAR(500) NOT NULL,
    properties JSONB DEFAULT '{}'::jsonb,
    embedding vector(384),
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create entity relationships table
CREATE TABLE entity_relationships (
    id VARCHAR(255) PRIMARY KEY DEFAULT gen_random_uuid()::text,
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    source_id VARCHAR(255) NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    target_id VARCHAR(255) NOT NULL REFERENCES entities(id) ON DELETE CASCADE,
    type VARCHAR(100) NOT NULL,
    properties JSONB DEFAULT '{}'::jsonb,
    weight DECIMAL(5,4) DEFAULT 1.0,
    bidirectional BOOLEAN DEFAULT false,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

-- Ensure no self-references and prevent duplicate relationships
CONSTRAINT no_self_reference CHECK (source_id != target_id),
    UNIQUE(source_id, target_id, type)
);

-- Create indexes for entities and relationships
CREATE INDEX idx_entities_tenant_id ON entities (tenant_id);

CREATE INDEX idx_entities_type ON entities(type);

CREATE INDEX idx_entities_name ON entities (name);

CREATE INDEX idx_entities_embedding ON entities USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);

CREATE INDEX idx_entity_relationships_tenant_id ON entity_relationships (tenant_id);

CREATE INDEX idx_entity_relationships_source_id ON entity_relationships (source_id);

CREATE INDEX idx_entity_relationships_target_id ON entity_relationships (target_id);

CREATE INDEX idx_entity_relationships_type ON entity_relationships(type);

-- Enable RLS for entities and relationships
ALTER TABLE entities ENABLE ROW LEVEL SECURITY;

ALTER TABLE entity_relationships ENABLE ROW LEVEL SECURITY;

-- Create updated_at trigger function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Add updated_at triggers to all tables
CREATE TRIGGER update_tenants_updated_at BEFORE UPDATE ON tenants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agents_updated_at BEFORE UPDATE ON agents FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tasks_updated_at BEFORE UPDATE ON tasks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_experiences_updated_at BEFORE UPDATE ON experiences FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_entities_updated_at BEFORE UPDATE ON entities FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_entity_relationships_updated_at BEFORE UPDATE ON entity_relationships FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Down migration

-- Drop triggers
DROP TRIGGER IF EXISTS update_tenants_updated_at ON tenants;

DROP TRIGGER IF EXISTS update_agents_updated_at ON agents;

DROP TRIGGER IF EXISTS update_tasks_updated_at ON tasks;

DROP TRIGGER IF EXISTS update_experiences_updated_at ON experiences;

DROP TRIGGER IF EXISTS update_entities_updated_at ON entities;

DROP TRIGGER IF EXISTS update_entity_relationships_updated_at ON entity_relationships;

-- Drop trigger function
DROP FUNCTION IF EXISTS update_updated_at_column ();

-- Drop tables in reverse dependency order
DROP TABLE IF EXISTS entity_relationships;

DROP TABLE IF EXISTS entities;

DROP TABLE IF EXISTS experiences;

DROP TABLE IF EXISTS tasks;

DROP TABLE IF EXISTS agents;

DROP TABLE IF EXISTS tenants;

-- Drop extensions (only if not used elsewhere)
-- DROP EXTENSION IF EXISTS "vector";
-- DROP EXTENSION IF EXISTS "pgcrypto";
-- DROP EXTENSION IF EXISTS "uuid-ossp";