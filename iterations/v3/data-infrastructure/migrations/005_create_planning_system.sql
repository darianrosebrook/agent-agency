-- Migration 005: Create Planning System Tables
-- Creates database tables for the execution planning system

-- Execution plans table
CREATE TABLE execution_plans (
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

-- Milestones table
CREATE TABLE milestones (
    id VARCHAR(100) NOT NULL,
    plan_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE,
    objective TEXT NOT NULL,
    scope JSONB NOT NULL DEFAULT '{}'::jsonb,
    interfaces JSONB NOT NULL DEFAULT '[]'::jsonb,
    tests JSONB NOT NULL DEFAULT '[]'::jsonb,
    evidence_gate JSONB NOT NULL DEFAULT '{}'::jsonb,
    rollback_plan TEXT,
    dependencies JSONB NOT NULL DEFAULT '[]'::jsonb,
    state VARCHAR(50) NOT NULL DEFAULT 'pending',
    assigned_worker_id UUID,
    estimated_effort DOUBLE PRECISION,
    priority VARCHAR(20) DEFAULT 'normal',
    risk_tier INTEGER DEFAULT 2,
    is_blocking BOOLEAN DEFAULT FALSE,
    blocking_reason TEXT,
    metrics JSONB,
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (plan_id, id)
);

-- Planning sessions table
CREATE TABLE planning_sessions (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE,
    orchestrator_id VARCHAR(255) NOT NULL,
    worker_pool_id VARCHAR(255) NOT NULL,
    council_session_id UUID,
    audit_correlation_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    execution_state JSONB NOT NULL DEFAULT '{}'::jsonb,
    started_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Evidence artifacts table
CREATE TABLE evidence_artifacts (
    id UUID PRIMARY KEY,
    milestone_id VARCHAR(100) NOT NULL,
    plan_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE,
    artifact_type VARCHAR(100) NOT NULL,
    artifact_data JSONB NOT NULL,
    verified BOOLEAN DEFAULT FALSE,
    collected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    verified_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Planning audit events table
CREATE TABLE planning_audit_events (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE,
    milestone_id VARCHAR(100),
    worker_id UUID,
    event_type VARCHAR(100) NOT NULL,
    description TEXT NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Planning telemetry table
CREATE TABLE planning_telemetry (
    id UUID PRIMARY KEY,
    plan_id UUID NOT NULL REFERENCES execution_plans(id) ON DELETE CASCADE,
    metric_type VARCHAR(100) NOT NULL,
    metric_value JSONB NOT NULL,
    collected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for performance
CREATE INDEX idx_execution_plans_session_id ON execution_plans(session_id);
CREATE INDEX idx_execution_plans_working_spec_id ON execution_plans(working_spec_id);
CREATE INDEX idx_execution_plans_state ON execution_plans(state);
CREATE INDEX idx_execution_plans_created_at ON execution_plans(created_at);

CREATE INDEX idx_milestones_plan_id ON milestones(plan_id);
CREATE INDEX idx_milestones_state ON milestones(state);
CREATE INDEX idx_milestones_assigned_worker ON milestones(assigned_worker_id);
CREATE INDEX idx_milestones_risk_tier ON milestones(risk_tier);

CREATE INDEX idx_planning_sessions_plan_id ON planning_sessions(plan_id);
CREATE INDEX idx_planning_sessions_status ON planning_sessions(status);

CREATE INDEX idx_evidence_artifacts_plan_id ON evidence_artifacts(plan_id);
CREATE INDEX idx_evidence_artifacts_milestone ON evidence_artifacts(plan_id, milestone_id);
CREATE INDEX idx_evidence_artifacts_type ON evidence_artifacts(artifact_type);
CREATE INDEX idx_evidence_artifacts_verified ON evidence_artifacts(verified);

CREATE INDEX idx_planning_audit_events_plan_id ON planning_audit_events(plan_id);
CREATE INDEX idx_planning_audit_events_milestone ON planning_audit_events(plan_id, milestone_id);
CREATE INDEX idx_planning_audit_events_worker ON planning_audit_events(worker_id);
CREATE INDEX idx_planning_audit_events_type ON planning_audit_events(event_type);
CREATE INDEX idx_planning_audit_events_created_at ON planning_audit_events(created_at);

CREATE INDEX idx_planning_telemetry_plan_id ON planning_telemetry(plan_id);
CREATE INDEX idx_planning_telemetry_type ON planning_telemetry(metric_type);
CREATE INDEX idx_planning_telemetry_collected_at ON planning_telemetry(collected_at);

-- Triggers for updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_execution_plans_updated_at BEFORE UPDATE ON execution_plans FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_milestones_updated_at BEFORE UPDATE ON milestones FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
