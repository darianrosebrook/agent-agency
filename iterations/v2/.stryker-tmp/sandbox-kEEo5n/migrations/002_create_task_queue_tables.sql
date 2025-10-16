/**
 * Migration: Create Task Queue Tables
 *
 * @author @darianrosebrook
 * @migration 002
 * @component ARBITER-005 (Arbiter Orchestrator - Task Queue)
 *
 * Creates tables for persistent task queue storage with priority support,
 * status tracking, and deadlock prevention.
 */

-- ============================================================================
-- Task Queue Table
-- ============================================================================
-- Stores queued tasks with priority ordering and metadata

CREATE TABLE IF NOT EXISTS task_queue (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id VARCHAR(255) NOT NULL UNIQUE,
    task_type VARCHAR(50) NOT NULL CHECK (task_type IN ('code-editing', 'code-review', 'analysis', 'research', 'validation', 'general')),
    description TEXT NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1 CHECK (priority >= 1 AND priority <= 10),
    timeout_ms BIGINT NOT NULL DEFAULT 30000 CHECK (timeout_ms > 0),
    attempts INTEGER NOT NULL DEFAULT 0 CHECK (attempts >= 0),
    max_attempts INTEGER NOT NULL DEFAULT 3 CHECK (max_attempts >= 1),

-- CAWS Budget constraints
budget_max_files INTEGER NOT NULL DEFAULT 40 CHECK (budget_max_files > 0),
budget_max_loc INTEGER NOT NULL DEFAULT 1500 CHECK (budget_max_loc > 0),

-- Task metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- JSON fields for flexible metadata
required_capabilities JSONB NOT NULL DEFAULT '{}',
task_metadata JSONB NOT NULL DEFAULT '{}',

-- Status and lifecycle
status VARCHAR(20) NOT NULL DEFAULT 'queued' CHECK (
    status IN (
        'queued',
        'dequeued',
        'completed',
        'failed',
        'cancelled'
    )
),
dequeued_at TIMESTAMPTZ,
completed_at TIMESTAMPTZ,

-- Indexing for priority queue operations
CONSTRAINT unique_task_id UNIQUE (task_id) );

-- Index for priority ordering (higher priority first, then FIFO)
CREATE INDEX IF NOT EXISTS idx_task_queue_priority_order ON task_queue (priority DESC, created_at ASC);

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_task_queue_status ON task_queue (status);

-- Index for cleanup operations
CREATE INDEX IF NOT EXISTS idx_task_queue_created_at ON task_queue (created_at);

-- Index for timeout monitoring
CREATE INDEX IF NOT EXISTS idx_task_queue_timeout_check ON task_queue (created_at, timeout_ms)
WHERE
    status = 'queued';

-- ============================================================================
-- Task Assignments Table
-- ============================================================================
-- Tracks task assignments to agents with routing decisions

CREATE TABLE IF NOT EXISTS task_assignments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    assignment_id VARCHAR(255) NOT NULL UNIQUE,
    task_id VARCHAR(255) NOT NULL REFERENCES task_queue(task_id) ON DELETE CASCADE,

-- Agent information
agent_id VARCHAR(255) NOT NULL,
agent_name VARCHAR(255) NOT NULL,
agent_model_family VARCHAR(50) NOT NULL,

-- Assignment details
assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
deadline TIMESTAMPTZ NOT NULL,
assignment_timeout_ms BIGINT NOT NULL,

-- Routing decision metadata
routing_confidence NUMERIC(3, 2) CHECK (
    routing_confidence >= 0
    AND routing_confidence <= 1
),
routing_strategy VARCHAR(30) NOT NULL CHECK (
    routing_strategy IN (
        'epsilon-greedy',
        'ucb',
        'capability-match',
        'load-balance'
    )
),
routing_reason TEXT,

-- Assignment status
status VARCHAR(20) NOT NULL DEFAULT 'assigned' CHECK (
    status IN (
        'assigned',
        'acknowledged',
        'executing',
        'completed',
        'failed',
        'cancelled'
    )
),
acknowledged_at TIMESTAMPTZ,
started_at TIMESTAMPTZ,
completed_at TIMESTAMPTZ,

-- Progress tracking
progress NUMERIC(3, 2) DEFAULT 0.0 CHECK (
    progress >= 0
    AND progress <= 1
),
last_progress_update TIMESTAMPTZ DEFAULT NOW(),

-- Error tracking
error_message TEXT, error_code VARCHAR(100),

-- Metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    assignment_metadata JSONB NOT NULL DEFAULT '{}'
);

-- Indexes for assignment queries
CREATE INDEX IF NOT EXISTS idx_task_assignments_task_id ON task_assignments (task_id);

CREATE INDEX IF NOT EXISTS idx_task_assignments_agent_id ON task_assignments (agent_id);

CREATE INDEX IF NOT EXISTS idx_task_assignments_status ON task_assignments (status);

CREATE INDEX IF NOT EXISTS idx_task_assignments_deadline ON task_assignments (deadline);

CREATE INDEX IF NOT EXISTS idx_task_assignments_created_at ON task_assignments (created_at DESC);

-- ============================================================================
-- Task Execution History Table
-- ============================================================================
-- Stores execution attempts and results for audit and analysis

CREATE TABLE IF NOT EXISTS task_execution_history (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    task_id VARCHAR(255) NOT NULL REFERENCES task_queue(task_id) ON DELETE CASCADE,
    assignment_id VARCHAR(255) NOT NULL REFERENCES task_assignments(assignment_id) ON DELETE CASCADE,

-- Execution details
attempt_number INTEGER NOT NULL DEFAULT 1 CHECK (attempt_number >= 1),
started_at TIMESTAMPTZ NOT NULL,
completed_at TIMESTAMPTZ,
execution_time_ms BIGINT,

-- Agent information (denormalized for performance)
agent_id VARCHAR(255) NOT NULL, agent_name VARCHAR(255) NOT NULL,

-- Results
success BOOLEAN,
quality_score NUMERIC(3, 2) CHECK (
    quality_score >= 0
    AND quality_score <= 1
),
tokens_used INTEGER,
error_message TEXT,

-- CAWS validation results
caws_passed BOOLEAN,
caws_violations JSONB DEFAULT '[]',
caws_budget_used_files INTEGER,
caws_budget_used_loc INTEGER,

-- Metadata
execution_metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for execution analysis
CREATE INDEX IF NOT EXISTS idx_task_execution_history_task_id ON task_execution_history (task_id);

CREATE INDEX IF NOT EXISTS idx_task_execution_history_agent_id ON task_execution_history (agent_id);

CREATE INDEX IF NOT EXISTS idx_task_execution_history_success ON task_execution_history (success);

CREATE INDEX IF NOT EXISTS idx_task_execution_history_started_at ON task_execution_history (started_at DESC);

-- ============================================================================
-- Queue Statistics Table
-- ============================================================================
-- Persistent storage for queue performance metrics

CREATE TABLE IF NOT EXISTS queue_statistics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Queue metrics
queue_depth INTEGER NOT NULL DEFAULT 0,
total_enqueued BIGINT NOT NULL DEFAULT 0,
total_dequeued BIGINT NOT NULL DEFAULT 0,
total_completed BIGINT NOT NULL DEFAULT 0,
total_failed BIGINT NOT NULL DEFAULT 0,

-- Performance metrics
average_wait_time_ms NUMERIC(10, 2),
max_wait_time_ms BIGINT,
average_processing_time_ms NUMERIC(10, 2),

-- Priority distribution
priority_distribution JSONB NOT NULL DEFAULT '{}',

-- Status distribution
status_distribution JSONB NOT NULL DEFAULT '{}',

-- System metrics
memory_usage_mb NUMERIC(8,2),
    active_connections INTEGER,

    CONSTRAINT queue_statistics_recorded_at_unique UNIQUE (recorded_at)
);

-- Index for time-series queries
CREATE INDEX IF NOT EXISTS idx_queue_statistics_recorded_at ON queue_statistics (recorded_at DESC);

-- ============================================================================
-- Queue Configuration Table
-- ============================================================================
-- Persistent configuration storage

CREATE TABLE IF NOT EXISTS queue_configuration (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4 (),
    config_key VARCHAR(100) NOT NULL UNIQUE,
    config_value JSONB NOT NULL,
    description TEXT,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by VARCHAR(100),
    CONSTRAINT queue_configuration_key_unique UNIQUE (config_key)
);

-- ============================================================================
-- Functions and Triggers
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers
CREATE TRIGGER update_task_queue_updated_at
    BEFORE UPDATE ON task_queue
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_task_assignments_updated_at
    BEFORE UPDATE ON task_assignments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ============================================================================
-- Initial Configuration Data
-- ============================================================================

-- Insert default queue configuration
INSERT INTO
    queue_configuration (
        config_key,
        config_value,
        description
    )
VALUES (
        'max_capacity',
        '1000',
        'Maximum number of tasks that can be queued'
    ),
    (
        'default_timeout_ms',
        '30000',
        'Default task timeout in milliseconds'
    ),
    (
        'max_retries',
        '3',
        'Maximum retry attempts for failed tasks'
    ),
    (
        'priority_mode',
        '"priority"',
        'Queue priority mode: fifo, priority, or deadline'
    ),
    (
        'persistence_enabled',
        'true',
        'Whether queue persistence is enabled'
    ) ON CONFLICT (config_key) DO NOTHING;

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON
TABLE task_queue IS 'Persistent task queue with priority ordering and CAWS budget constraints';

COMMENT ON
TABLE task_assignments IS 'Task assignments to agents with routing decision tracking';

COMMENT ON
TABLE task_execution_history IS 'Audit trail of task execution attempts and results';

COMMENT ON
TABLE queue_statistics IS 'Time-series performance metrics for queue monitoring';

COMMENT ON
TABLE queue_configuration IS 'Persistent configuration storage for queue settings';

COMMENT ON COLUMN task_queue.priority IS 'Task priority (1-10, higher = more urgent)';

COMMENT ON COLUMN task_queue.attempts IS 'Number of execution attempts made';

COMMENT ON COLUMN task_assignments.routing_confidence IS 'Confidence score of routing decision (0-1)';

COMMENT ON COLUMN task_assignments.deadline IS 'Assignment deadline timestamp';

COMMENT ON COLUMN task_execution_history.caws_passed IS 'Whether task passed CAWS validation';

-- ============================================================================
-- Rollback Section (for migration reversal)
-- ============================================================================

-- Note: In a production system, you'd want proper rollback scripts
-- For now, this migration is additive and can be rolled back by dropping tables

/*
-- Rollback commands (run in reverse order):
-- DROP TABLE IF EXISTS queue_configuration;
-- DROP TABLE IF EXISTS queue_statistics;
-- DROP TABLE IF EXISTS task_execution_history;
-- DROP TABLE IF EXISTS task_assignments;
-- DROP TABLE IF EXISTS task_queue;
-- DROP FUNCTION IF EXISTS update_updated_at_column();
*/