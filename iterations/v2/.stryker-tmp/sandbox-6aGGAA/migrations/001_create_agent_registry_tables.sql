/**
 * Migration: Create Agent Registry Tables
 * 
 * @author @darianrosebrook
 * @migration 001
 * @component ARBITER-001 (Agent Registry Manager)
 * 
 * Creates tables for storing agent profiles, capabilities, and performance history.
 * Supports zero-downtime deployment with rollback capability.
 */

-- Enable UUID extension for agent IDs
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- ============================================================================
-- Agent Profiles Table
-- ============================================================================
-- Stores core agent information and current state

CREATE TABLE IF NOT EXISTS agent_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(255) NOT NULL,
    model_family VARCHAR(50) NOT NULL,
    registered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_active_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    active_tasks INTEGER NOT NULL DEFAULT 0 CHECK (active_tasks >= 0),
    queued_tasks INTEGER NOT NULL DEFAULT 0 CHECK (queued_tasks >= 0),
    utilization_percent NUMERIC(5,2) NOT NULL DEFAULT 0.00 CHECK (utilization_percent >= 0 AND utilization_percent <= 100),

-- Metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Indexing for common queries
CONSTRAINT unique_agent_name UNIQUE (name) );

-- Index for querying active agents
CREATE INDEX IF NOT EXISTS idx_agent_profiles_last_active ON agent_profiles (last_active_at DESC);

-- Index for load balancing queries
CREATE INDEX IF NOT EXISTS idx_agent_profiles_utilization ON agent_profiles (utilization_percent ASC)
WHERE
    active_tasks < 10;

-- ============================================================================
-- Agent Capabilities Table
-- ============================================================================
-- Stores agent capabilities for task routing

CREATE TABLE IF NOT EXISTS agent_capabilities (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
    capability_type VARCHAR(50) NOT NULL, -- 'task_type', 'language', 'specialization'
    capability_value VARCHAR(100) NOT NULL,

-- Metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Prevent duplicate capabilities
CONSTRAINT unique_agent_capability UNIQUE (agent_id, capability_type, capability_value)
);

-- Index for capability queries
CREATE INDEX IF NOT EXISTS idx_agent_capabilities_lookup ON agent_capabilities (
    capability_type,
    capability_value
);

-- Index for agent capability retrieval
CREATE INDEX IF NOT EXISTS idx_agent_capabilities_by_agent ON agent_capabilities (agent_id);

-- ============================================================================
-- Agent Performance History Table
-- ============================================================================
-- Stores running average performance metrics

CREATE TABLE IF NOT EXISTS agent_performance_history (
    agent_id UUID PRIMARY KEY REFERENCES agent_profiles(id) ON DELETE CASCADE,
    success_rate NUMERIC(5,4) NOT NULL DEFAULT 0.8000 CHECK (success_rate >= 0 AND success_rate <= 1),
    average_quality NUMERIC(5,4) NOT NULL DEFAULT 0.7000 CHECK (average_quality >= 0 AND average_quality <= 1),
    average_latency_ms INTEGER NOT NULL DEFAULT 5000 CHECK (average_latency_ms >= 0),
    task_count INTEGER NOT NULL DEFAULT 0 CHECK (task_count >= 0),

-- Metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for performance-based queries
CREATE INDEX IF NOT EXISTS idx_agent_performance_success_rate ON agent_performance_history (success_rate DESC);

-- ============================================================================
-- Agent Performance Events Table (Optional)
-- ============================================================================
-- Stores individual performance events for detailed analysis
-- This is optional and can be used for auditing and debugging

CREATE TABLE IF NOT EXISTS agent_performance_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id UUID NOT NULL REFERENCES agent_profiles(id) ON DELETE CASCADE,
    task_id VARCHAR(255),
    success BOOLEAN NOT NULL,
    quality_score NUMERIC(5,4) NOT NULL CHECK (quality_score >= 0 AND quality_score <= 1),
    latency_ms INTEGER NOT NULL CHECK (latency_ms >= 0),
    tokens_used INTEGER,
    task_type VARCHAR(50),

-- Timestamp
recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Indexing for time-series queries
CONSTRAINT check_task_id_not_empty CHECK (task_id IS NULL OR task_id != '')
);

-- Index for time-series queries (recent events)
CREATE INDEX IF NOT EXISTS idx_agent_performance_events_time ON agent_performance_events (recorded_at DESC);

-- Index for agent-specific event retrieval
CREATE INDEX IF NOT EXISTS idx_agent_performance_events_by_agent ON agent_performance_events (agent_id, recorded_at DESC);

-- ============================================================================
-- Updated Timestamp Trigger
-- ============================================================================
-- Automatically update updated_at timestamps

CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER update_agent_profiles_updated_at
    BEFORE UPDATE ON agent_profiles
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agent_performance_history_updated_at
    BEFORE UPDATE ON agent_performance_history
    FOR EACH ROW
    EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Views for Common Queries
-- ============================================================================

-- View: Agent with all capabilities
CREATE OR REPLACE VIEW agent_profiles_with_capabilities AS
SELECT 
    ap.id,
    ap.name,
    ap.model_family,
    ap.registered_at,
    ap.last_active_at,
    ap.active_tasks,
    ap.queued_tasks,
    ap.utilization_percent,
    aph.success_rate,
    aph.average_quality,
    aph.average_latency_ms,
    aph.task_count,
    COALESCE(
        json_agg(
            json_build_object(
                'type', ac.capability_type,
                'value', ac.capability_value
            )
        ) FILTER (WHERE ac.id IS NOT NULL),
        '[]'::json
    ) as capabilities
FROM agent_profiles ap
LEFT JOIN agent_performance_history aph ON ap.id = aph.agent_id
LEFT JOIN agent_capabilities ac ON ap.id = ac.agent_id
GROUP BY ap.id, ap.name, ap.model_family, ap.registered_at, ap.last_active_at,
         ap.active_tasks, ap.queued_tasks, ap.utilization_percent,
         aph.success_rate, aph.average_quality, aph.average_latency_ms, aph.task_count;

-- View: Available agents (low utilization, recent activity)
CREATE OR REPLACE VIEW available_agents AS
SELECT ap.*, aph.success_rate, aph.average_quality, aph.task_count
FROM
    agent_profiles ap
    JOIN agent_performance_history aph ON ap.id = aph.agent_id
WHERE
    ap.utilization_percent < 80
    AND ap.last_active_at > NOW() - INTERVAL '24 hours'
ORDER BY aph.success_rate DESC, ap.utilization_percent ASC;

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON
TABLE agent_profiles IS 'Core agent information and current load state';

COMMENT ON
TABLE agent_capabilities IS 'Agent capabilities for task routing (task types, languages, specializations)';

COMMENT ON
TABLE agent_performance_history IS 'Running average performance metrics per agent';

COMMENT ON
TABLE agent_performance_events IS 'Individual performance events for detailed analysis and auditing';

COMMENT ON COLUMN agent_profiles.utilization_percent IS 'Current utilization as percentage (0-100)';

COMMENT ON COLUMN agent_performance_history.success_rate IS 'Running average success rate (0.0-1.0)';

COMMENT ON COLUMN agent_performance_history.average_quality IS 'Running average quality score from evaluations (0.0-1.0)';

COMMENT ON COLUMN agent_performance_history.task_count IS 'Total tasks completed (used for running average calculation)';

-- ============================================================================
-- Grants (adjust based on your security model)
-- ============================================================================

-- Grant appropriate permissions (example for a service account)
-- GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA public TO agent_agency_service;
-- GRANT SELECT ON ALL VIEWS IN SCHEMA public TO agent_agency_service;
-- GRANT USAGE, SELECT ON ALL SEQUENCES IN SCHEMA public TO agent_agency_service;

-- ============================================================================
-- Migration Complete
-- ============================================================================
-- This migration can be rolled back by running:
-- DROP VIEW IF EXISTS available_agents CASCADE;
-- DROP VIEW IF EXISTS agent_profiles_with_capabilities CASCADE;
-- DROP TABLE IF EXISTS agent_performance_events CASCADE;
-- DROP TABLE IF EXISTS agent_performance_history CASCADE;
-- DROP TABLE IF NOT EXISTS agent_capabilities CASCADE;
-- DROP TABLE IF EXISTS agent_profiles CASCADE;
-- DROP FUNCTION IF EXISTS update_updated_at_column() CASCADE;