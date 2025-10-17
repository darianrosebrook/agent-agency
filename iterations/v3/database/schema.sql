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
TABLE audit_trail IS 'Complete audit trail for all system decisions';