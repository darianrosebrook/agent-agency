-- Migration 006: Multi-Turn Learning Coordinator Tables
-- Description: Creates tables for learning sessions, iterations, error patterns, and context snapshots
-- Created: 2025-10-12
-- Part of: ARBITER-009 (Multi-Turn Learning Coordinator)
-- Risk Tier: 1 (Critical)

-- Learning Sessions Table
-- Stores metadata and outcomes for each learning session
CREATE TABLE IF NOT EXISTS learning_sessions (
  session_id VARCHAR(255) PRIMARY KEY,
  task_id VARCHAR(255) NOT NULL,
  agent_id VARCHAR(255) NOT NULL,
  status VARCHAR(50) NOT NULL,

-- Configuration
max_iterations INTEGER NOT NULL DEFAULT 10,
progress_timeout INTEGER NOT NULL DEFAULT 30000,
no_progress_limit INTEGER NOT NULL DEFAULT 3,
resource_budget_mb INTEGER NOT NULL DEFAULT 100,
compression_ratio DECIMAL(3, 2) NOT NULL DEFAULT 0.70,
quality_threshold DECIMAL(3, 2) NOT NULL DEFAULT 0.85,
enable_adaptive_prompting BOOLEAN NOT NULL DEFAULT TRUE,
enable_error_recognition BOOLEAN NOT NULL DEFAULT TRUE,

-- Timing
start_time TIMESTAMP
WITH
    TIME ZONE NOT NULL DEFAULT NOW(),
    end_time TIMESTAMP
WITH
    TIME ZONE,

-- Metrics
iteration_count INTEGER NOT NULL DEFAULT 0,
quality_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
improvement_trajectory JSONB,
error_patterns JSONB,

-- Results
final_result JSONB,
learning_summary JSONB,
created_at TIMESTAMP
WITH
    TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP
WITH
    TIME ZONE NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT learning_sessions_status_check CHECK (
    status IN ('initializing', 'active', 'evaluating', 'completed', 'failed', 'timeout', 'resource_exhausted')
  ),
  CONSTRAINT learning_sessions_quality_score_check CHECK (quality_score >= 0 AND quality_score <= 1),
  CONSTRAINT learning_sessions_compression_ratio_check CHECK (compression_ratio >= 0 AND compression_ratio <= 1),
  CONSTRAINT learning_sessions_quality_threshold_check CHECK (quality_threshold >= 0 AND quality_threshold <= 1)
);

-- Learning Iterations Table
-- Stores individual iteration records within learning sessions
CREATE TABLE IF NOT EXISTS learning_iterations (
  iteration_id VARCHAR(255) PRIMARY KEY,
  session_id VARCHAR(255) NOT NULL REFERENCES learning_sessions(session_id) ON DELETE CASCADE,
  iteration_number INTEGER NOT NULL,

-- Timing
start_time TIMESTAMP
WITH
    TIME ZONE NOT NULL DEFAULT NOW(),
    end_time TIMESTAMP
WITH
    TIME ZONE,
    duration_ms INTEGER,

-- Context
context_snapshot_id VARCHAR(255) NOT NULL,

-- Error detection
error_detected BOOLEAN NOT NULL DEFAULT FALSE,
error_category VARCHAR(100),

-- Quality metrics
quality_score DECIMAL(5, 4) NOT NULL DEFAULT 0,
improvement_delta DECIMAL(5, 4) NOT NULL DEFAULT 0,
resource_usage_mb DECIMAL(8, 2) NOT NULL DEFAULT 0,

-- Modifications
prompt_modifications JSONB,
feedback JSONB,
created_at TIMESTAMP
WITH
    TIME ZONE NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT learning_iterations_quality_score_check CHECK (quality_score >= 0 AND quality_score <= 1),
  CONSTRAINT learning_iterations_error_category_check CHECK (
    error_category IS NULL OR error_category IN (
      'syntax_error', 'type_error', 'runtime_error', 'logic_error',
      'resource_error', 'timeout_error', 'validation_error',
      'dependency_error', 'configuration_error', 'unknown'
    )
  ),
  CONSTRAINT learning_iterations_unique_session_iteration UNIQUE (session_id, iteration_number)
);

-- Error Patterns Table
-- Stores recognized error patterns with remediation strategies


CREATE TABLE IF NOT EXISTS error_patterns (
  pattern_id VARCHAR(255) PRIMARY KEY,
  category VARCHAR(100) NOT NULL,
  pattern TEXT NOT NULL,
  frequency INTEGER NOT NULL DEFAULT 1,
  confidence DECIMAL(3, 2) NOT NULL DEFAULT 0,
  detected_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  remediation_strategy TEXT NOT NULL,
  success_rate DECIMAL(3, 2) NOT NULL DEFAULT 0,
  examples JSONB,
  
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT error_patterns_category_check CHECK (
    category IN (
      'syntax_error', 'type_error', 'runtime_error', 'logic_error',
      'resource_error', 'timeout_error', 'validation_error',
      'dependency_error', 'configuration_error', 'unknown'
    )
  ),
  CONSTRAINT error_patterns_confidence_check CHECK (confidence >= 0 AND confidence <= 1),
  CONSTRAINT error_patterns_success_rate_check CHECK (success_rate >= 0 AND success_rate <= 1)
);

-- Context Snapshots Table
-- Stores compressed context states for iterations
CREATE TABLE IF NOT EXISTS context_snapshots (
  snapshot_id VARCHAR(255) PRIMARY KEY,
  session_id VARCHAR(255) NOT NULL REFERENCES learning_sessions(session_id) ON DELETE CASCADE,
  iteration_number INTEGER NOT NULL,
  timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

-- Context data (stored compressed)
full_context JSONB,
compressed_context TEXT NOT NULL,
compression_ratio DECIMAL(5, 4) NOT NULL,
checksum_md5 VARCHAR(32) NOT NULL,
size_bytes INTEGER NOT NULL,

-- Differential storage
is_diff BOOLEAN NOT NULL DEFAULT FALSE,
based_on_snapshot_id VARCHAR(255),
created_at TIMESTAMP
WITH
    TIME ZONE NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT context_snapshots_compression_ratio_check CHECK (compression_ratio >= 0 AND compression_ratio <= 1),
  CONSTRAINT context_snapshots_based_on_fk FOREIGN KEY (based_on_snapshot_id) 
    REFERENCES context_snapshots(snapshot_id) ON DELETE SET NULL
);

-- Indexes for Performance

-- Learning Sessions indexes
CREATE INDEX IF NOT EXISTS idx_learning_sessions_task_id ON learning_sessions (task_id);

CREATE INDEX IF NOT EXISTS idx_learning_sessions_agent_id ON learning_sessions (agent_id);

CREATE INDEX IF NOT EXISTS idx_learning_sessions_status ON learning_sessions (status);

CREATE INDEX IF NOT EXISTS idx_learning_sessions_start_time ON learning_sessions (start_time DESC);

-- Learning Iterations indexes
CREATE INDEX IF NOT EXISTS idx_learning_iterations_session_id ON learning_iterations (session_id);

CREATE INDEX IF NOT EXISTS idx_learning_iterations_iteration_number ON learning_iterations (iteration_number);

CREATE INDEX IF NOT EXISTS idx_learning_iterations_error_detected ON learning_iterations (error_detected);

CREATE INDEX IF NOT EXISTS idx_learning_iterations_error_category ON learning_iterations (error_category);

-- Error Patterns indexes
CREATE INDEX IF NOT EXISTS idx_error_patterns_category ON error_patterns (category);

CREATE INDEX IF NOT EXISTS idx_error_patterns_confidence ON error_patterns (confidence DESC);

CREATE INDEX IF NOT EXISTS idx_error_patterns_success_rate ON error_patterns (success_rate DESC);

CREATE INDEX IF NOT EXISTS idx_error_patterns_detected_at ON error_patterns (detected_at DESC);

-- Context Snapshots indexes
CREATE INDEX IF NOT EXISTS idx_context_snapshots_session_id ON context_snapshots (session_id);

CREATE INDEX IF NOT EXISTS idx_context_snapshots_iteration_number ON context_snapshots (iteration_number);

CREATE INDEX IF NOT EXISTS idx_context_snapshots_timestamp ON context_snapshots (timestamp DESC);

CREATE INDEX IF NOT EXISTS idx_context_snapshots_based_on ON context_snapshots (based_on_snapshot_id);

-- GIN indexes for JSONB columns
CREATE INDEX IF NOT EXISTS idx_learning_sessions_error_patterns ON learning_sessions USING GIN (error_patterns);

CREATE INDEX IF NOT EXISTS idx_learning_iterations_prompt_modifications ON learning_iterations USING GIN (prompt_modifications);

CREATE INDEX IF NOT EXISTS idx_error_patterns_examples ON error_patterns USING GIN (examples);

-- Table Comments
COMMENT ON
TABLE learning_sessions IS 'Learning session metadata and outcomes for multi-turn agent learning';

COMMENT ON
TABLE learning_iterations IS 'Individual iteration records within learning sessions';

COMMENT ON
TABLE error_patterns IS 'Recognized error patterns with remediation strategies';

COMMENT ON
TABLE context_snapshots IS 'Compressed context states for iteration rollback and recovery';

-- Column Comments
COMMENT ON COLUMN learning_sessions.session_id IS 'Unique identifier for learning session';

COMMENT ON COLUMN learning_sessions.task_id IS 'ID of task being learned';

COMMENT ON COLUMN learning_sessions.agent_id IS 'ID of agent performing learning';

COMMENT ON COLUMN learning_sessions.improvement_trajectory IS 'Array of quality scores over iterations';

COMMENT ON COLUMN learning_sessions.learning_summary IS 'Summary of learning outcomes and insights';

COMMENT ON COLUMN learning_iterations.context_snapshot_id IS 'ID of context snapshot for this iteration';

COMMENT ON COLUMN learning_iterations.improvement_delta IS 'Change in quality score from previous iteration';

COMMENT ON COLUMN learning_iterations.prompt_modifications IS 'Adaptive prompt changes for this iteration';

COMMENT ON COLUMN error_patterns.remediation_strategy IS 'Suggested approach to resolve this error pattern';

COMMENT ON COLUMN error_patterns.examples IS 'Example instances of this error pattern';

COMMENT ON COLUMN context_snapshots.compressed_context IS 'Compressed context data (gzip + base64)';

COMMENT ON COLUMN context_snapshots.is_diff IS 'Whether this snapshot stores only differences';

COMMENT ON COLUMN context_snapshots.based_on_snapshot_id IS 'Base snapshot ID for differential storage';