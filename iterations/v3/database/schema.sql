-- Agent Agency V3 Database Schema
-- Simplified schema for council-based arbiter system

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE EXTENSION IF NOT EXISTS "pgvector";

-- Council and Judge Management
CREATE TABLE judges (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    name VARCHAR(255) NOT NULL UNIQUE,
    model_name VARCHAR(255) NOT NULL,
    endpoint VARCHAR(500) NOT NULL,
    weight DECIMAL(3, 2) NOT NULL DEFAULT 0.2,
    timeout_ms INTEGER NOT NULL DEFAULT 5000,
    optimization_target VARCHAR(20) NOT NULL CHECK (
        optimization_target IN ('ANE', 'GPU', 'CPU')
    ),
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Worker Pool Management
CREATE TABLE workers (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    name VARCHAR(255) NOT NULL,
    worker_type VARCHAR(50) NOT NULL CHECK (
        worker_type IN ('generalist', 'specialist')
    ),
    specialty VARCHAR(100),
    model_name VARCHAR(255) NOT NULL,
    endpoint VARCHAR(500) NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '{}',
    performance_history JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Task Management
CREATE TABLE tasks (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    risk_tier VARCHAR(10) NOT NULL CHECK (
        risk_tier IN ('Tier1', 'Tier2', 'Tier3')
    ),
    scope JSONB NOT NULL DEFAULT '{}',
    acceptance_criteria JSONB NOT NULL DEFAULT '[]',
    context JSONB NOT NULL DEFAULT '{}',
    caws_spec JSONB,
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
        status IN (
            'pending',
            'in_progress',
            'completed',
            'failed',
            'cancelled'
        )
    ),
    assigned_worker_id UUID REFERENCES workers (id),
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        completed_at TIMESTAMP
    WITH
        TIME ZONE
);

-- Task Execution Results
CREATE TABLE task_executions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    task_id UUID NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES workers (id),
    execution_started_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        execution_completed_at TIMESTAMP
    WITH
        TIME ZONE,
        execution_time_ms INTEGER,
        status VARCHAR(20) NOT NULL DEFAULT 'running' CHECK (
            status IN (
                'running',
                'completed',
                'failed',
                'timeout'
            )
        ),
        worker_output JSONB NOT NULL,
        self_assessment JSONB NOT NULL DEFAULT '{}',
        metadata JSONB NOT NULL DEFAULT '{}',
        error_message TEXT,
        tokens_used INTEGER,
        created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Council Verdicts and Consensus Results
CREATE TABLE council_verdicts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    task_id UUID NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    verdict_id UUID NOT NULL UNIQUE,
    consensus_score DECIMAL(3, 2) NOT NULL,
    final_verdict JSONB NOT NULL,
    individual_verdicts JSONB NOT NULL DEFAULT '{}',
    debate_rounds INTEGER NOT NULL DEFAULT 0,
    evaluation_time_ms INTEGER NOT NULL,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Individual Judge Evaluations
CREATE TABLE judge_evaluations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    verdict_id UUID NOT NULL REFERENCES council_verdicts (verdict_id) ON DELETE CASCADE,
    judge_id UUID NOT NULL REFERENCES judges (id),
    judge_verdict JSONB NOT NULL,
    evaluation_time_ms INTEGER NOT NULL,
    tokens_used INTEGER,
    confidence DECIMAL(3, 2),
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Debate Sessions
CREATE TABLE debate_sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    session_id UUID NOT NULL UNIQUE,
    task_id UUID NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    conflicting_judges JSONB NOT NULL DEFAULT '[]',
    rounds JSONB NOT NULL DEFAULT '[]',
    status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (
        status IN (
            'active',
            'resolved',
            'timeout',
            'failed'
        )
    ),
    final_consensus JSONB,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        resolved_at TIMESTAMP
    WITH
        TIME ZONE
);

-- Knowledge Base for Research Agent
CREATE TABLE knowledge_entries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    title VARCHAR(500) NOT NULL,
    content TEXT NOT NULL,
    source VARCHAR(255) NOT NULL,
    source_url VARCHAR(1000),
    relevance_score DECIMAL(3, 2) NOT NULL DEFAULT 0.5,
    tags JSONB NOT NULL DEFAULT '[]',
    embedding VECTOR (1536), -- OpenAI ada-002 embedding dimension
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW(),
        updated_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Performance Metrics and Analytics
CREATE TABLE performance_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    entity_type VARCHAR(50) NOT NULL CHECK (
        entity_type IN (
            'judge',
            'worker',
            'council',
            'system'
        )
    ),
    entity_id UUID NOT NULL,
    metric_name VARCHAR(100) NOT NULL,
    metric_value DECIMAL(10, 4) NOT NULL,
    metric_unit VARCHAR(20),
    metadata JSONB NOT NULL DEFAULT '{}',
    recorded_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- CAWS Compliance Tracking
CREATE TABLE caws_compliance (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    task_id UUID NOT NULL REFERENCES tasks (id) ON DELETE CASCADE,
    verdict_id UUID REFERENCES council_verdicts (verdict_id),
    compliance_score DECIMAL(3, 2) NOT NULL,
    violations JSONB NOT NULL DEFAULT '[]',
    waivers JSONB NOT NULL DEFAULT '[]',
    budget_adherence JSONB NOT NULL DEFAULT '{}',
    quality_gates JSONB NOT NULL DEFAULT '{}',
    provenance_trail JSONB NOT NULL DEFAULT '[]',
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Audit Trail for All Council Decisions
CREATE TABLE audit_trail (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    entity_type VARCHAR(50) NOT NULL,
    entity_id UUID NOT NULL,
    action VARCHAR(100) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}',
    user_id VARCHAR(255),
    ip_address INET,
    created_at TIMESTAMP
    WITH
        TIME ZONE DEFAULT NOW()
);

-- Indexes for Performance
CREATE INDEX idx_tasks_status ON tasks (status);

CREATE INDEX idx_tasks_risk_tier ON tasks (risk_tier);

CREATE INDEX idx_tasks_assigned_worker ON tasks (assigned_worker_id);

CREATE INDEX idx_tasks_created_at ON tasks (created_at);

CREATE INDEX idx_task_executions_task_id ON task_executions (task_id);

CREATE INDEX idx_task_executions_worker_id ON task_executions (worker_id);

CREATE INDEX idx_task_executions_status ON task_executions (status);

CREATE INDEX idx_task_executions_started_at ON task_executions (execution_started_at);

CREATE INDEX idx_council_verdicts_task_id ON council_verdicts (task_id);

CREATE INDEX idx_council_verdicts_consensus_score ON council_verdicts (consensus_score);

CREATE INDEX idx_council_verdicts_created_at ON council_verdicts (created_at);

CREATE INDEX idx_judge_evaluations_verdict_id ON judge_evaluations (verdict_id);

CREATE INDEX idx_judge_evaluations_judge_id ON judge_evaluations (judge_id);

CREATE INDEX idx_judge_evaluations_confidence ON judge_evaluations (confidence);

CREATE INDEX idx_debate_sessions_task_id ON debate_sessions (task_id);

CREATE INDEX idx_debate_sessions_status ON debate_sessions (status);

CREATE INDEX idx_knowledge_entries_source ON knowledge_entries (source);

CREATE INDEX idx_knowledge_entries_tags ON knowledge_entries USING GIN (tags);

CREATE INDEX idx_knowledge_entries_embedding ON knowledge_entries USING ivfflat (embedding vector_cosine_ops)
WITH (lists = 100);

CREATE INDEX idx_performance_metrics_entity ON performance_metrics (entity_type, entity_id);

CREATE INDEX idx_performance_metrics_name ON performance_metrics (metric_name);

CREATE INDEX idx_performance_metrics_recorded_at ON performance_metrics (recorded_at);

CREATE INDEX idx_caws_compliance_task_id ON caws_compliance (task_id);

CREATE INDEX idx_caws_compliance_score ON caws_compliance (compliance_score);

CREATE INDEX idx_audit_trail_entity ON audit_trail (entity_type, entity_id);

CREATE INDEX idx_audit_trail_action ON audit_trail (action);

CREATE INDEX idx_audit_trail_created_at ON audit_trail (created_at);

-- Views for Common Queries
CREATE VIEW council_metrics AS
SELECT
    DATE_TRUNC ('hour', created_at) as hour,
    COUNT(*) as total_verdicts,
    AVG(consensus_score) as avg_consensus_score,
    COUNT(
        CASE
            WHEN final_verdict ->> 'type' = 'accepted' THEN 1
        END
    ) as accepted_count,
    COUNT(
        CASE
            WHEN final_verdict ->> 'type' = 'rejected' THEN 1
        END
    ) as rejected_count,
    COUNT(
        CASE
            WHEN final_verdict ->> 'type' = 'requires_modification' THEN 1
        END
    ) as modification_required_count,
    AVG(evaluation_time_ms) as avg_evaluation_time_ms
FROM council_verdicts
GROUP BY
    DATE_TRUNC ('hour', created_at)
ORDER BY hour DESC;

CREATE VIEW judge_performance AS
SELECT
    j.name as judge_name,
    j.model_name,
    COUNT(je.id) as total_evaluations,
    AVG(je.evaluation_time_ms) as avg_evaluation_time_ms,
    AVG(je.confidence) as avg_confidence,
    COUNT(
        CASE
            WHEN je.judge_verdict ->> 'verdict' = 'pass' THEN 1
        END
    ) as pass_count,
    COUNT(
        CASE
            WHEN je.judge_verdict ->> 'verdict' = 'fail' THEN 1
        END
    ) as fail_count,
    COUNT(
        CASE
            WHEN je.judge_verdict ->> 'verdict' = 'uncertain' THEN 1
        END
    ) as uncertain_count
FROM
    judges j
    LEFT JOIN judge_evaluations je ON j.id = je.judge_id
WHERE
    j.is_active = true
GROUP BY
    j.id,
    j.name,
    j.model_name
ORDER BY total_evaluations DESC;

CREATE VIEW worker_performance AS
SELECT
    w.name as worker_name,
    w.worker_type,
    w.specialty,
    COUNT(te.id) as total_executions,
    AVG(te.execution_time_ms) as avg_execution_time_ms,
    COUNT(
        CASE
            WHEN te.status = 'completed' THEN 1
        END
    ) as completed_count,
    COUNT(
        CASE
            WHEN te.status = 'failed' THEN 1
        END
    ) as failed_count,
    AVG(te.tokens_used) as avg_tokens_used
FROM workers w
    LEFT JOIN task_executions te ON w.id = te.worker_id
WHERE
    w.is_active = true
GROUP BY
    w.id,
    w.name,
    w.worker_type,
    w.specialty
ORDER BY total_executions DESC;

-- Functions for Common Operations
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers for automatic timestamp updates
CREATE TRIGGER update_judges_updated_at BEFORE UPDATE ON judges FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_workers_updated_at BEFORE UPDATE ON workers FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_tasks_updated_at BEFORE UPDATE ON tasks FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_knowledge_entries_updated_at BEFORE UPDATE ON knowledge_entries FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to clean up old audit trail entries
CREATE OR REPLACE FUNCTION cleanup_audit_trail()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM audit_trail 
    WHERE created_at < NOW() - INTERVAL '90 days';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to get task execution summary
CREATE OR REPLACE FUNCTION get_task_execution_summary(p_task_id UUID)
RETURNS JSONB AS $$
DECLARE
    result JSONB;
BEGIN
    SELECT jsonb_build_object(
        'task_id', t.id,
        'title', t.title,
        'status', t.status,
        'risk_tier', t.risk_tier,
        'executions', COALESCE(executions.data, '[]'::jsonb),
        'verdicts', COALESCE(verdicts.data, '[]'::jsonb),
        'compliance', COALESCE(compliance.data, '{}'::jsonb)
    ) INTO result
    FROM tasks t
    LEFT JOIN (
        SELECT 
            task_id,
            jsonb_agg(
                jsonb_build_object(
                    'id', id,
                    'worker_id', worker_id,
                    'status', status,
                    'execution_time_ms', execution_time_ms,
                    'tokens_used', tokens_used,
                    'created_at', execution_started_at
                )
            ) as data
        FROM task_executions 
        WHERE task_id = p_task_id
        GROUP BY task_id
    ) executions ON t.id = executions.task_id
    LEFT JOIN (
        SELECT 
            task_id,
            jsonb_agg(
                jsonb_build_object(
                    'verdict_id', verdict_id,
                    'consensus_score', consensus_score,
                    'final_verdict', final_verdict,
                    'evaluation_time_ms', evaluation_time_ms,
                    'created_at', created_at
                )
            ) as data
        FROM council_verdicts 
        WHERE task_id = p_task_id
        GROUP BY task_id
    ) verdicts ON t.id = verdicts.task_id
    LEFT JOIN (
        SELECT 
            task_id,
            jsonb_build_object(
                'compliance_score', compliance_score,
                'violations', violations,
                'waivers', waivers,
                'budget_adherence', budget_adherence
            ) as data
        FROM caws_compliance 
        WHERE task_id = p_task_id
        LIMIT 1
    ) compliance ON t.id = compliance.task_id
    WHERE t.id = p_task_id;
    
    RETURN COALESCE(result, '{}'::jsonb);
END;
$$ LANGUAGE plpgsql;

-- Insert default judges
INSERT INTO
    judges (
        name,
        model_name,
        endpoint,
        weight,
        timeout_ms,
        optimization_target
    )
VALUES (
        'Constitutional Judge',
        'llama3.3:3b-constitutional-caws',
        'http://localhost:11434',
        0.4,
        100,
        'ANE'
    ),
    (
        'Technical Auditor',
        'codellama:7b-audit-specialist',
        'http://localhost:11434',
        0.2,
        500,
        'GPU'
    ),
    (
        'Quality Evaluator',
        'gemma2:3b-quality-judge',
        'http://localhost:11434',
        0.2,
        200,
        'CPU'
    ),
    (
        'Integration Validator',
        'mistral:3b-integration-checker',
        'http://localhost:11434',
        0.2,
        150,
        'CPU'
    );

-- Insert default workers
INSERT INTO
    workers (
        name,
        worker_type,
        specialty,
        model_name,
        endpoint,
        capabilities
    )
VALUES (
        'Generalist Worker 1',
        'generalist',
        NULL,
        'llama3.3:7b-caws-aware',
        'http://localhost:11434',
        '{"general": 0.8, "caws_compliance": 0.9}'
    ),
    (
        'TypeScript Specialist',
        'specialist',
        'typescript',
        'llama3.3:7b-typescript',
        'http://localhost:11434',
        '{"typescript": 0.95, "react": 0.9, "nodejs": 0.85}'
    ),
    (
        'Python Specialist',
        'specialist',
        'python',
        'llama3.3:7b-python',
        'http://localhost:11434',
        '{"python": 0.95, "django": 0.9, "fastapi": 0.85}'
    ),
    (
        'Database Specialist',
        'specialist',
        'database',
        'llama3.3:7b-database',
        'http://localhost:11434',
        '{"postgresql": 0.95, "migrations": 0.9, "optimization": 0.85}'
    );

-- Comments for documentation
COMMENT ON
TABLE judges IS 'Council judges with their model specifications and performance characteristics';

COMMENT ON
TABLE workers IS 'Worker pool with different specializations and capabilities';

COMMENT ON
TABLE tasks IS 'Task specifications and execution tracking';

COMMENT ON
TABLE task_executions IS 'Individual worker execution results and performance metrics';

COMMENT ON
TABLE council_verdicts IS 'Council consensus results and final verdicts';

COMMENT ON
TABLE judge_evaluations IS 'Individual judge evaluations contributing to consensus';

COMMENT ON
TABLE debate_sessions IS 'Debate sessions for resolving judge conflicts';

COMMENT ON
TABLE knowledge_entries IS 'Research agent knowledge base with vector embeddings';

COMMENT ON
TABLE performance_metrics IS 'System performance metrics and analytics';

COMMENT ON
TABLE caws_compliance IS 'CAWS compliance tracking and violation records';

COMMENT ON
TABLE audit_trail IS 'Complete audit trail for all system decisions';-- Agent Memory System Database Schema
-- Extends the base agent_agency database with memory-specific tables

-- ===========================================
-- MEMORY EMBEDDINGS
-- ===========================================

-- Store vector embeddings for semantic memory search
CREATE TABLE IF NOT EXISTS memory_embeddings (
    memory_id UUID PRIMARY KEY REFERENCES agent_experiences(id) ON DELETE CASCADE,
    embedding VECTOR(768),  -- pgvector extension for embeddings
    workspace_id UUID NULL, -- NULL = global memory, UUID = workspace-scoped
    importance_score FLOAT DEFAULT 1.0 CHECK (importance_score >= 0.0 AND importance_score <= 3.0),
    decay_factor FLOAT DEFAULT 1.0 CHECK (decay_factor >= 0.0 AND decay_factor <= 1.0),
    last_accessed TIMESTAMPTZ DEFAULT NOW(),
    access_count INTEGER DEFAULT 0,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Create vector similarity indexes
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_embedding ON memory_embeddings
USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);

-- Performance indexes
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_workspace ON memory_embeddings(workspace_id);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_importance ON memory_embeddings(importance_score);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_decay ON memory_embeddings(decay_factor);
CREATE INDEX IF NOT EXISTS idx_memory_embeddings_access ON memory_embeddings(last_accessed);

-- ===========================================
-- KNOWLEDGE GRAPH ENTITIES
-- ===========================================

-- Core entities in the knowledge graph
CREATE TABLE IF NOT EXISTS knowledge_graph_entities (
    id VARCHAR(255) PRIMARY KEY,
    workspace_id UUID NULL, -- NULL = global entity, UUID = workspace-scoped
    entity_type INTEGER NOT NULL,  -- 0=Agent, 1=Task, 2=Capability, etc.
    name VARCHAR(500) NOT NULL,
    description TEXT,
    properties JSONB DEFAULT '{}',
    embedding VECTOR(768),
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    source_memories UUID[] DEFAULT '{}'
);

-- Indexes for knowledge graph entities
CREATE INDEX IF NOT EXISTS idx_entities_workspace ON knowledge_graph_entities(workspace_id);
CREATE INDEX IF NOT EXISTS idx_entities_type ON knowledge_graph_entities(entity_type);
CREATE INDEX IF NOT EXISTS idx_entities_name ON knowledge_graph_entities(name);
CREATE INDEX IF NOT EXISTS idx_entities_embedding ON knowledge_graph_entities
USING ivfflat (embedding vector_cosine_ops) WITH (lists = 100);
CREATE INDEX IF NOT EXISTS idx_entities_confidence ON knowledge_graph_entities(confidence);
CREATE INDEX IF NOT EXISTS idx_entities_updated ON knowledge_graph_entities(updated_at);

-- ===========================================
-- KNOWLEDGE GRAPH RELATIONSHIPS
-- ===========================================

-- Relationships between entities (can span workspaces via cross-links)
CREATE TABLE IF NOT EXISTS knowledge_graph_relationships (
    id VARCHAR(255) PRIMARY KEY,
    workspace_id UUID NULL, -- NULL = global relationship, UUID = workspace-scoped
    source_entity VARCHAR(255) NOT NULL REFERENCES knowledge_graph_entities(id) ON DELETE CASCADE,
    target_entity VARCHAR(255) NOT NULL REFERENCES knowledge_graph_entities(id) ON DELETE CASCADE,
    relationship_type INTEGER NOT NULL,  -- 0=Performs, 1=Requires, etc.
    properties JSONB DEFAULT '{}',
    strength FLOAT DEFAULT 1.0 CHECK (strength >= 0.0 AND strength <= 2.0),
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    bidirectional BOOLEAN DEFAULT FALSE,
    cross_workspace BOOLEAN DEFAULT FALSE, -- True for relationships spanning workspaces
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    source_memories UUID[] DEFAULT '{}',
    CONSTRAINT different_entities CHECK (source_entity != target_entity)
);

-- Indexes for knowledge graph relationships
CREATE INDEX IF NOT EXISTS idx_relationships_workspace ON knowledge_graph_relationships(workspace_id);
CREATE INDEX IF NOT EXISTS idx_relationships_cross_workspace ON knowledge_graph_relationships(cross_workspace);
CREATE INDEX IF NOT EXISTS idx_relationships_source ON knowledge_graph_relationships(source_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_target ON knowledge_graph_relationships(target_entity);
CREATE INDEX IF NOT EXISTS idx_relationships_type ON knowledge_graph_relationships(relationship_type);
CREATE INDEX IF NOT EXISTS idx_relationships_strength ON knowledge_graph_relationships(strength);
CREATE INDEX IF NOT EXISTS idx_relationships_confidence ON knowledge_graph_relationships(confidence);
CREATE INDEX IF NOT EXISTS idx_relationships_updated ON knowledge_graph_relationships(updated_at);

-- ===========================================
-- TEMPORAL ANALYSIS RESULTS
-- ===========================================

-- Store results of temporal analysis and trends
CREATE TABLE IF NOT EXISTS temporal_analysis_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NULL, -- NULL = global analysis, UUID = workspace-scoped
    entity_type VARCHAR(50) NOT NULL,  -- 'agent', 'task', 'capability'
    entity_id VARCHAR(255) NOT NULL,
    analysis_type VARCHAR(50) NOT NULL,  -- 'trend', 'change_point', 'causality'
    time_range TSRANGE NOT NULL,
    results JSONB NOT NULL,
    confidence FLOAT DEFAULT 1.0 CHECK (confidence >= 0.0 AND confidence <= 1.0),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for temporal analysis
CREATE INDEX IF NOT EXISTS idx_temporal_workspace ON temporal_analysis_results(workspace_id);
CREATE INDEX IF NOT EXISTS idx_temporal_entity ON temporal_analysis_results(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_temporal_type ON temporal_analysis_results(analysis_type);
CREATE INDEX IF NOT EXISTS idx_temporal_range ON temporal_analysis_results(time_range);
CREATE INDEX IF NOT EXISTS idx_temporal_created ON temporal_analysis_results(created_at);

-- ===========================================
-- PROVENANCE TRACKING
-- ===========================================

-- Track memory operations for explainability
CREATE TABLE IF NOT EXISTS memory_provenance (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NULL, -- NULL = global operation, UUID = workspace-scoped
    operation VARCHAR(50) NOT NULL,  -- 'store', 'retrieve', 'search', etc.
    memory_id UUID REFERENCES agent_experiences(id) ON DELETE SET NULL,
    agent_id VARCHAR(255),
    context JSONB DEFAULT '{}',
    reasoning TEXT[],
    confidence FLOAT CHECK (confidence >= 0.0 AND confidence <= 1.0),
    processing_time_ms INTEGER,
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for provenance tracking
CREATE INDEX IF NOT EXISTS idx_provenance_workspace ON memory_provenance(workspace_id);
CREATE INDEX IF NOT EXISTS idx_provenance_operation ON memory_provenance(operation);
CREATE INDEX IF NOT EXISTS idx_provenance_memory ON memory_provenance(memory_id);
CREATE INDEX IF NOT EXISTS idx_provenance_agent ON memory_provenance(agent_id);
CREATE INDEX IF NOT EXISTS idx_provenance_timestamp ON memory_provenance(timestamp);

-- ===========================================
-- CONTEXT OFFLOADING
-- ===========================================

-- Store offloaded context for memory compression
CREATE TABLE IF NOT EXISTS offloaded_contexts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NULL, -- NULL = global offload, UUID = workspace-scoped
    original_memory_id UUID REFERENCES agent_experiences(id) ON DELETE CASCADE,
    context_type VARCHAR(50) NOT NULL,  -- 'episodic', 'semantic', 'working'
    compressed_content TEXT NOT NULL,
    compression_ratio FLOAT,
    retrieval_count INTEGER DEFAULT 0,
    last_retrieved TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for context offloading
CREATE INDEX IF NOT EXISTS idx_offloaded_workspace ON offloaded_contexts(workspace_id);
CREATE INDEX IF NOT EXISTS idx_offloaded_memory ON offloaded_contexts(original_memory_id);
CREATE INDEX IF NOT EXISTS idx_offloaded_type ON offloaded_contexts(context_type);
CREATE INDEX IF NOT EXISTS idx_offloaded_expires ON offloaded_contexts(expires_at);
CREATE INDEX IF NOT EXISTS idx_offloaded_retrieved ON offloaded_contexts(last_retrieved);

-- ===========================================
-- MEMORY SYSTEM METRICS
-- ===========================================

-- Track memory system performance and health
CREATE TABLE IF NOT EXISTS memory_system_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workspace_id UUID NULL, -- NULL = global metrics, UUID = workspace-scoped
    metric_name VARCHAR(100) NOT NULL,
    metric_value FLOAT NOT NULL,
    metric_unit VARCHAR(20),
    labels JSONB DEFAULT '{}',
    timestamp TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for metrics
CREATE INDEX IF NOT EXISTS idx_metrics_workspace ON memory_system_metrics(workspace_id);
CREATE INDEX IF NOT EXISTS idx_metrics_name ON memory_system_metrics(metric_name);
CREATE INDEX IF NOT EXISTS idx_metrics_timestamp ON memory_system_metrics(timestamp);
CREATE INDEX IF NOT EXISTS idx_metrics_labels ON memory_system_metrics USING gin(labels);

-- ===========================================
-- ENUM TYPE DEFINITIONS
-- ===========================================

-- Entity types enum (for better query performance)
DO $$ BEGIN
    CREATE TYPE entity_type AS ENUM (
        'agent', 'task', 'capability', 'domain', 'tool',
        'outcome', 'concept', 'person', 'organization', 'location', 'technology'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Relationship types enum
DO $$ BEGIN
    CREATE TYPE relationship_type AS ENUM (
        'performs', 'requires', 'enables', 'conflicts', 'improves',
        'learns_from', 'collaborates_with', 'manages', 'creates', 'uses',
        'contains', 'related_to', 'causes', 'prevents', 'similar_to'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Memory operation types enum
DO $$ BEGIN
    CREATE TYPE memory_operation AS ENUM (
        'store', 'retrieve', 'update', 'delete', 'search',
        'reason', 'consolidate', 'decay', 'offload'
    );
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ===========================================
-- VIEWS FOR COMMON QUERIES
-- ===========================================

-- View for agent performance over time (workspace-scoped)
CREATE OR REPLACE VIEW agent_performance_trends AS
SELECT
    ae.agent_id,
    me.workspace_id,
    DATE_TRUNC('day', ae.timestamp) as date,
    AVG((ae.outcome->>'performance_score')::float) as avg_performance,
    AVG((ae.outcome->>'execution_time_ms')::float) as avg_execution_time,
    COUNT(*) as experience_count,
    COUNT(CASE WHEN ae.outcome->>'success' = 'true' THEN 1 END)::float / COUNT(*)::float as success_rate
FROM agent_experiences ae
LEFT JOIN memory_embeddings me ON ae.id = me.memory_id
GROUP BY ae.agent_id, me.workspace_id, DATE_TRUNC('day', ae.timestamp)
ORDER BY ae.agent_id, me.workspace_id, date;

-- View for capability learning patterns (workspace-scoped)
CREATE OR REPLACE VIEW capability_learning_patterns AS
SELECT
    ae.agent_id,
    me.workspace_id,
    jsonb_array_elements_text(ae.outcome->'learned_capabilities') as capability,
    DATE_TRUNC('week', ae.timestamp) as week,
    COUNT(*) as learning_events,
    AVG((ae.outcome->>'performance_score')::float) as avg_performance
FROM agent_experiences ae
LEFT JOIN memory_embeddings me ON ae.id = me.memory_id
WHERE jsonb_array_length(ae.outcome->'learned_capabilities') > 0
GROUP BY ae.agent_id, me.workspace_id, capability, DATE_TRUNC('week', ae.timestamp)
ORDER BY ae.agent_id, me.workspace_id, capability, week;

-- View for memory access patterns (workspace-scoped)
CREATE OR REPLACE VIEW memory_access_patterns AS
SELECT
    me.memory_id,
    me.workspace_id,
    ae.agent_id,
    ae.context->>'task_type' as task_type,
    me.importance_score,
    me.decay_factor,
    me.access_count,
    me.last_accessed,
    AGE(NOW(), me.created_at) as memory_age
FROM memory_embeddings me
JOIN agent_experiences ae ON me.memory_id = ae.id
ORDER BY me.workspace_id, me.last_accessed DESC;

-- ===========================================
-- UTILITY FUNCTIONS
-- ===========================================

-- Function to calculate memory relevance score
CREATE OR REPLACE FUNCTION calculate_memory_relevance(
    importance FLOAT,
    decay FLOAT,
    recency_hours FLOAT,
    access_count INTEGER
) RETURNS FLOAT AS $$
BEGIN
    -- Combine importance, decay, recency, and access patterns
    RETURN importance * decay *
           GREATEST(0.5, 1.0 - (recency_hours / 168.0)) *  -- 7 day recency factor
           (1.0 + LOG(GREATEST(1, access_count)) / 10.0);  -- Access frequency bonus
END;
$$ LANGUAGE plpgsql IMMUTABLE;

-- Function to find similar memories by embedding (with workspace scoping)
CREATE OR REPLACE FUNCTION find_similar_memories(
    query_embedding VECTOR(768),
    workspace_filter UUID DEFAULT NULL, -- NULL = search all workspaces
    similarity_threshold FLOAT DEFAULT 0.7,
    max_results INTEGER DEFAULT 10
) RETURNS TABLE(
    memory_id UUID,
    workspace_id UUID,
    similarity_score FLOAT,
    importance_score FLOAT,
    relevance_score FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        me.memory_id,
        me.workspace_id,
        (1.0 - (me.embedding <=> query_embedding)) as similarity_score,
        me.importance_score,
        calculate_memory_relevance(
            me.importance_score,
            me.decay_factor,
            EXTRACT(EPOCH FROM (NOW() - me.last_accessed)) / 3600.0,
            me.access_count
        ) as relevance_score
    FROM memory_embeddings me
    WHERE (1.0 - (me.embedding <=> query_embedding)) >= similarity_threshold
      AND (workspace_filter IS NULL OR me.workspace_id = workspace_filter OR me.workspace_id IS NULL)
    ORDER BY relevance_score DESC, similarity_score DESC
    LIMIT max_results;
END;
$$ LANGUAGE plpgsql;

-- Function to update memory access statistics
CREATE OR REPLACE FUNCTION update_memory_access(memory_uuid UUID) RETURNS VOID AS $$
BEGIN
    UPDATE memory_embeddings
    SET access_count = access_count + 1,
        last_accessed = NOW()
    WHERE memory_id = memory_uuid;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- TRIGGERS FOR AUTOMATIC MAINTENANCE
-- ===========================================

-- Trigger to automatically update memory access on retrieval
CREATE OR REPLACE FUNCTION trigger_memory_access() RETURNS TRIGGER AS $$
BEGIN
    -- Update access statistics when memory is retrieved
    PERFORM update_memory_access(NEW.id);
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Note: This trigger would be added to agent_experiences table when memory system is active
-- CREATE TRIGGER memory_access_trigger AFTER SELECT ON agent_experiences
--     FOR EACH ROW EXECUTE FUNCTION trigger_memory_access();

-- ===========================================
-- CLEANUP AND MAINTENANCE
-- ===========================================

-- Function to clean up expired offloaded contexts
CREATE OR REPLACE FUNCTION cleanup_expired_contexts() RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM offloaded_contexts
    WHERE expires_at < NOW();

    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Function to apply memory decay
CREATE OR REPLACE FUNCTION apply_memory_decay_batch(
    decay_rate FLOAT DEFAULT 0.95,
    max_age_hours INTEGER DEFAULT 168
) RETURNS INTEGER AS $$
DECLARE
    updated_count INTEGER;
BEGIN
    UPDATE memory_embeddings
    SET decay_factor = GREATEST(decay_factor * decay_rate, 0.1)
    WHERE last_accessed < NOW() - (max_age_hours || ' hours')::INTERVAL
      AND decay_factor > 0.1;

    GET DIAGNOSTICS updated_count = ROW_COUNT;
    RETURN updated_count;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- PERFORMANCE OPTIMIZATION
-- ===========================================

-- Create partial indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_embeddings_high_importance ON memory_embeddings(importance_score)
WHERE importance_score > 1.5;

CREATE INDEX IF NOT EXISTS idx_embeddings_recent_access ON memory_embeddings(last_accessed)
WHERE last_accessed > NOW() - INTERVAL '7 days';

CREATE INDEX IF NOT EXISTS idx_relationships_strong ON knowledge_graph_relationships(strength)
WHERE strength > 1.2;

-- ===========================================
-- MONITORING AND HEALTH CHECKS
-- ===========================================

-- Function to get memory system health metrics (with workspace scoping)
CREATE OR REPLACE FUNCTION get_memory_system_health(workspace_filter UUID DEFAULT NULL) RETURNS JSONB AS $$
DECLARE
    result JSONB;
BEGIN
    SELECT jsonb_build_object(
        'workspace_id', workspace_filter,
        'total_memories', (SELECT COUNT(*) FROM agent_experiences ae
                          WHERE workspace_filter IS NULL OR ae.id IN (
                              SELECT me.memory_id FROM memory_embeddings me
                              WHERE me.workspace_id = workspace_filter OR me.workspace_id IS NULL
                          )),
        'embedded_memories', (SELECT COUNT(*) FROM memory_embeddings me
                             WHERE workspace_filter IS NULL OR me.workspace_id = workspace_filter OR me.workspace_id IS NULL),
        'workspace_memories', (SELECT COUNT(*) FROM memory_embeddings me
                              WHERE me.workspace_id = workspace_filter),
        'global_memories', (SELECT COUNT(*) FROM memory_embeddings me
                           WHERE me.workspace_id IS NULL),
        'knowledge_entities', (SELECT COUNT(*) FROM knowledge_graph_entities kge
                              WHERE workspace_filter IS NULL OR kge.workspace_id = workspace_filter OR kge.workspace_id IS NULL),
        'knowledge_relationships', (SELECT COUNT(*) FROM knowledge_graph_relationships kgr
                                   WHERE workspace_filter IS NULL OR kgr.workspace_id = workspace_filter OR kgr.workspace_id IS NULL),
        'cross_workspace_relationships', (SELECT COUNT(*) FROM knowledge_graph_relationships kgr
                                        WHERE kgr.cross_workspace = true),
        'avg_importance', (SELECT AVG(importance_score) FROM memory_embeddings me
                          WHERE workspace_filter IS NULL OR me.workspace_id = workspace_filter OR me.workspace_id IS NULL),
        'avg_decay', (SELECT AVG(decay_factor) FROM memory_embeddings me
                     WHERE workspace_filter IS NULL OR me.workspace_id = workspace_filter OR me.workspace_id IS NULL),
        'oldest_memory', (SELECT MIN(ae.created_at) FROM agent_experiences ae
                         WHERE workspace_filter IS NULL OR ae.id IN (
                             SELECT me.memory_id FROM memory_embeddings me
                             WHERE me.workspace_id = workspace_filter OR me.workspace_id IS NULL
                         )),
        'newest_memory', (SELECT MAX(ae.created_at) FROM agent_experiences ae
                         WHERE workspace_filter IS NULL OR ae.id IN (
                             SELECT me.memory_id FROM memory_embeddings me
                             WHERE me.workspace_id = workspace_filter OR me.workspace_id IS NULL
                         )),
        'expired_contexts', (SELECT COUNT(*) FROM offloaded_contexts oc
                           WHERE oc.expires_at < NOW()
                           AND (workspace_filter IS NULL OR oc.workspace_id = workspace_filter OR oc.workspace_id IS NULL))
    ) INTO result;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- MIGRATION HELPERS
-- ===========================================

-- Function to migrate existing experiences to memory embeddings
CREATE OR REPLACE FUNCTION migrate_experiences_to_embeddings() RETURNS INTEGER AS $$
DECLARE
    migrated_count INTEGER := 0;
    experience_record RECORD;
BEGIN
    -- Note: This would need to be called with actual embedding generation
    -- For now, just create placeholder records
    FOR experience_record IN
        SELECT id FROM agent_experiences
        WHERE id NOT IN (SELECT memory_id FROM memory_embeddings)
        LIMIT 1000  -- Batch processing
    LOOP
        INSERT INTO memory_embeddings (memory_id, embedding, importance_score, decay_factor)
        VALUES (experience_record.id, NULL, 1.0, 1.0);

        migrated_count := migrated_count + 1;
    END LOOP;

    RETURN migrated_count;
END;
$$ LANGUAGE plpgsql;

COMMENT ON TABLE memory_embeddings IS 'Vector embeddings for semantic memory search with decay and importance tracking';
COMMENT ON TABLE knowledge_graph_entities IS 'Entities in the knowledge graph (agents, tasks, capabilities, etc.)';
COMMENT ON TABLE knowledge_graph_relationships IS 'Relationships between knowledge graph entities';
COMMENT ON TABLE temporal_analysis_results IS 'Cached results of temporal analysis and trend detection';
COMMENT ON TABLE memory_provenance IS 'Audit trail of memory operations for explainability';
COMMENT ON TABLE offloaded_contexts IS 'Compressed/archived contexts for memory efficiency';
COMMENT ON TABLE memory_system_metrics IS 'Performance and health metrics for the memory system';
