/**
 * Migration: Create Verification Tables
 *
 * @author @darianrosebrook
 * @migration 004
 * @component ARBITER-007 (Verification Engine)
 *
 * Creates tables for storing verification requests, results, and method performance
 * with proper indexing for efficient querying and analysis.
 */

-- ============================================================================
-- Verification Requests Table
-- ============================================================================
-- Stores verification requests with their metadata and processing status

CREATE TABLE IF NOT EXISTS verification_requests (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content TEXT NOT NULL,
    source TEXT,
    context TEXT,
    priority VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (
        priority IN ('low', 'medium', 'high', 'critical')
    ),
    timeout_ms BIGINT NOT NULL DEFAULT 30000 CHECK (timeout_ms > 0),
    verification_types TEXT[] NOT NULL DEFAULT '{}',
    metadata JSONB NOT NULL DEFAULT '{}',
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
        status IN ('pending', 'processing', 'completed', 'failed', 'cancelled')
    ),

    -- Processing metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    processing_time_ms BIGINT,

    -- Error tracking
    error_message TEXT,
    error_code VARCHAR(50),
    retry_count INTEGER NOT NULL DEFAULT 0 CHECK (retry_count >= 0),
    max_retries INTEGER NOT NULL DEFAULT 3 CHECK (max_retries >= 0),

    -- Cache key for result caching
    cache_key VARCHAR(128) UNIQUE,

    -- Full-text search
    content_vector TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('english', content), 'A') ||
        setweight(to_tsvector('english', COALESCE(context, '')), 'B')
    ) STORED
);

-- Indexes for request processing
CREATE INDEX IF NOT EXISTS idx_verification_requests_status ON verification_requests (status);
CREATE INDEX IF NOT EXISTS idx_verification_requests_priority ON verification_requests (priority DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_verification_requests_created_at ON verification_requests (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_verification_requests_source ON verification_requests (source);
CREATE INDEX IF NOT EXISTS idx_verification_requests_cache_key ON verification_requests (cache_key);

-- Full-text search index
CREATE INDEX IF NOT EXISTS idx_verification_requests_fts ON verification_requests USING GIN (content_vector);

-- Partial index for active requests
CREATE INDEX IF NOT EXISTS idx_verification_requests_active
    ON verification_requests (created_at DESC)
    WHERE status IN ('pending', 'processing');

-- ============================================================================
-- Verification Results Table
-- ============================================================================
-- Stores verification results with confidence scores and evidence

CREATE TABLE IF NOT EXISTS verification_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    request_id UUID NOT NULL REFERENCES verification_requests(id) ON DELETE CASCADE,

    -- Result data
    verdict VARCHAR(30) NOT NULL CHECK (
        verdict IN ('verified_true', 'verified_false', 'partially_true', 'unverified', 'contradictory', 'insufficient_data')
    ),
    confidence NUMERIC(3,2) NOT NULL CHECK (
        confidence >= 0 AND confidence <= 1
    ),

    -- Reasoning and evidence
    reasoning TEXT[] NOT NULL DEFAULT '{}',
    supporting_evidence JSONB NOT NULL DEFAULT '[]',
    contradictory_evidence JSONB NOT NULL DEFAULT '[]',

    -- Method results
    verification_methods JSONB NOT NULL DEFAULT '[]',

    -- Processing metadata
    processing_time_ms BIGINT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Quality metrics
    evidence_quality_score NUMERIC(3,2),
    consensus_strength NUMERIC(3,2),
    uncertainty_bounds JSONB,

    -- Error information
    error_message TEXT,
    error_code VARCHAR(50),

    -- Audit trail
    verified_by VARCHAR(100),
    verification_version VARCHAR(20),

    -- Result cache
    result_hash VARCHAR(64) UNIQUE,
    cache_expiry TIMESTAMPTZ
);

-- Indexes for result analysis
CREATE INDEX IF NOT EXISTS idx_verification_results_request_id ON verification_results (request_id);
CREATE INDEX IF NOT EXISTS idx_verification_results_verdict ON verification_results (verdict);
CREATE INDEX IF NOT EXISTS idx_verification_results_confidence ON verification_results (confidence DESC);
CREATE INDEX IF NOT EXISTS idx_verification_results_created_at ON verification_results (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_verification_results_result_hash ON verification_results (result_hash);
CREATE INDEX IF NOT EXISTS idx_verification_results_cache_expiry ON verification_results (cache_expiry) WHERE cache_expiry IS NOT NULL;

-- ============================================================================
-- Verification Methods Table
-- ============================================================================
-- Tracks performance and health of individual verification methods

CREATE TABLE IF NOT EXISTS verification_methods (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    method_type VARCHAR(30) NOT NULL UNIQUE CHECK (
        method_type IN ('fact_checking', 'source_credibility', 'cross_reference', 'consistency_check', 'logical_validation', 'statistical_validation')
    ),

    -- Configuration
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    priority INTEGER NOT NULL DEFAULT 1 CHECK (priority >= 1 AND priority <= 10),
    timeout_ms BIGINT NOT NULL DEFAULT 10000 CHECK (timeout_ms > 0),
    config JSONB NOT NULL DEFAULT '{}',

    -- Health and performance tracking
    last_used_at TIMESTAMPTZ,
    total_requests BIGINT NOT NULL DEFAULT 0,
    successful_requests BIGINT NOT NULL DEFAULT 0,
    failed_requests BIGINT NOT NULL DEFAULT 0,
    average_processing_time_ms NUMERIC(8,2),
    last_error_message TEXT,
    last_error_at TIMESTAMPTZ,

    -- Quality metrics
    accuracy_rate NUMERIC(5,4),
    false_positive_rate NUMERIC(5,4),
    precision_rate NUMERIC(5,4),
    recall_rate NUMERIC(5,4),

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    version VARCHAR(20) NOT NULL DEFAULT '1.0.0'
);

-- Indexes for method monitoring
CREATE INDEX IF NOT EXISTS idx_verification_methods_enabled ON verification_methods (enabled);
CREATE INDEX IF NOT EXISTS idx_verification_methods_priority ON verification_methods (priority DESC);
CREATE INDEX IF NOT EXISTS idx_verification_methods_type ON verification_methods (method_type);
CREATE INDEX IF NOT EXISTS idx_verification_methods_last_used ON verification_methods (last_used_at DESC);

-- ============================================================================
-- Verification Evidence Table
-- ============================================================================
-- Stores detailed evidence used in verification processes

CREATE TABLE IF NOT EXISTS verification_evidence (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    result_id UUID NOT NULL REFERENCES verification_results(id) ON DELETE CASCADE,
    request_id UUID NOT NULL REFERENCES verification_requests(id) ON DELETE CASCADE,

    -- Evidence details
    source_url TEXT,
    source_title TEXT,
    source_publisher VARCHAR(255),
    content_excerpt TEXT,
    content_hash VARCHAR(64),

    -- Quality assessments
    relevance_score NUMERIC(3,2) NOT NULL CHECK (
        relevance_score >= 0 AND relevance_score <= 1
    ),
    credibility_score NUMERIC(3,2) NOT NULL CHECK (
        credibility_score >= 0 AND credibility_score <= 1
    ),

    -- Classification
    supporting_claim BOOLEAN NOT NULL,
    evidence_type VARCHAR(30) NOT NULL DEFAULT 'general' CHECK (
        evidence_type IN ('factual', 'statistical', 'testimonial', 'logical', 'anecdotal', 'general')
    ),

    -- Temporal information
    publish_date TIMESTAMPTZ,
    access_date TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Verification method that found this evidence
    verification_method VARCHAR(30) CHECK (
        verification_method IN ('fact_checking', 'source_credibility', 'cross_reference', 'consistency_check', 'logical_validation', 'statistical_validation')
    ),

    -- Metadata
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Full-text search
    content_vector TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('english', COALESCE(content_excerpt, '')), 'A') ||
        setweight(to_tsvector('english', COALESCE(source_title, '')), 'B')
    ) STORED
);

-- Indexes for evidence analysis
CREATE INDEX IF NOT EXISTS idx_verification_evidence_result_id ON verification_evidence (result_id);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_request_id ON verification_evidence (request_id);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_relevance ON verification_evidence (relevance_score DESC);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_credibility ON verification_evidence (credibility_score DESC);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_supporting ON verification_evidence (supporting_claim);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_method ON verification_evidence (verification_method);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_content_hash ON verification_evidence (content_hash);
CREATE INDEX IF NOT EXISTS idx_verification_evidence_publish_date ON verification_evidence (publish_date DESC);

-- Full-text search index
CREATE INDEX IF NOT EXISTS idx_verification_evidence_fts ON verification_evidence USING GIN (content_vector);

-- ============================================================================
-- Verification Cache Table
-- ============================================================================
-- Caches verification results to improve performance

CREATE TABLE IF NOT EXISTS verification_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cache_key VARCHAR(128) NOT NULL UNIQUE,
    request_hash VARCHAR(64) NOT NULL UNIQUE,

    -- Cached result
    result_data JSONB NOT NULL,
    result_hash VARCHAR(64) NOT NULL,

    -- Cache lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Cache performance
    size_bytes INTEGER NOT NULL,
    compression_ratio NUMERIC(3,2) DEFAULT 1.0,

    -- Metadata
    request_count INTEGER NOT NULL DEFAULT 1,
    average_confidence NUMERIC(3,2),
    cache_hit_rate NUMERIC(3,2) DEFAULT 1.0
);

-- Indexes for cache operations
CREATE INDEX IF NOT EXISTS idx_verification_cache_key ON verification_cache (cache_key);
CREATE INDEX IF NOT EXISTS idx_verification_cache_request_hash ON verification_cache (request_hash);
CREATE INDEX IF NOT EXISTS idx_verification_cache_expires ON verification_cache (expires_at);
CREATE INDEX IF NOT EXISTS idx_verification_cache_access ON verification_cache (last_accessed_at DESC, access_count DESC);

-- ============================================================================
-- Functions and Triggers
-- ============================================================================

-- Function to update verification method stats
CREATE OR REPLACE FUNCTION update_verification_method_stats()
RETURNS TRIGGER AS $$
BEGIN
    -- Update timestamps
    NEW.updated_at = NOW();

    -- Update performance metrics if this is an update
    IF TG_OP = 'UPDATE' THEN
        -- Recalculate accuracy rate
        IF NEW.total_requests > 0 THEN
            NEW.accuracy_rate = NEW.successful_requests::NUMERIC / NEW.total_requests;
        END IF;
    END IF;

    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update trigger
CREATE TRIGGER update_verification_methods_stats
    BEFORE UPDATE ON verification_methods
    FOR EACH ROW EXECUTE FUNCTION update_verification_method_stats();

-- Function to update cache access statistics
CREATE OR REPLACE FUNCTION update_verification_cache_access()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE verification_cache
    SET access_count = access_count + 1,
        last_accessed_at = NOW()
    WHERE id = NEW.id;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Note: Cache access trigger would be applied when reading from cache

-- ============================================================================
-- Initial Data
-- ============================================================================

-- Insert default verification methods
INSERT INTO verification_methods (method_type, enabled, priority, timeout_ms, config) VALUES
    ('fact_checking', true, 1, 10000, '{"providers": ["google_fact_check", "snopes", "factcheck_org"]}'),
    ('source_credibility', true, 2, 5000, '{"credibility_database": "media_bias_fact_check"}'),
    ('cross_reference', true, 3, 15000, '{"max_sources": 5, "min_consensus": 0.7}'),
    ('consistency_check', true, 4, 8000, '{"logic_engine": "default"}'),
    ('logical_validation', true, 5, 12000, '{"reasoning_engine": "symbolic"}'),
    ('statistical_validation', true, 6, 10000, '{"statistical_tests": ["chi_square", "correlation", "significance"]}'),
    ('insufficient_data', false, 10, 1000, '{}')  -- Disabled by default, used for fallback
ON CONFLICT (method_type) DO NOTHING;

-- ============================================================================
-- Views for Analytics and Monitoring
-- ============================================================================

-- View for verification performance analysis
CREATE OR REPLACE VIEW verification_performance AS
SELECT
    vr.id,
    vr.verdict,
    vr.confidence,
    vr.processing_time_ms,
    vr.created_at,
    vreq.priority,
    vreq.status,
    vreq.retry_count,
    array_length(vr.reasoning, 1) as reasoning_count,
    jsonb_array_length(vr.supporting_evidence) as supporting_evidence_count,
    jsonb_array_length(vr.contradictory_evidence) as contradictory_evidence_count,
    jsonb_array_length(vr.verification_methods) as methods_used
FROM verification_results vr
JOIN verification_requests vreq ON vr.request_id = vreq.id;

-- View for method performance tracking
CREATE OR REPLACE VIEW method_performance AS
SELECT
    method_type,
    enabled,
    priority,
    total_requests,
    successful_requests,
    failed_requests,
    CASE
        WHEN total_requests > 0
        THEN ROUND((successful_requests::NUMERIC / total_requests) * 100, 2)
        ELSE 0
    END as success_rate_percent,
    average_processing_time_ms,
    accuracy_rate,
    false_positive_rate,
    last_used_at,
    last_error_at
FROM verification_methods
ORDER BY priority ASC, success_rate_percent DESC;

-- View for evidence quality analysis
CREATE OR REPLACE VIEW evidence_quality_analysis AS
SELECT
    verification_method,
    COUNT(*) as total_evidence,
    AVG(relevance_score) as avg_relevance,
    AVG(credibility_score) as avg_credibility,
    COUNT(CASE WHEN supporting_claim THEN 1 END) as supporting_count,
    COUNT(CASE WHEN NOT supporting_claim THEN 1 END) as contradicting_count,
    COUNT(DISTINCT result_id) as unique_results,
    MIN(access_date) as earliest_evidence,
    MAX(access_date) as latest_evidence
FROM verification_evidence
GROUP BY verification_method
ORDER BY total_evidence DESC;

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON TABLE verification_requests IS 'Stores verification requests with content, metadata, and processing status';
COMMENT ON TABLE verification_results IS 'Stores verification results with confidence scores, evidence, and reasoning';
COMMENT ON TABLE verification_methods IS 'Tracks performance and configuration of individual verification methods';
COMMENT ON TABLE verification_evidence IS 'Stores detailed evidence used in verification processes';
COMMENT ON TABLE verification_cache IS 'Caches verification results for improved performance';

COMMENT ON COLUMN verification_requests.verification_types IS 'Array of verification method types to apply';
COMMENT ON COLUMN verification_results.confidence IS 'Overall confidence score (0.0-1.0) in the verification result';
COMMENT ON COLUMN verification_evidence.relevance_score IS 'How relevant this evidence is to the claim (0.0-1.0)';
COMMENT ON COLUMN verification_evidence.credibility_score IS 'Credibility score of the evidence source (0.0-1.0)';
COMMENT ON COLUMN verification_cache.cache_hit_rate IS 'Percentage of cache accesses that were hits';

-- ============================================================================
-- Rollback Section (for migration reversal)
-- ============================================================================

/*
-- Rollback commands (run in reverse order):
-- DROP VIEW IF EXISTS evidence_quality_analysis;
-- DROP VIEW IF EXISTS method_performance;
-- DROP VIEW IF EXISTS verification_performance;
-- DROP TABLE IF EXISTS verification_cache;
-- DROP TABLE IF EXISTS verification_evidence;
-- DROP TABLE IF EXISTS verification_methods;
-- DROP TABLE IF EXISTS verification_results;
-- DROP TABLE IF EXISTS verification_requests;
-- DROP FUNCTION IF EXISTS update_verification_method_stats();
-- DROP FUNCTION IF EXISTS update_verification_cache_access();
*/
