-- Migration: Learning Signals Infrastructure
-- Purpose: Add learning signals table for adaptive routing and performance tracking
-- Version: 002
-- Created: 2025-01-24

-- Learning signals table for capturing task outcomes and judge performance
CREATE TABLE learning_signals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    verdict_id UUID REFERENCES council_verdicts(id) ON DELETE CASCADE,

-- Task outcome classification
outcome_type TEXT NOT NULL CHECK (
    outcome_type IN (
        'success',
        'partial_success',
        'failure',
        'timeout'
    )
),
outcome_data JSONB NOT NULL, -- Serialized TaskOutcome data

-- Judge dissent tracking
dissent_data JSONB NOT NULL DEFAULT '[]'::jsonb, -- Array of JudgeDissent

-- Performance metrics
latency_ms INTEGER NOT NULL,
quality_score FLOAT NOT NULL CHECK (
    quality_score >= 0.0
    AND quality_score <= 1.0
),
caws_compliance_score FLOAT NOT NULL CHECK (
    caws_compliance_score >= 0.0
    AND caws_compliance_score <= 1.0
),
claim_verification_score FLOAT CHECK (
    claim_verification_score >= 0.0
    AND claim_verification_score <= 1.0
),

-- Resource usage metrics
cpu_usage_percent FLOAT NOT NULL CHECK (
    cpu_usage_percent >= 0.0
    AND cpu_usage_percent <= 100.0
),
memory_usage_mb INTEGER NOT NULL CHECK (memory_usage_mb >= 0),
thermal_status TEXT NOT NULL CHECK (
    thermal_status IN (
        'normal',
        'warning',
        'throttling',
        'critical'
    )
),
ane_utilization FLOAT CHECK (
    ane_utilization >= 0.0
    AND ane_utilization <= 1.0
),
gpu_utilization FLOAT CHECK (
    gpu_utilization >= 0.0
    AND gpu_utilization <= 1.0
),
energy_consumption FLOAT CHECK (energy_consumption >= 0.0),

-- Task complexity assessment
task_complexity JSONB NOT NULL, -- Serialized TaskComplexity

-- Worker performance (optional)
worker_performance JSONB, -- Serialized WorkerPerformanceMetrics

-- Timestamps
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX idx_learning_signals_task_id ON learning_signals (task_id);

CREATE INDEX idx_learning_signals_verdict_id ON learning_signals (verdict_id);

CREATE INDEX idx_learning_signals_outcome_type ON learning_signals (outcome_type);

CREATE INDEX idx_learning_signals_created_at ON learning_signals (created_at);

CREATE INDEX idx_learning_signals_quality_score ON learning_signals (quality_score);

CREATE INDEX idx_learning_signals_latency_ms ON learning_signals (latency_ms);

-- Composite indexes for common queries
CREATE INDEX idx_learning_signals_task_outcome ON learning_signals (task_id, outcome_type);

CREATE INDEX idx_learning_signals_time_quality ON learning_signals (created_at, quality_score);

-- Performance metrics aggregation view
CREATE VIEW learning_signals_performance AS
SELECT
    DATE_TRUNC ('hour', created_at) as hour_bucket,
    COUNT(*) as total_signals,
    AVG(quality_score) as avg_quality_score,
    AVG(latency_ms) as avg_latency_ms,
    AVG(caws_compliance_score) as avg_caws_compliance,
    AVG(cpu_usage_percent) as avg_cpu_usage,
    AVG(memory_usage_mb) as avg_memory_usage,
    COUNT(*) FILTER (
        WHERE
            outcome_type = 'success'
    ) * 100.0 / COUNT(*) as success_rate,
    COUNT(*) FILTER (
        WHERE
            outcome_type = 'failure'
    ) * 100.0 / COUNT(*) as failure_rate,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'normal'
    ) * 100.0 / COUNT(*) as thermal_normal_rate
FROM learning_signals
GROUP BY
    DATE_TRUNC ('hour', created_at)
ORDER BY hour_bucket DESC;

-- Judge performance aggregation view
CREATE VIEW judge_performance_metrics AS
WITH
    judge_dissent_analysis AS (
        SELECT
            id,
            task_id,
            verdict_id,
            outcome_type,
            quality_score,
            latency_ms,
            jsonb_array_length (dissent_data) as dissent_count,
            created_at
        FROM learning_signals
    ),
    judge_verdict_mapping AS (
        SELECT l.id, l.task_id, l.verdict_id, l.outcome_type, l.quality_score, l.latency_ms, l.dissent_count, l.created_at, cv.judge_verdicts
        FROM
            judge_dissent_analysis l
            LEFT JOIN council_verdicts cv ON l.verdict_id = cv.id
    )
SELECT
    DATE_TRUNC ('day', created_at) as day_bucket,
    COUNT(*) as total_evaluations,
    AVG(quality_score) as avg_quality_score,
    AVG(latency_ms) as avg_latency_ms,
    AVG(dissent_count) as avg_dissent_count,
    COUNT(*) FILTER (
        WHERE
            outcome_type = 'success'
    ) * 100.0 / COUNT(*) as success_rate,
    COUNT(*) FILTER (
        WHERE
            dissent_count = 0
    ) * 100.0 / COUNT(*) as consensus_rate
FROM judge_verdict_mapping
GROUP BY
    DATE_TRUNC ('day', created_at)
ORDER BY day_bucket DESC;

-- Resource usage trends view
CREATE VIEW resource_usage_trends AS
SELECT
    DATE_TRUNC ('hour', created_at) as hour_bucket,
    AVG(cpu_usage_percent) as avg_cpu_usage,
    AVG(memory_usage_mb) as avg_memory_usage,
    AVG(COALESCE(ane_utilization, 0)) as avg_ane_utilization,
    AVG(COALESCE(gpu_utilization, 0)) as avg_gpu_utilization,
    AVG(
        COALESCE(energy_consumption, 0)
    ) as avg_energy_consumption,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'normal'
    ) * 100.0 / COUNT(*) as thermal_normal_rate,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'warning'
    ) * 100.0 / COUNT(*) as thermal_warning_rate,
    COUNT(*) FILTER (
        WHERE
            thermal_status = 'throttling'
    ) * 100.0 / COUNT(*) as thermal_throttling_rate
FROM learning_signals
GROUP BY
    DATE_TRUNC ('hour', created_at)
ORDER BY hour_bucket DESC;

-- Learning recommendations table
CREATE TABLE learning_recommendations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    recommendation_type TEXT NOT NULL CHECK (recommendation_type IN (
        'routing_optimization', 'resource_allocation', 'judge_selection',
        'task_complexity_adjustment', 'caws_compliance_improvement', 'claim_verification_enhancement'
    )),
    priority TEXT NOT NULL CHECK (priority IN ('critical', 'high', 'medium', 'low')),
    description TEXT NOT NULL,
    rationale TEXT NOT NULL,
    expected_impact FLOAT NOT NULL CHECK (expected_impact >= 0.0 AND expected_impact <= 1.0),
    implementation_effort TEXT NOT NULL CHECK (implementation_effort IN (
        'trivial', 'simple', 'moderate', 'complex', 'very_complex'
    )),
    evidence JSONB NOT NULL DEFAULT '[]'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'in_progress', 'completed', 'rejected')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    implemented_at TIMESTAMPTZ,
    implemented_by TEXT
);

-- Indexes for learning recommendations
CREATE INDEX idx_learning_recommendations_type ON learning_recommendations (recommendation_type);

CREATE INDEX idx_learning_recommendations_priority ON learning_recommendations (priority);

CREATE INDEX idx_learning_recommendations_status ON learning_recommendations (status);

CREATE INDEX idx_learning_recommendations_created_at ON learning_recommendations (created_at);

-- Performance benchmarks table for model evaluation
CREATE TABLE performance_benchmarks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    benchmark_id TEXT NOT NULL,
    model_id TEXT NOT NULL,
    task_surface TEXT NOT NULL,
    benchmark_type TEXT NOT NULL CHECK (benchmark_type IN ('micro', 'macro', 'smoke_test', 'capability_assessment', 'comparative')),

-- Benchmark scores
scores JSONB NOT NULL, -- {taskCompletionRate, cawsComplianceScore, efficiencyRating, ...}
metrics JSONB NOT NULL, -- {latencyP95Ms, rewardHackingIncidents, ...}

-- Benchmark metadata
duration_seconds INTEGER NOT NULL,
dataset_version TEXT,
environment_info JSONB,

-- Results
passed BOOLEAN NOT NULL, failure_reason TEXT,

-- Timestamps
started_at TIMESTAMPTZ NOT NULL,
    completed_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for performance benchmarks
CREATE INDEX idx_performance_benchmarks_benchmark_id ON performance_benchmarks (benchmark_id);

CREATE INDEX idx_performance_benchmarks_model_id ON performance_benchmarks (model_id);

CREATE INDEX idx_performance_benchmarks_task_surface ON performance_benchmarks (task_surface);

CREATE INDEX idx_performance_benchmarks_type ON performance_benchmarks (benchmark_type);

CREATE INDEX idx_performance_benchmarks_passed ON performance_benchmarks (passed);

CREATE INDEX idx_performance_benchmarks_completed_at ON performance_benchmarks (completed_at);

-- Composite index for model performance tracking
CREATE INDEX idx_performance_benchmarks_model_surface_time ON performance_benchmarks (
    model_id,
    task_surface,
    completed_at
);

-- Model performance summary view
CREATE VIEW model_performance_summary AS
WITH latest_benchmarks AS (
    SELECT DISTINCT ON (model_id, task_surface)
        model_id,
        task_surface,
        scores,
        metrics,
        completed_at,
        passed
    FROM performance_benchmarks
    WHERE benchmark_type IN ('macro', 'capability_assessment')
    ORDER BY model_id, task_surface, completed_at DESC
)
SELECT 
    model_id,
    task_surface,
    scores->>'taskCompletionRate'::float as task_completion_rate,
    scores->>'cawsComplianceScore'::float as caws_compliance_score,
    scores->>'efficiencyRating'::float as efficiency_rating,
    metrics->>'latencyP95Ms'::integer as latency_p95_ms,
    metrics->>'rewardHackingIncidents'::integer as reward_hacking_incidents,
    passed,
    completed_at
FROM latest_benchmarks;

-- Functions for learning signal analysis

-- Function to get performance trends for an entity
CREATE OR REPLACE FUNCTION get_performance_trends(
    entity_type TEXT,
    entity_id TEXT,
    time_window_hours INTEGER DEFAULT 24
)
RETURNS TABLE (
    time_bucket TIMESTAMPTZ,
    total_signals BIGINT,
    avg_quality_score FLOAT,
    avg_latency_ms FLOAT,
    success_rate FLOAT,
    dissent_rate FLOAT
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        DATE_TRUNC('hour', ls.created_at) as time_bucket,
        COUNT(*) as total_signals,
        AVG(ls.quality_score) as avg_quality_score,
        AVG(ls.latency_ms) as avg_latency_ms,
        COUNT(*) FILTER (WHERE ls.outcome_type = 'success') * 100.0 / COUNT(*) as success_rate,
        COUNT(*) FILTER (WHERE jsonb_array_length(ls.dissent_data) > 0) * 100.0 / COUNT(*) as dissent_rate
    FROM learning_signals ls
    WHERE ls.created_at >= NOW() - INTERVAL '1 hour' * time_window_hours
    AND (
        (entity_type = 'task_type' AND ls.task_id::text LIKE '%' || entity_id || '%')
        OR (entity_type = 'judge' AND ls.dissent_data @> ('[{"judge_id": "' || entity_id || '"}]')::jsonb)
        OR (entity_type = 'system' AND true)
    )
    GROUP BY DATE_TRUNC('hour', ls.created_at)
    ORDER BY time_bucket;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate recommendation confidence
CREATE OR REPLACE FUNCTION calculate_recommendation_confidence(
    similar_signals_count INTEGER,
    success_rate FLOAT
)
RETURNS FLOAT AS $$
BEGIN
    IF similar_signals_count = 0 THEN
        RETURN 0.5; -- Default confidence with no data
    END IF;
    
    -- Confidence based on success rate and sample size
    DECLARE
        sample_confidence FLOAT := LEAST(similar_signals_count::FLOAT / 100.0, 1.0);
        confidence FLOAT := success_rate * 0.7 + sample_confidence * 0.3;
    BEGIN
        RETURN confidence;
    END;
END;
$$ LANGUAGE plpgsql;

-- Trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_learning_signals_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_learning_signals_updated_at
    BEFORE UPDATE ON learning_signals
    FOR EACH ROW
    EXECUTE FUNCTION update_learning_signals_updated_at();

CREATE TRIGGER trigger_learning_recommendations_updated_at
    BEFORE UPDATE ON learning_recommendations
    FOR EACH ROW
    EXECUTE FUNCTION update_learning_signals_updated_at();

-- Add comments for documentation
COMMENT ON
TABLE learning_signals IS 'Captures learning signals from council decisions for adaptive routing and performance tracking';

COMMENT ON
TABLE learning_recommendations IS 'Stores learning recommendations generated from signal analysis';

COMMENT ON
TABLE performance_benchmarks IS 'Stores benchmark results for model performance evaluation';

COMMENT ON COLUMN learning_signals.outcome_data IS 'Serialized TaskOutcome enum data';

COMMENT ON COLUMN learning_signals.dissent_data IS 'Array of JudgeDissent objects';

COMMENT ON COLUMN learning_signals.task_complexity IS 'Serialized TaskComplexity assessment';

COMMENT ON COLUMN learning_signals.worker_performance IS 'Optional serialized WorkerPerformanceMetrics';

COMMENT ON VIEW learning_signals_performance IS 'Hourly aggregated performance metrics from learning signals';

COMMENT ON VIEW judge_performance_metrics IS 'Daily aggregated judge performance metrics';

COMMENT ON VIEW resource_usage_trends IS 'Hourly resource usage trends and thermal status';

COMMENT ON VIEW model_performance_summary IS 'Latest benchmark results per model and task surface';
