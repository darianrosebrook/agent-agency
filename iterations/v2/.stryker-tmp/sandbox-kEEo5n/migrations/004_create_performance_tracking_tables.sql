-- Performance Tracking Tables Migration
-- ARBITER-004: Comprehensive Performance Tracker Implementation
--
-- This migration creates tables for storing performance metrics, benchmark data,
-- and RL training datasets with proper indexing and constraints.

-- Enable UUID extension if not already enabled
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- =====================================================
-- Performance Events Table
-- Stores raw performance events from all agent interactions
-- =====================================================

CREATE TABLE performance_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN (
        'task_execution_start',
        'task_execution_complete',
        'routing_decision',
        'agent_selection',
        'constitutional_validation',
        'evaluation_outcome',
        'anomaly_detected',
        'system_load_spike'
    )),
    timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    agent_id VARCHAR(255),
    task_id VARCHAR(255),
    integrity_hash VARCHAR(128) NOT NULL,

-- Performance metrics as JSONB for flexibility
latency_metrics JSONB,
accuracy_metrics JSONB,
resource_metrics JSONB,
compliance_metrics JSONB,
cost_metrics JSONB,
reliability_metrics JSONB,

-- Additional context data
context JSONB, metadata JSONB,

-- Audit fields
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT performance_events_integrity_hash_unique UNIQUE (integrity_hash),
    CONSTRAINT performance_events_timestamp_not_future CHECK (timestamp <= NOW() + INTERVAL '1 hour')
);

-- Indexes for performance
CREATE INDEX idx_performance_events_timestamp ON performance_events (timestamp DESC);

CREATE INDEX idx_performance_events_agent_id ON performance_events (agent_id);

CREATE INDEX idx_performance_events_task_id ON performance_events (task_id);

CREATE INDEX idx_performance_events_event_type ON performance_events (event_type);

CREATE INDEX idx_performance_events_integrity_hash ON performance_events (integrity_hash);

-- Partial indexes for common queries (removed due to IMMUTABLE function requirement)
-- CREATE INDEX idx_performance_events_recent ON performance_events (timestamp DESC)
-- WHERE
--     timestamp > NOW() - INTERVAL '24 hours';

-- CREATE INDEX idx_performance_events_agent_recent ON performance_events (agent_id, timestamp DESC)
-- WHERE
--     timestamp > NOW() - INTERVAL '7 days';

-- =====================================================
-- Agent Performance Profiles Table
-- Stores aggregated performance profiles for agents
-- =====================================================

CREATE TABLE agent_performance_profiles (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id VARCHAR(255) NOT NULL,
    task_type VARCHAR(255) NOT NULL,
    aggregation_window VARCHAR(20) NOT NULL CHECK (aggregation_window IN ('realtime', 'short', 'medium', 'long')),

-- Time range for this profile
start_time TIMESTAMPTZ NOT NULL, end_time TIMESTAMPTZ NOT NULL,

-- Aggregation metadata
sample_count INTEGER NOT NULL CHECK (sample_count > 0),
confidence_score DECIMAL(3, 2) NOT NULL CHECK (
    confidence_score >= 0
    AND confidence_score <= 1
),

-- Complete performance metrics
performance_metrics JSONB NOT NULL,

-- Trend analysis
trend_direction VARCHAR(20) NOT NULL CHECK (
    trend_direction IN (
        'improving',
        'declining',
        'stable'
    )
),
trend_magnitude DECIMAL(4, 3) NOT NULL CHECK (
    trend_magnitude >= -1
    AND trend_magnitude <= 1
),
trend_confidence DECIMAL(3, 2) NOT NULL CHECK (
    trend_confidence >= 0
    AND trend_confidence <= 1
),
trend_time_window_hours INTEGER NOT NULL CHECK (trend_time_window_hours > 0),

-- Outlier information
outlier_count INTEGER NOT NULL DEFAULT 0, outlier_events JSONB,

-- Audit fields
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT agent_performance_profiles_time_range_valid CHECK (start_time < end_time),
    CONSTRAINT agent_performance_profiles_unique_agent_task_window UNIQUE (agent_id, task_type, aggregation_window, start_time)
);

-- Indexes for performance
CREATE INDEX idx_agent_performance_profiles_agent_task ON agent_performance_profiles (agent_id, task_type);

CREATE INDEX idx_agent_performance_profiles_window_time ON agent_performance_profiles (
    aggregation_window,
    end_time DESC
);

CREATE INDEX idx_agent_performance_profiles_confidence ON agent_performance_profiles (confidence_score DESC);

CREATE INDEX idx_agent_performance_profiles_trend ON agent_performance_profiles (
    trend_direction,
    trend_confidence DESC
);

-- =====================================================
-- Benchmark Datasets Table
-- Stores benchmark datasets for model evaluation
-- =====================================================

CREATE TABLE benchmark_datasets (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(500) NOT NULL,
    description TEXT,
    task_type VARCHAR(255) NOT NULL,

-- Dataset metadata
version VARCHAR(50) NOT NULL DEFAULT '1.0.0',
test_case_count INTEGER NOT NULL CHECK (test_case_count > 0),
dataset_size_bytes BIGINT,
is_active BOOLEAN NOT NULL DEFAULT true,

-- Baseline performance metrics
baseline_metrics JSONB NOT NULL,

-- Dataset storage
storage_location VARCHAR(1000),
access_pattern VARCHAR(50) CHECK (
    access_pattern IN (
        'local',
        's3',
        'http',
        'database'
    )
),

-- Validation
checksum VARCHAR(128),
validation_status VARCHAR(20) DEFAULT 'pending' CHECK (
    validation_status IN (
        'pending',
        'validating',
        'valid',
        'invalid'
    )
),

-- Audit fields
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
created_by VARCHAR(255),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_by VARCHAR(255),

-- Constraints
CONSTRAINT benchmark_datasets_id_format CHECK (id ~ '^[a-zA-Z0-9_-]+$')
);

-- Indexes for performance
CREATE INDEX idx_benchmark_datasets_task_type ON benchmark_datasets (task_type);

CREATE INDEX idx_benchmark_datasets_active ON benchmark_datasets (is_active)
WHERE
    is_active = true;

CREATE INDEX idx_benchmark_datasets_created ON benchmark_datasets (created_at DESC);

-- =====================================================
-- Benchmark Evaluations Table
-- Stores results from benchmark evaluations
-- =====================================================

CREATE TABLE benchmark_evaluations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id VARCHAR(255) NOT NULL,
    dataset_id VARCHAR(255) NOT NULL REFERENCES benchmark_datasets(id) ON DELETE CASCADE,

-- Evaluation timing
started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
completed_at TIMESTAMPTZ,
duration_ms INTEGER,

-- Status tracking
status VARCHAR(20) NOT NULL DEFAULT 'queued' CHECK (
    status IN (
        'queued',
        'running',
        'completed',
        'failed',
        'cancelled'
    )
),
progress DECIMAL(3, 2) CHECK (
    progress >= 0
    AND progress <= 1
),

-- Results
overall_score DECIMAL(4, 3) CHECK (
    overall_score >= 0
    AND overall_score <= 1
),
performance_metrics JSONB,

-- Test case results
test_case_results JSONB,
passed_test_cases INTEGER DEFAULT 0,
failed_test_cases INTEGER DEFAULT 0,

-- Baseline comparison
baseline_comparison JSONB,

-- Error information
error_message TEXT, error_details JSONB,

-- Configuration used
evaluation_config JSONB,

-- Audit fields
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
created_by VARCHAR(255),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT benchmark_evaluations_completion_check CHECK (
        (status = 'completed' AND completed_at IS NOT NULL AND overall_score IS NOT NULL) OR
        (status != 'completed')
    ),
    CONSTRAINT benchmark_evaluations_duration_positive CHECK (duration_ms IS NULL OR duration_ms > 0)
);

-- Indexes for performance
CREATE INDEX idx_benchmark_evaluations_agent_dataset ON benchmark_evaluations (agent_id, dataset_id);

CREATE INDEX idx_benchmark_evaluations_status ON benchmark_evaluations (status);

CREATE INDEX idx_benchmark_evaluations_started ON benchmark_evaluations (started_at DESC);

CREATE INDEX idx_benchmark_evaluations_score ON benchmark_evaluations (overall_score DESC)
WHERE
    status = 'completed';

-- =====================================================
-- RL Training Batches Table
-- Stores batches of RL training data
-- =====================================================

CREATE TABLE rl_training_batches (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    agent_id VARCHAR(255) NOT NULL,

-- Batch metadata
sample_count INTEGER NOT NULL CHECK (sample_count > 0),
quality_score DECIMAL(3, 2) NOT NULL CHECK (
    quality_score >= 0
    AND quality_score <= 1
),
anonymization_level VARCHAR(20) NOT NULL DEFAULT 'differential' CHECK (
    anonymization_level IN (
        'basic',
        'differential',
        'secure'
    )
),

-- Batch status
status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
    status IN (
        'pending',
        'processing',
        'ready',
        'consumed',
        'expired'
    )
),
priority INTEGER DEFAULT 0,

-- Training data
training_samples JSONB NOT NULL,

-- Processing metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
processed_at TIMESTAMPTZ,
consumed_at TIMESTAMPTZ,
expires_at TIMESTAMPTZ,

-- Quality metrics
data_quality_metrics JSONB,

-- Audit fields
created_by VARCHAR(255),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT rl_training_batches_expiry_logic CHECK (
        (status = 'expired' AND expires_at IS NOT NULL) OR
        (status != 'expired')
    )
);

-- Indexes for performance
CREATE INDEX idx_rl_training_batches_agent_status ON rl_training_batches (agent_id, status);

CREATE INDEX idx_rl_training_batches_quality ON rl_training_batches (quality_score DESC)
WHERE
    status = 'ready';

CREATE INDEX idx_rl_training_batches_created ON rl_training_batches (created_at DESC);

CREATE INDEX idx_rl_training_batches_expires ON rl_training_batches (expires_at)
WHERE
    status = 'ready';

-- =====================================================
-- Performance Anomalies Table
-- Stores detected performance anomalies
-- =====================================================

CREATE TABLE performance_anomalies (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    anomaly_type VARCHAR(50) NOT NULL CHECK (anomaly_type IN (
        'latency_spike',
        'accuracy_drop',
        'error_rate_increase',
        'resource_saturation',
        'constitutional_violation_spike'
    )),
    severity VARCHAR(20) NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),

-- Affected entities
agent_id VARCHAR(255), task_id VARCHAR(255),

-- Detection information
detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
detection_method VARCHAR(100) NOT NULL,

-- Anomaly details
description TEXT NOT NULL,
anomaly_metrics JSONB,
threshold_values JSONB,

-- Impact assessment
impact_assessment JSONB NOT NULL,

-- Resolution information
status VARCHAR(20) NOT NULL DEFAULT 'active' CHECK (
    status IN (
        'active',
        'investigating',
        'resolved',
        'false_positive'
    )
),
resolved_at TIMESTAMPTZ,
resolution_notes TEXT,

-- Recommendations
recommendations JSONB,

-- Related events
related_event_ids UUID[],

-- Audit fields
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_by VARCHAR(255),

-- Constraints
CONSTRAINT performance_anomalies_resolution_check CHECK (
        (status IN ('resolved', 'false_positive') AND resolved_at IS NOT NULL) OR
        (status NOT IN ('resolved', 'false_positive'))
    )
);

-- Indexes for performance
CREATE INDEX idx_performance_anomalies_status_severity ON performance_anomalies (status, severity);

CREATE INDEX idx_performance_anomalies_agent ON performance_anomalies (agent_id);

CREATE INDEX idx_performance_anomalies_detected ON performance_anomalies (detected_at DESC);

CREATE INDEX idx_performance_anomalies_type ON performance_anomalies (anomaly_type);

-- =====================================================
-- Data Quality Metrics Table
-- Stores data quality metrics for monitoring
-- =====================================================

CREATE TABLE data_quality_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_type VARCHAR(50) NOT NULL CHECK (metric_type IN (
        'collection_integrity',
        'aggregation_accuracy',
        'anonymization_effectiveness',
        'training_data_quality'
    )),
    agent_id VARCHAR(255),

-- Metric values
metric_value DECIMAL(8, 4) NOT NULL,
threshold_value DECIMAL(8, 4),
is_above_threshold BOOLEAN,

-- Time window
time_window_start TIMESTAMPTZ NOT NULL,
time_window_end TIMESTAMPTZ NOT NULL,

-- Additional context
metric_details JSONB,

-- Audit fields
recorded_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT data_quality_metrics_time_window_valid CHECK (time_window_start < time_window_end)
);

-- Indexes for performance
CREATE INDEX idx_data_quality_metrics_type_agent ON data_quality_metrics (metric_type, agent_id);

CREATE INDEX idx_data_quality_metrics_recorded ON data_quality_metrics (recorded_at DESC);

CREATE INDEX idx_data_quality_metrics_threshold ON data_quality_metrics (is_above_threshold);

-- =====================================================
-- System Health Metrics Table
-- Stores system-wide performance health metrics
-- =====================================================

CREATE TABLE system_health_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    metric_timestamp TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- System-wide metrics
active_agents INTEGER NOT NULL DEFAULT 0,
total_tasks_processed BIGINT NOT NULL DEFAULT 0,
average_latency_ms DECIMAL(8, 2),
overall_success_rate DECIMAL(4, 3),

-- Resource utilization
system_cpu_utilization DECIMAL(5, 2),
system_memory_utilization DECIMAL(5, 2),
database_connections_active INTEGER,
database_connections_idle INTEGER,

-- Performance indicators
p95_latency_ms DECIMAL(8, 2),
p99_latency_ms DECIMAL(8, 2),
error_rate_percent DECIMAL(5, 2),
throughput_tasks_per_minute DECIMAL(6, 2),

-- Anomaly counts
active_anomalies INTEGER NOT NULL DEFAULT 0,
critical_anomalies INTEGER NOT NULL DEFAULT 0,

-- Data collection stats
events_collected_last_hour INTEGER NOT NULL DEFAULT 0,
data_integrity_violations INTEGER NOT NULL DEFAULT 0,

-- Additional context
system_status VARCHAR(20) DEFAULT 'healthy' CHECK (
    system_status IN (
        'healthy',
        'degraded',
        'critical',
        'maintenance'
    )
),
status_details JSONB,

-- Audit fields
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW() );

-- Indexes for performance
CREATE INDEX idx_system_health_metrics_timestamp ON system_health_metrics (metric_timestamp DESC);

CREATE INDEX idx_system_health_metrics_status ON system_health_metrics (system_status);

CREATE INDEX idx_system_health_metrics_anomalies ON system_health_metrics (
    active_anomalies,
    critical_anomalies
);

-- =====================================================
-- Triggers and Functions
-- =====================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers to relevant tables
CREATE TRIGGER update_performance_events_updated_at BEFORE UPDATE ON performance_events FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_agent_performance_profiles_updated_at BEFORE UPDATE ON agent_performance_profiles FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_benchmark_datasets_updated_at BEFORE UPDATE ON benchmark_datasets FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_benchmark_evaluations_updated_at BEFORE UPDATE ON benchmark_evaluations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rl_training_batches_updated_at BEFORE UPDATE ON rl_training_batches FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_performance_anomalies_updated_at BEFORE UPDATE ON performance_anomalies FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Function to automatically set expires_at for RL training batches
CREATE OR REPLACE FUNCTION set_rl_batch_expiry()
RETURNS TRIGGER AS $$
BEGIN
    IF NEW.expires_at IS NULL THEN
        NEW.expires_at := NEW.created_at + INTERVAL '30 days';
    END IF;
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER set_rl_batch_expiry_trigger BEFORE INSERT ON rl_training_batches FOR EACH ROW EXECUTE FUNCTION set_rl_batch_expiry();

-- Function to validate performance metrics JSONB structure
CREATE OR REPLACE FUNCTION validate_performance_metrics(metrics JSONB)
RETURNS BOOLEAN AS $$
BEGIN
    -- Basic validation - ensure required metric categories exist
    RETURN metrics ? 'latency' AND metrics ? 'accuracy' AND metrics ? 'resources' AND metrics ? 'compliance' AND metrics ? 'cost' AND metrics ? 'reliability';
END;
$$ language 'plpgsql';

-- =====================================================
-- Views for Common Queries
-- =====================================================

-- View for recent performance events (last 24 hours)
CREATE VIEW recent_performance_events AS
SELECT *
FROM performance_events
WHERE
    timestamp > NOW() - INTERVAL '24 hours'
ORDER BY timestamp DESC;

-- View for agent performance summary
CREATE VIEW agent_performance_summary AS
SELECT
    agent_id,
    task_type,
    COUNT(*) as total_events,
    AVG((latency_metrics->>'averageMs')::numeric) as avg_latency_ms,
    AVG((accuracy_metrics->>'successRate')::numeric) as avg_success_rate,
    MIN(timestamp) as first_event,
    MAX(timestamp) as last_event
FROM performance_events
WHERE agent_id IS NOT NULL
GROUP BY agent_id, task_type;

-- View for system performance dashboard
CREATE VIEW system_performance_dashboard AS
SELECT
    DATE_TRUNC ('hour', metric_timestamp) as hour,
    AVG(average_latency_ms) as avg_latency,
    AVG(overall_success_rate) as avg_success_rate,
    AVG(throughput_tasks_per_minute) as avg_throughput,
    MAX(active_anomalies) as max_anomalies,
    COUNT(*) as metric_count
FROM system_health_metrics
WHERE
    metric_timestamp > NOW() - INTERVAL '7 days'
GROUP BY
    DATE_TRUNC ('hour', metric_timestamp)
ORDER BY hour DESC;

-- =====================================================
-- Comments for Documentation
-- =====================================================

COMMENT ON
TABLE performance_events IS 'Raw performance events from all agent interactions with integrity verification';

COMMENT ON
TABLE agent_performance_profiles IS 'Aggregated performance profiles for agents with trend analysis';

COMMENT ON
TABLE benchmark_datasets IS 'Benchmark datasets for model evaluation and comparison';

COMMENT ON
TABLE benchmark_evaluations IS 'Results from benchmark evaluations against datasets';

COMMENT ON
TABLE rl_training_batches IS 'Batches of RL training data ready for model training';

COMMENT ON
TABLE performance_anomalies IS 'Detected performance anomalies with impact assessment';

COMMENT ON
TABLE data_quality_metrics IS 'Data quality metrics for monitoring pipeline health';

COMMENT ON
TABLE system_health_metrics IS 'System-wide performance health metrics';

-- =====================================================
-- Migration Complete
-- =====================================================