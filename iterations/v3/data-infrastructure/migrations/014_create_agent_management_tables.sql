-- Migration 014: Create Agent Management Tables
-- Creates database tables for core agent management functionality:
-- tasks, task_executions, workers, judges, judge_evaluations, council_verdicts, debate_sessions

-- ===========================================
-- WORKERS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS workers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    worker_type VARCHAR(100) NOT NULL,
    specialty VARCHAR(255),
    model_name VARCHAR(255) NOT NULL,
    endpoint VARCHAR(500) NOT NULL,
    capabilities JSONB NOT NULL DEFAULT '{}'::jsonb,
    performance_history JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for workers
CREATE INDEX IF NOT EXISTS idx_workers_name ON workers(name);
CREATE INDEX IF NOT EXISTS idx_workers_type ON workers(worker_type);
CREATE INDEX IF NOT EXISTS idx_workers_specialty ON workers(specialty);
CREATE INDEX IF NOT EXISTS idx_workers_model_name ON workers(model_name);
CREATE INDEX IF NOT EXISTS idx_workers_is_active ON workers(is_active);
CREATE INDEX IF NOT EXISTS idx_workers_created_at ON workers(created_at);

-- ===========================================
-- JUDGES TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS judges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    model_name VARCHAR(255) NOT NULL,
    endpoint VARCHAR(500) NOT NULL,
    weight FLOAT NOT NULL DEFAULT 1.0 CHECK (weight >= 0.0 AND weight <= 1.0),
    timeout_ms INTEGER NOT NULL DEFAULT 30000 CHECK (timeout_ms > 0),
    optimization_target VARCHAR(100) NOT NULL DEFAULT 'quality',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for judges
CREATE INDEX IF NOT EXISTS idx_judges_name ON judges(name);
CREATE INDEX IF NOT EXISTS idx_judges_model_name ON judges(model_name);
CREATE INDEX IF NOT EXISTS idx_judges_is_active ON judges(is_active);
CREATE INDEX IF NOT EXISTS idx_judges_created_at ON judges(created_at);

-- ===========================================
-- TASKS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS tasks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    description TEXT NOT NULL,
    risk_tier VARCHAR(50) NOT NULL DEFAULT '2' CHECK (risk_tier IN ('1', '2', '3')),
    scope JSONB NOT NULL DEFAULT '{}'::jsonb,
    acceptance_criteria JSONB NOT NULL DEFAULT '[]'::jsonb,
    context JSONB NOT NULL DEFAULT '{}'::jsonb,
    caws_spec JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'paused', 'completed', 'cancelled', 'failed')),
    assigned_worker_id UUID REFERENCES workers(id) ON DELETE SET NULL,
    priority INTEGER CHECK (priority >= 0 AND priority <= 10),
    deadline TIMESTAMP WITH TIME ZONE,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for tasks
CREATE INDEX IF NOT EXISTS idx_tasks_title ON tasks(title);
CREATE INDEX IF NOT EXISTS idx_tasks_status ON tasks(status);
CREATE INDEX IF NOT EXISTS idx_tasks_risk_tier ON tasks(risk_tier);
CREATE INDEX IF NOT EXISTS idx_tasks_assigned_worker_id ON tasks(assigned_worker_id);
CREATE INDEX IF NOT EXISTS idx_tasks_priority ON tasks(priority);
CREATE INDEX IF NOT EXISTS idx_tasks_deadline ON tasks(deadline);
CREATE INDEX IF NOT EXISTS idx_tasks_created_at ON tasks(created_at);
CREATE INDEX IF NOT EXISTS idx_tasks_completed_at ON tasks(completed_at);
CREATE INDEX IF NOT EXISTS idx_tasks_scope ON tasks USING GIN(scope);
CREATE INDEX IF NOT EXISTS idx_tasks_context ON tasks USING GIN(context);

-- ===========================================
-- TASK EXECUTIONS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS task_executions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL REFERENCES workers(id) ON DELETE CASCADE,
    execution_started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    execution_completed_at TIMESTAMP WITH TIME ZONE,
    execution_time_ms INTEGER CHECK (execution_time_ms >= 0),
    status VARCHAR(50) NOT NULL DEFAULT 'running' CHECK (status IN ('running', 'completed', 'failed', 'cancelled')),
    worker_output JSONB NOT NULL DEFAULT '{}'::jsonb,
    self_assessment JSONB NOT NULL DEFAULT '{}'::jsonb,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    error_message TEXT,
    tokens_used INTEGER CHECK (tokens_used >= 0),
    execution_metadata JSONB,
    result_data JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for task_executions
CREATE INDEX IF NOT EXISTS idx_task_executions_task_id ON task_executions(task_id);
CREATE INDEX IF NOT EXISTS idx_task_executions_worker_id ON task_executions(worker_id);
CREATE INDEX IF NOT EXISTS idx_task_executions_status ON task_executions(status);
CREATE INDEX IF NOT EXISTS idx_task_executions_started_at ON task_executions(execution_started_at);
CREATE INDEX IF NOT EXISTS idx_task_executions_completed_at ON task_executions(execution_completed_at);
CREATE INDEX IF NOT EXISTS idx_task_executions_task_worker ON task_executions(task_id, worker_id);

-- ===========================================
-- COUNCIL VERDICTS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS council_verdicts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    verdict_id UUID NOT NULL UNIQUE,
    consensus_score FLOAT NOT NULL CHECK (consensus_score >= 0.0 AND consensus_score <= 1.0),
    final_verdict JSONB NOT NULL,
    individual_verdicts JSONB NOT NULL DEFAULT '[]'::jsonb,
    debate_rounds INTEGER NOT NULL DEFAULT 0 CHECK (debate_rounds >= 0),
    evaluation_time_ms INTEGER NOT NULL CHECK (evaluation_time_ms >= 0),
    contract JSONB NOT NULL DEFAULT '{}'::jsonb,
    verdict_details JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for council_verdicts
CREATE INDEX IF NOT EXISTS idx_council_verdicts_task_id ON council_verdicts(task_id);
CREATE INDEX IF NOT EXISTS idx_council_verdicts_verdict_id ON council_verdicts(verdict_id);
CREATE INDEX IF NOT EXISTS idx_council_verdicts_consensus_score ON council_verdicts(consensus_score);
CREATE INDEX IF NOT EXISTS idx_council_verdicts_created_at ON council_verdicts(created_at);

-- ===========================================
-- JUDGE EVALUATIONS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS judge_evaluations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    verdict_id UUID NOT NULL,
    judge_id UUID NOT NULL REFERENCES judges(id) ON DELETE CASCADE,
    judge_verdict JSONB NOT NULL,
    evaluation_time_ms INTEGER NOT NULL CHECK (evaluation_time_ms >= 0),
    tokens_used INTEGER CHECK (tokens_used >= 0),
    confidence FLOAT CHECK (confidence >= 0.0 AND confidence <= 1.0),
    evaluation_score FLOAT CHECK (evaluation_score >= 0.0 AND evaluation_score <= 1.0),
    confidence_score FLOAT CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    reasoning TEXT,
    evidence_used JSONB,
    evaluation_metadata JSONB,
    verdict_decision VARCHAR(100),
    risk_assessment JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for judge_evaluations
CREATE INDEX IF NOT EXISTS idx_judge_evaluations_verdict_id ON judge_evaluations(verdict_id);
CREATE INDEX IF NOT EXISTS idx_judge_evaluations_judge_id ON judge_evaluations(judge_id);
CREATE INDEX IF NOT EXISTS idx_judge_evaluations_created_at ON judge_evaluations(created_at);
CREATE INDEX IF NOT EXISTS idx_judge_evaluations_verdict_judge ON judge_evaluations(verdict_id, judge_id);

-- Foreign key constraint for judge_evaluations.verdict_id -> council_verdicts.verdict_id
-- Note: This references the unique verdict_id field, not the primary key id
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'council_verdicts') THEN
        -- Add foreign key constraint if council_verdicts table exists
        IF NOT EXISTS (
            SELECT 1 FROM information_schema.table_constraints 
            WHERE constraint_name = 'fk_judge_evaluations_verdict_id'
        ) THEN
            ALTER TABLE judge_evaluations 
            ADD CONSTRAINT fk_judge_evaluations_verdict_id 
            FOREIGN KEY (verdict_id) REFERENCES council_verdicts(verdict_id) ON DELETE CASCADE;
        END IF;
    END IF;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- ===========================================
-- DEBATE SESSIONS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS debate_sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    session_id UUID NOT NULL,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    conflicting_judges JSONB NOT NULL DEFAULT '[]'::jsonb,
    rounds JSONB NOT NULL DEFAULT '[]'::jsonb,
    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'resolved', 'failed')),
    final_consensus JSONB,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE
);

-- Indexes for debate_sessions
CREATE INDEX IF NOT EXISTS idx_debate_sessions_session_id ON debate_sessions(session_id);
CREATE INDEX IF NOT EXISTS idx_debate_sessions_task_id ON debate_sessions(task_id);
CREATE INDEX IF NOT EXISTS idx_debate_sessions_status ON debate_sessions(status);
CREATE INDEX IF NOT EXISTS idx_debate_sessions_created_at ON debate_sessions(created_at);

-- ===========================================
-- TRIGGERS FOR UPDATED_AT TIMESTAMPS
-- ===========================================

-- Reuse existing update_updated_at_column function from migration 005
-- If it doesn't exist, create it
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_workers_updated_at') THEN
        CREATE TRIGGER update_workers_updated_at
        BEFORE UPDATE ON workers
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_judges_updated_at') THEN
        CREATE TRIGGER update_judges_updated_at
        BEFORE UPDATE ON judges
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_tasks_updated_at') THEN
        CREATE TRIGGER update_tasks_updated_at
        BEFORE UPDATE ON tasks
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_task_executions_updated_at') THEN
        CREATE TRIGGER update_task_executions_updated_at
        BEFORE UPDATE ON task_executions
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_council_verdicts_updated_at') THEN
        CREATE TRIGGER update_council_verdicts_updated_at
        BEFORE UPDATE ON council_verdicts
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;

    IF NOT EXISTS (SELECT 1 FROM pg_trigger WHERE tgname = 'update_judge_evaluations_updated_at') THEN
        CREATE TRIGGER update_judge_evaluations_updated_at
        BEFORE UPDATE ON judge_evaluations
        FOR EACH ROW
        EXECUTE FUNCTION update_updated_at_column();
    END IF;
END
$$;

-- ===========================================
-- LOG MIGRATION
-- ===========================================

INSERT INTO migration_log (version, description, applied_at)
VALUES ('014', 'Create agent management tables (tasks, task_executions, workers, judges, judge_evaluations, council_verdicts, debate_sessions)', NOW())
ON CONFLICT (version) DO NOTHING;

