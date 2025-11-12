-- Migration 023: Create Execution Results Table
-- Creates database table for storing plan execution results

-- Execution results table
CREATE TABLE IF NOT EXISTS plan_execution_results (
    plan_id UUID PRIMARY KEY REFERENCES execution_plans(id) ON DELETE CASCADE,
    success BOOLEAN NOT NULL,
    milestones_completed INTEGER NOT NULL DEFAULT 0 CHECK (milestones_completed >= 0),
    total_duration_ms BIGINT NOT NULL DEFAULT 0 CHECK (total_duration_ms >= 0),
    evidence JSONB NOT NULL DEFAULT '{}'::jsonb,
    metrics JSONB NOT NULL DEFAULT '{}'::jsonb,
    final_state VARCHAR(50) NOT NULL,
    timeline JSONB NOT NULL DEFAULT '[]'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_plan_execution_results_success ON plan_execution_results(success);
CREATE INDEX IF NOT EXISTS idx_plan_execution_results_created_at ON plan_execution_results(created_at);
CREATE INDEX IF NOT EXISTS idx_plan_execution_results_final_state ON plan_execution_results(final_state);

-- Trigger for updated_at timestamp
CREATE TRIGGER update_plan_execution_results_updated_at 
    BEFORE UPDATE ON plan_execution_results 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();








