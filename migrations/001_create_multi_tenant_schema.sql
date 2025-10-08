-- Multi-Tenant Memory System Database Schema
-- Migration: 001_create_multi_tenant_schema.sql
-- Description: Initial schema for multi-tenant memory system with PostgreSQL and pgvector

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgvector";

-- Create custom types
CREATE TYPE isolation_level AS ENUM ('strict', 'shared', 'federated');
CREATE TYPE archival_policy AS ENUM ('delete', 'archive', 'retain');
CREATE TYPE backup_frequency AS ENUM ('daily', 'weekly', 'monthly');
CREATE TYPE session_status AS ENUM ('forming', 'active', 'aggregating', 'completed', 'failed');
CREATE TYPE privacy_level AS ENUM ('basic', 'differential', 'secure');
CREATE TYPE aggregation_method AS ENUM ('weighted', 'consensus', 'hybrid');

-- ============================================================================
-- TENANT MANAGEMENT TABLES
-- ============================================================================

-- Projects table (top-level organizational unit)
CREATE TABLE projects (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL UNIQUE,
    description TEXT,
    owner_id VARCHAR(255) NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    settings JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}'
);

-- Tenants table (project-specific execution contexts)
CREATE TABLE tenants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL UNIQUE,
    project_id UUID NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
    name VARCHAR(255) NOT NULL,
    isolation_level isolation_level NOT NULL DEFAULT 'shared',
    encryption_enabled BOOLEAN DEFAULT false,
    audit_logging BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    settings JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}',

    CONSTRAINT unique_tenant_per_project UNIQUE (tenant_id, project_id)
);

-- Tenant access policies
CREATE TABLE tenant_access_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    resource_type VARCHAR(100) NOT NULL,
    access_level VARCHAR(50) NOT NULL CHECK (access_level IN ('read', 'write', 'share', 'federate')),
    allowed_tenants TEXT[] DEFAULT '{}',
    restrictions JSONB DEFAULT '[]',
    conditions JSONB DEFAULT '[]',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT unique_policy_per_tenant_resource UNIQUE (tenant_id, resource_type)
);

-- Tenant sharing rules (for cross-tenant data sharing)
CREATE TABLE tenant_sharing_rules (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    target_tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    resource_types TEXT[] NOT NULL DEFAULT '{}',
    conditions JSONB DEFAULT '[]',
    anonymization_level VARCHAR(50) DEFAULT 'none',
    active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT different_tenants CHECK (source_tenant_id != target_tenant_id)
);

-- Data retention policies
CREATE TABLE data_retention_policies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    default_retention_days INTEGER NOT NULL DEFAULT 30,
    archival_policy archival_policy DEFAULT 'delete',
    compliance_requirements TEXT[] DEFAULT '{}',
    backup_frequency backup_frequency DEFAULT 'weekly',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT positive_retention CHECK (default_retention_days > 0)
);

-- ============================================================================
-- MEMORY MANAGEMENT TABLES
-- ============================================================================

-- Contextual memories (core memory storage)
CREATE TABLE contextual_memories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    memory_id VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    relevance_score DECIMAL(3,2) NOT NULL CHECK (relevance_score >= 0 AND relevance_score <= 1),
    context_match JSONB NOT NULL DEFAULT '{}',
    reasoning_path JSONB,
    temporal_relevance JSONB,
    content JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    access_count INTEGER DEFAULT 0,
    last_accessed TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}',

    CONSTRAINT unique_memory_per_tenant UNIQUE (memory_id, tenant_id),
    CONSTRAINT valid_relevance_score CHECK (relevance_score >= 0 AND relevance_score <= 1)
);

-- Offloaded contexts (compressed/externally stored contexts)
CREATE TABLE offloaded_contexts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    context_id VARCHAR(255) NOT NULL,
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    original_context JSONB NOT NULL,
    summarized_context JSONB NOT NULL DEFAULT '{}',
    embedding VECTOR(384), -- Default embedding dimension
    compression_ratio DECIMAL(5,4) CHECK (compression_ratio >= 0 AND compression_ratio <= 1),
    retrieval_metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    last_accessed TIMESTAMP WITH TIME ZONE,
    access_count INTEGER DEFAULT 0,
    expires_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}',

    CONSTRAINT unique_context_per_tenant UNIQUE (context_id, tenant_id)
);

-- Memory relationships (for knowledge graphs)
CREATE TABLE memory_relationships (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_memory_id UUID NOT NULL REFERENCES contextual_memories(id) ON DELETE CASCADE,
    target_memory_id UUID NOT NULL REFERENCES contextual_memories(id) ON DELETE CASCADE,
    relationship_type VARCHAR(100) NOT NULL,
    strength DECIMAL(3,2) NOT NULL DEFAULT 1.0 CHECK (strength >= 0 AND strength <= 1),
    properties JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT different_memories CHECK (source_memory_id != target_memory_id),
    CONSTRAINT valid_strength CHECK (strength >= 0 AND strength <= 1)
);

-- ============================================================================
-- FEDERATED LEARNING TABLES
-- ============================================================================

-- Federated learning participants
CREATE TABLE federated_participants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,
    contribution_weight DECIMAL(3,2) NOT NULL DEFAULT 1.0 CHECK (contribution_weight > 0),
    privacy_level privacy_level NOT NULL DEFAULT 'basic',
    reputation_score DECIMAL(3,2) NOT NULL DEFAULT 1.0 CHECK (reputation_score >= 0 AND reputation_score <= 1),
    active BOOLEAN DEFAULT true,
    last_contribution TIMESTAMP WITH TIME ZONE,
    total_contributions INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),

    CONSTRAINT unique_participant_per_tenant UNIQUE (tenant_id)
);

-- Federated learning sessions
CREATE TABLE federated_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id VARCHAR(255) NOT NULL UNIQUE,
    topic VARCHAR(500) NOT NULL,
    status session_status NOT NULL DEFAULT 'forming',
    initiator_tenant_id UUID NOT NULL REFERENCES tenants(id),
    privacy_metrics JSONB DEFAULT '{}',
    convergence_score DECIMAL(5,4) DEFAULT 0 CHECK (convergence_score >= 0 AND convergence_score <= 1),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    settings JSONB DEFAULT '{}',
    metadata JSONB DEFAULT '{}'
);

-- Session participants
CREATE TABLE session_participants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES federated_sessions(id) ON DELETE CASCADE,
    participant_id UUID NOT NULL REFERENCES federated_participants(id) ON DELETE CASCADE,
    joined_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    contribution_count INTEGER DEFAULT 0,
    reputation_at_join DECIMAL(3,2),

    CONSTRAINT unique_participant_per_session UNIQUE (session_id, participant_id)
);

-- Aggregated insights from federated learning
CREATE TABLE aggregated_insights (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID NOT NULL REFERENCES federated_sessions(id) ON DELETE CASCADE,
    topic_key VARCHAR(500) NOT NULL,
    aggregated_content JSONB NOT NULL DEFAULT '{}',
    confidence_score DECIMAL(3,2) NOT NULL CHECK (confidence_score >= 0 AND confidence_score <= 1),
    source_participants INTEGER NOT NULL DEFAULT 1,
    aggregation_method aggregation_method NOT NULL DEFAULT 'weighted',
    privacy_preserved BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    expires_at TIMESTAMP WITH TIME ZONE,
    access_count INTEGER DEFAULT 0,
    metadata JSONB DEFAULT '{}'
);

-- ============================================================================
-- AUDIT AND MONITORING TABLES
-- ============================================================================

-- Audit log for all operations
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE SET NULL,
    operation VARCHAR(100) NOT NULL,
    resource_type VARCHAR(100),
    resource_id VARCHAR(255),
    user_id VARCHAR(255),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    success BOOLEAN NOT NULL DEFAULT true,
    details JSONB DEFAULT '{}',
    ip_address INET,
    user_agent TEXT,
    session_id VARCHAR(255)
);

-- Performance metrics
CREATE TABLE performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id UUID REFERENCES tenants(id) ON DELETE CASCADE,
    metric_type VARCHAR(100) NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    value DECIMAL(15,6),
    unit VARCHAR(50),
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    context JSONB DEFAULT '{}'
);

-- System health monitoring
CREATE TABLE system_health (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    component VARCHAR(100) NOT NULL,
    metric_name VARCHAR(255) NOT NULL,
    value DECIMAL(15,6),
    status VARCHAR(50) DEFAULT 'healthy',
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    details JSONB DEFAULT '{}'
);

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- Tenant and project indexes
CREATE INDEX idx_tenants_tenant_id ON tenants(tenant_id);
CREATE INDEX idx_tenants_project_id ON tenants(project_id);
CREATE INDEX idx_projects_owner_id ON projects(owner_id);

-- Memory indexes
CREATE INDEX idx_contextual_memories_tenant_id ON contextual_memories(tenant_id);
CREATE INDEX idx_contextual_memories_relevance ON contextual_memories(relevance_score DESC);
CREATE INDEX idx_contextual_memories_created ON contextual_memories(created_at DESC);
CREATE INDEX idx_contextual_memories_expires ON contextual_memories(expires_at) WHERE expires_at IS NOT NULL;

-- Vector similarity search index for embeddings
CREATE INDEX idx_offloaded_contexts_embedding ON offloaded_contexts USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX idx_offloaded_contexts_tenant_id ON offloaded_contexts(tenant_id);
CREATE INDEX idx_offloaded_contexts_context_id ON offloaded_contexts(context_id);

-- Relationship indexes
CREATE INDEX idx_memory_relationships_source ON memory_relationships(source_memory_id);
CREATE INDEX idx_memory_relationships_target ON memory_relationships(target_memory_id);
CREATE INDEX idx_memory_relationships_type ON memory_relationships(relationship_type);

-- Federated learning indexes
CREATE INDEX idx_federated_participants_tenant ON federated_participants(tenant_id);
CREATE INDEX idx_federated_sessions_status ON federated_sessions(status);
CREATE INDEX idx_federated_sessions_topic ON federated_sessions(topic);
CREATE INDEX idx_session_participants_session ON session_participants(session_id);
CREATE INDEX idx_aggregated_insights_session ON aggregated_insights(session_id);
CREATE INDEX idx_aggregated_insights_topic ON aggregated_insights(topic_key);

-- Audit and monitoring indexes
CREATE INDEX idx_audit_log_tenant_id ON audit_log(tenant_id);
CREATE INDEX idx_audit_log_operation ON audit_log(operation);
CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_performance_metrics_tenant ON performance_metrics(tenant_id);
CREATE INDEX idx_performance_metrics_type ON performance_metrics(metric_type);
CREATE INDEX idx_performance_metrics_timestamp ON performance_metrics(timestamp DESC);

-- ============================================================================
-- TRIGGERS FOR AUTOMATIC UPDATES
-- ============================================================================

-- Update timestamps on row changes
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply timestamp triggers
CREATE TRIGGER update_projects_updated_at BEFORE UPDATE ON projects FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_contextual_memories_updated_at BEFORE UPDATE ON contextual_memories FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_memory_relationships_updated_at BEFORE UPDATE ON memory_relationships FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_federated_participants_updated_at BEFORE UPDATE ON federated_participants FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Update last_accessed on tenant access
CREATE OR REPLACE FUNCTION update_tenant_last_accessed()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE tenants SET last_accessed = NOW() WHERE id = NEW.tenant_id;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_tenant_access_time AFTER INSERT OR UPDATE ON contextual_memories FOR EACH ROW EXECUTE FUNCTION update_tenant_last_accessed();
CREATE TRIGGER update_tenant_access_time_offloaded AFTER INSERT OR UPDATE ON offloaded_contexts FOR EACH ROW EXECUTE FUNCTION update_tenant_last_accessed();

-- ============================================================================
-- ROW LEVEL SECURITY (RLS) POLICIES
-- ============================================================================

-- Enable RLS on key tables
ALTER TABLE tenants ENABLE ROW LEVEL SECURITY;
ALTER TABLE contextual_memories ENABLE ROW LEVEL SECURITY;
ALTER TABLE offloaded_contexts ENABLE ROW LEVEL SECURITY;
ALTER TABLE audit_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE performance_metrics ENABLE ROW LEVEL SECURITY;

-- Note: Specific RLS policies should be created based on your authentication system
-- These are placeholder policies that assume a current_tenant_id() function exists

-- Example RLS policies (uncomment and modify based on your auth system):
/*
-- Tenants: Users can only see tenants they belong to or have access to
CREATE POLICY tenant_isolation ON tenants
    USING (tenant_id = current_tenant_id() OR project_id IN (
        SELECT project_id FROM tenants WHERE tenant_id = current_tenant_id()
    ));

-- Memories: Strict tenant isolation
CREATE POLICY memory_isolation ON contextual_memories
    USING (tenant_id IN (
        SELECT id FROM tenants WHERE tenant_id = current_tenant_id()
    ));

-- Offloaded contexts: Strict tenant isolation
CREATE POLICY context_isolation ON offloaded_contexts
    USING (tenant_id IN (
        SELECT id FROM tenants WHERE tenant_id = current_tenant_id()
    ));

-- Audit log: Users can see their own tenant's audit logs
CREATE POLICY audit_tenant_isolation ON audit_log
    USING (tenant_id IN (
        SELECT id FROM tenants WHERE tenant_id = current_tenant_id()
    ) OR tenant_id IS NULL);
*/

-- ============================================================================
-- VIEWS FOR COMMON QUERIES
-- ============================================================================

-- Active tenant overview
CREATE VIEW tenant_overview AS
SELECT
    t.id,
    t.tenant_id,
    t.name,
    t.isolation_level,
    p.name as project_name,
    t.created_at,
    t.last_accessed,
    COUNT(cm.id) as memory_count,
    COUNT(oc.id) as offloaded_context_count
FROM tenants t
LEFT JOIN projects p ON t.project_id = p.id
LEFT JOIN contextual_memories cm ON t.id = cm.tenant_id
LEFT JOIN offloaded_contexts oc ON t.id = oc.tenant_id
GROUP BY t.id, t.tenant_id, t.name, t.isolation_level, p.name, t.created_at, t.last_accessed;

-- Memory usage statistics
CREATE VIEW memory_usage_stats AS
SELECT
    t.tenant_id,
    COUNT(cm.id) as total_memories,
    AVG(cm.relevance_score) as avg_relevance,
    SUM(cm.access_count) as total_accesses,
    COUNT(oc.id) as offloaded_contexts,
    AVG(oc.compression_ratio) as avg_compression_ratio,
    MAX(cm.created_at) as latest_memory_date
FROM tenants t
LEFT JOIN contextual_memories cm ON t.id = cm.tenant_id
LEFT JOIN offloaded_contexts oc ON t.id = oc.tenant_id
GROUP BY t.id, t.tenant_id;

-- Federated learning statistics
CREATE VIEW federated_learning_stats AS
SELECT
    t.tenant_id,
    fp.reputation_score,
    fp.total_contributions,
    COUNT(sp.id) as active_sessions,
    AVG(fs.convergence_score) as avg_convergence
FROM tenants t
LEFT JOIN federated_participants fp ON t.id = fp.tenant_id
LEFT JOIN session_participants sp ON fp.id = sp.participant_id
LEFT JOIN federated_sessions fs ON sp.session_id = fs.id
WHERE fs.status IN ('active', 'aggregating')
GROUP BY t.id, t.tenant_id, fp.reputation_score, fp.total_contributions;

-- ============================================================================
-- INITIAL DATA (Optional seed data)
-- ============================================================================

-- Insert default system project (uncomment if needed)
/*
INSERT INTO projects (name, description, owner_id) VALUES
('system', 'System-level project for internal operations', 'system');
*/

-- ============================================================================
-- MIGRATION METADATA
-- ============================================================================

-- Record this migration
CREATE TABLE schema_migrations (
    version VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    executed_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    checksum VARCHAR(255)
);

INSERT INTO schema_migrations (version, name, checksum) VALUES
('001', 'create_multi_tenant_schema', 'initial_schema_v1');

COMMIT;
