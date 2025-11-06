-- Add Multi-Tenant Isolation with Row Level Security
-- Migration: 011_add_multi_tenant_isolation.sql
-- Author: @darianrosebrook
--
-- This migration adds:
-- - Tenant management infrastructure (UUID-based)
-- - Tenant columns to existing v3 tables
-- - Row Level Security (RLS) policies
-- - Privacy configuration for federated learning
-- - Access control and audit logging

BEGIN;

-- ============================================================================
-- CUSTOM TYPES FOR MULTI-TENANCY
-- ============================================================================

-- Tenant isolation levels
DO $$ BEGIN
    CREATE TYPE isolation_level AS ENUM (
        'strict',      -- No data sharing between tenants
        'shared',      -- Explicit sharing rules apply
        'federated'    -- Cross-tenant learning allowed with privacy preservation
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Privacy levels for federated learning
DO $$ BEGIN
    CREATE TYPE privacy_level AS ENUM (
        'basic',        -- Basic anonymization
        'differential', -- Differential privacy with noise
        'secure'        -- Secure multi-party computation
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- Data retention policies
DO $$ BEGIN
    CREATE TYPE retention_policy AS ENUM (
        'delete',   -- Delete after retention period
        'archive',  -- Move to cold storage
        'retain'    -- Keep indefinitely
    );
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- ============================================================================
-- TENANT MANAGEMENT TABLES
-- ============================================================================

-- Store tenant information (UUID-based for consistency with workspace_id)
CREATE TABLE IF NOT EXISTS tenants (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_key VARCHAR(255) NOT NULL UNIQUE, -- Human-readable identifier
    project_id UUID,
    name VARCHAR(255) NOT NULL,
    
    -- Isolation configuration
    isolation_level isolation_level NOT NULL DEFAULT 'strict',
    
    -- Access control
    access_policies JSONB DEFAULT '[]'::jsonb,
    sharing_rules JSONB DEFAULT '[]'::jsonb,
    
    -- Data retention
    data_retention JSONB NOT NULL DEFAULT '{
        "policy": "archive",
        "retention_days": 730,
        "archive_after_days": 90
    }'::jsonb,
    
    -- Security settings
    encryption_enabled BOOLEAN DEFAULT false,
    audit_logging BOOLEAN DEFAULT true,
    
    -- Configuration
    config JSONB DEFAULT '{}'::jsonb,
    
    -- Temporal
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for tenants
CREATE INDEX IF NOT EXISTS idx_tenants_key ON tenants(tenant_key);
CREATE INDEX IF NOT EXISTS idx_tenants_project ON tenants(project_id);
CREATE INDEX IF NOT EXISTS idx_tenants_isolation ON tenants(isolation_level);

-- ============================================================================
-- PRIVACY CONFIGURATION FOR FEDERATED LEARNING
-- ============================================================================

CREATE TABLE IF NOT EXISTS tenant_privacy_config (
    tenant_id UUID PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,
    
    -- Privacy level
    privacy_level privacy_level NOT NULL DEFAULT 'differential',
    
    -- Differential privacy parameters
    noise_magnitude DECIMAL(5, 4) DEFAULT 0.01 CHECK (noise_magnitude >= 0.0),
    k_anonymity INTEGER DEFAULT 5 CHECK (k_anonymity >= 2),
    epsilon DECIMAL(5, 4) DEFAULT 1.0 CHECK (epsilon > 0.0),
    delta DECIMAL(10, 8) DEFAULT 0.00001 CHECK (delta >= 0.0),
    
    -- Data sharing preferences
    allow_cross_tenant_learning BOOLEAN DEFAULT false,
    allowed_tenant_groups UUID[] DEFAULT '{}',
    
    -- Temporal
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- ADD TENANT_ID TO EXISTING V3 TABLES
-- ============================================================================

-- Add tenant_id to agent_experiences
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'agent_experiences' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE agent_experiences ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add tenant_id to memory_embeddings
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'memory_embeddings' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE memory_embeddings ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add tenant_id to knowledge_graph_entities
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'knowledge_graph_entities' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE knowledge_graph_entities ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add tenant_id to agent_contexts
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'agent_contexts' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE agent_contexts ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add tenant_id to offloaded_contexts
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'offloaded_contexts' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE offloaded_contexts ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- Add tenant_id to chat_sessions (already has it, but ensure it exists)
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'chat_sessions' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE chat_sessions ADD COLUMN tenant_id UUID;
    END IF;
END $$;

-- ============================================================================
-- CREATE DEFAULT TENANT
-- ============================================================================

-- Create default tenant if it doesn't exist
INSERT INTO tenants (id, tenant_key, name, isolation_level)
VALUES (
    '00000000-0000-0000-0000-000000000000'::UUID,
    'default',
    'Default Tenant',
    'strict'
)
ON CONFLICT (tenant_key) DO NOTHING;

-- Backfill tenant_id for existing records (set to default tenant)
UPDATE agent_experiences
SET tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
WHERE tenant_id IS NULL;

UPDATE memory_embeddings
SET tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
WHERE tenant_id IS NULL AND workspace_id IS NOT NULL;

UPDATE knowledge_graph_entities
SET tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
WHERE tenant_id IS NULL AND workspace_id IS NOT NULL;

UPDATE agent_contexts
SET tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
WHERE tenant_id IS NULL AND workspace_id IS NOT NULL;

UPDATE offloaded_contexts
SET tenant_id = '00000000-0000-0000-0000-000000000000'::UUID
WHERE tenant_id IS NULL AND workspace_id IS NOT NULL;

-- ============================================================================
-- FOREIGN KEY CONSTRAINTS
-- ============================================================================

-- Add foreign key constraints (optional, for strict referential integrity)
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_agent_experiences_tenant'
    ) THEN
        ALTER TABLE agent_experiences
        ADD CONSTRAINT fk_agent_experiences_tenant 
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE SET NULL;
    END IF;
END $$;

DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_memory_embeddings_tenant'
    ) THEN
        ALTER TABLE memory_embeddings
        ADD CONSTRAINT fk_memory_embeddings_tenant 
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE SET NULL;
    END IF;
END $$;

DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.table_constraints 
        WHERE constraint_name = 'fk_chat_sessions_tenant'
    ) THEN
        ALTER TABLE chat_sessions
        ADD CONSTRAINT fk_chat_sessions_tenant 
        FOREIGN KEY (tenant_id) REFERENCES tenants(id) ON DELETE SET NULL;
    END IF;
END $$;

-- ============================================================================
-- INDEXES FOR TENANT_ID
-- ============================================================================

CREATE INDEX IF NOT EXISTS idx_agent_experiences_tenant ON agent_experiences(tenant_id);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_tenant ON memory_embeddings(tenant_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_graph_entities_tenant ON knowledge_graph_entities(tenant_id);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_tenant ON agent_contexts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_offloaded_contexts_tenant ON offloaded_contexts(tenant_id);
CREATE INDEX IF NOT EXISTS idx_chat_sessions_tenant ON chat_sessions(tenant_id);

-- Composite indexes for workspace + tenant queries
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_workspace_tenant ON memory_embeddings(workspace_id, tenant_id);
CREATE INDEX IF NOT EXISTS idx_agent_contexts_workspace_tenant ON agent_contexts(workspace_id, tenant_id);

-- ============================================================================
-- ROW LEVEL SECURITY POLICIES
-- ============================================================================

-- Enable RLS on tenant-scoped tables
ALTER TABLE agent_experiences ENABLE ROW LEVEL SECURITY;
ALTER TABLE memory_embeddings ENABLE ROW LEVEL SECURITY;
ALTER TABLE knowledge_graph_entities ENABLE ROW LEVEL SECURITY;
ALTER TABLE agent_contexts ENABLE ROW LEVEL SECURITY;
ALTER TABLE offloaded_contexts ENABLE ROW LEVEL SECURITY;
ALTER TABLE chat_sessions ENABLE ROW LEVEL SECURITY;

-- Policy for strict isolation mode (default)
CREATE POLICY tenant_strict_isolation_memory ON memory_embeddings
    USING (
        tenant_id = current_setting('app.current_tenant', true)::UUID
        OR tenant_id IS NULL  -- Allow global memory (workspace_id NULL)
        OR EXISTS (
            SELECT 1 FROM tenants 
            WHERE id = memory_embeddings.tenant_id 
            AND isolation_level = 'strict'
            AND id = current_setting('app.current_tenant', true)::UUID
        )
    );

-- Policy for shared isolation mode
CREATE POLICY tenant_shared_access_memory ON memory_embeddings
    USING (
        tenant_id = current_setting('app.current_tenant', true)::UUID
        OR tenant_id IS NULL  -- Global memory
        OR (
            EXISTS (
                SELECT 1 FROM tenants 
                WHERE id = memory_embeddings.tenant_id 
                AND isolation_level = 'shared'
            )
            AND tenant_id IN (
                SELECT unnest(
                    COALESCE(
                        (sharing_rules->>'allowed_tenants')::UUID[],
                        '{}'::UUID[]
                    )
                )
                FROM tenants 
                WHERE id = current_setting('app.current_tenant', true)::UUID
            )
        )
    );

-- Policy for context tables
CREATE POLICY tenant_isolation_contexts ON agent_contexts
    USING (
        tenant_id = current_setting('app.current_tenant', true)::UUID
        OR tenant_id IS NULL  -- Global contexts
    );

CREATE POLICY tenant_isolation_offloaded ON offloaded_contexts
    USING (
        tenant_id = current_setting('app.current_tenant', true)::UUID
        OR tenant_id IS NULL  -- Global offloaded contexts
    );

-- Policy for chat sessions
CREATE POLICY tenant_isolation_chat ON chat_sessions
    USING (
        tenant_id = current_setting('app.current_tenant', true)::UUID
        OR tenant_id IS NULL  -- Allow global chat sessions
    );

-- ============================================================================
-- PRIVACY-PRESERVING FUNCTIONS
-- ============================================================================

-- Function to apply differential privacy noise
CREATE OR REPLACE FUNCTION add_dp_noise(
    value DECIMAL,
    p_tenant_id UUID
) RETURNS DECIMAL AS $$
DECLARE
    config RECORD;
    noise DECIMAL;
BEGIN
    -- Get privacy config
    SELECT * INTO config 
    FROM tenant_privacy_config 
    WHERE tenant_id = p_tenant_id;
    
    -- If no config or basic privacy, return as-is
    IF NOT FOUND OR config.privacy_level = 'basic' THEN
        RETURN value;
    END IF;
    
    -- Add Laplace noise for differential privacy
    noise := config.noise_magnitude * (random() - 0.5) * 2 / NULLIF(config.epsilon, 0);
    
    RETURN value + noise;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- Function to check k-anonymity compliance
CREATE OR REPLACE FUNCTION check_k_anonymity(
    group_size INTEGER,
    p_tenant_id UUID
) RETURNS BOOLEAN AS $$
DECLARE
    config RECORD;
BEGIN
    SELECT * INTO config 
    FROM tenant_privacy_config 
    WHERE tenant_id = p_tenant_id;
    
    IF NOT FOUND THEN
        RETURN true; -- No config means no k-anonymity requirement
    END IF;
    
    RETURN group_size >= config.k_anonymity;
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- ============================================================================
-- TENANT CONTEXT FUNCTIONS
-- ============================================================================

-- Function to set current tenant context for RLS
CREATE OR REPLACE FUNCTION set_tenant_context(p_tenant_id UUID)
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant', p_tenant_id::TEXT, false);
END;
$$ LANGUAGE plpgsql;

-- Function to clear tenant context
CREATE OR REPLACE FUNCTION clear_tenant_context()
RETURNS VOID AS $$
BEGIN
    PERFORM set_config('app.current_tenant', NULL::TEXT, false);
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE tenants IS 'Tenant management with isolation levels and privacy configuration';
COMMENT ON TABLE tenant_privacy_config IS 'Privacy settings for federated learning and cross-tenant data sharing';
COMMENT ON FUNCTION set_tenant_context IS 'Set current tenant context for Row Level Security';
COMMENT ON FUNCTION add_dp_noise IS 'Apply differential privacy noise to values';

-- Log migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('011', 'add_multi_tenant_isolation', NOW())
ON CONFLICT (version) DO NOTHING;

COMMIT;

