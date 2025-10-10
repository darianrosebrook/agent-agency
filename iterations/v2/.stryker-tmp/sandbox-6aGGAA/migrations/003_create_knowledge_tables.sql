/**
 * Migration: Create Knowledge Tables
 *
 * @author @darianrosebrook
 * @migration 003
 * @component ARBITER-006 (Knowledge Seeker)
 *
 * Creates tables for storing knowledge queries, search results, and responses
 * with proper indexing for efficient querying and analysis.
 */

-- ============================================================================
-- Knowledge Queries Table
-- ============================================================================
-- Stores knowledge queries with their metadata and processing status

CREATE TABLE IF NOT EXISTS knowledge_queries (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    query_text TEXT NOT NULL,
    query_type VARCHAR(20) NOT NULL CHECK (
        query_type IN ('factual', 'explanatory', 'comparative', 'trend', 'technical')
    ),
    requester_id VARCHAR(255) NOT NULL,
    priority INTEGER NOT NULL DEFAULT 1 CHECK (priority >= 1 AND priority <= 10),
    max_results INTEGER NOT NULL DEFAULT 10 CHECK (max_results > 0),
    relevance_threshold NUMERIC(3,2) NOT NULL DEFAULT 0.5 CHECK (
        relevance_threshold >= 0 AND relevance_threshold <= 1
    ),
    timeout_ms BIGINT NOT NULL DEFAULT 30000 CHECK (timeout_ms > 0),

    -- Query context and metadata
    context JSONB NOT NULL DEFAULT '{}',
    tags TEXT[] NOT NULL DEFAULT '{}',

    -- Processing status
    status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
        status IN ('pending', 'processing', 'completed', 'failed')
    ),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,

    -- Error tracking
    error_message TEXT,
    retry_count INTEGER NOT NULL DEFAULT 0 CHECK (retry_count >= 0)
);

-- Indexes for query processing
CREATE INDEX IF NOT EXISTS idx_knowledge_queries_status ON knowledge_queries (status);
CREATE INDEX IF NOT EXISTS idx_knowledge_queries_requester ON knowledge_queries (requester_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_queries_priority ON knowledge_queries (priority DESC, created_at ASC);
CREATE INDEX IF NOT EXISTS idx_knowledge_queries_created_at ON knowledge_queries (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_knowledge_queries_tags ON knowledge_queries USING GIN (tags);

-- ============================================================================
-- Search Results Table
-- ============================================================================
-- Stores individual search results with quality scores and metadata

CREATE TABLE IF NOT EXISTS search_results (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    query_id UUID NOT NULL REFERENCES knowledge_queries(id) ON DELETE CASCADE,

    -- Result content
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    url TEXT NOT NULL,
    domain VARCHAR(255) NOT NULL,

    -- Classification
    source_type VARCHAR(20) NOT NULL CHECK (
        source_type IN ('web', 'academic', 'news', 'documentation', 'social', 'internal')
    ),

    -- Quality scores
    relevance_score NUMERIC(3,2) NOT NULL CHECK (
        relevance_score >= 0 AND relevance_score <= 1
    ),
    credibility_score NUMERIC(3,2) NOT NULL CHECK (
        credibility_score >= 0 AND credibility_score <= 1
    ),
    quality VARCHAR(20) NOT NULL DEFAULT 'medium' CHECK (
        quality IN ('high', 'medium', 'low', 'unreliable')
    ),

    -- Provider information
    provider VARCHAR(100) NOT NULL,
    provider_metadata JSONB NOT NULL DEFAULT '{}',

    -- Temporal information
    published_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Content hash for duplicate detection
    content_hash VARCHAR(64) UNIQUE,

    -- Full-text search
    search_vector TSVECTOR GENERATED ALWAYS AS (
        setweight(to_tsvector('english', title), 'A') ||
        setweight(to_tsvector('english', content), 'B')
    ) STORED
);

-- Indexes for result processing and querying
CREATE INDEX IF NOT EXISTS idx_search_results_query_id ON search_results (query_id);
CREATE INDEX IF NOT EXISTS idx_search_results_relevance ON search_results (relevance_score DESC);
CREATE INDEX IF NOT EXISTS idx_search_results_credibility ON search_results (credibility_score DESC);
CREATE INDEX IF NOT EXISTS idx_search_results_quality ON search_results (quality);
CREATE INDEX IF NOT EXISTS idx_search_results_provider ON search_results (provider);
CREATE INDEX IF NOT EXISTS idx_search_results_domain ON search_results (domain);
CREATE INDEX IF NOT EXISTS idx_search_results_source_type ON search_results (source_type);
CREATE INDEX IF NOT EXISTS idx_search_results_published_at ON search_results (published_at DESC);
CREATE INDEX IF NOT EXISTS idx_search_results_content_hash ON search_results (content_hash);

-- Full-text search index
CREATE INDEX IF NOT EXISTS idx_search_results_fts ON search_results USING GIN (search_vector);

-- ============================================================================
-- Knowledge Responses Table
-- ============================================================================
-- Stores aggregated responses with metadata and performance metrics

CREATE TABLE IF NOT EXISTS knowledge_responses (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    query_id UUID NOT NULL REFERENCES knowledge_queries(id) ON DELETE CASCADE,

    -- Response content
    summary TEXT NOT NULL,
    confidence NUMERIC(3,2) NOT NULL CHECK (
        confidence >= 0 AND confidence <= 1
    ),

    -- Processing metadata
    sources_used TEXT[] NOT NULL DEFAULT '{}',
    total_results_found INTEGER NOT NULL DEFAULT 0,
    results_filtered INTEGER NOT NULL DEFAULT 0,
    processing_time_ms BIGINT NOT NULL,
    cache_used BOOLEAN NOT NULL DEFAULT FALSE,
    providers_queried TEXT[] NOT NULL DEFAULT '{}',

    -- Response quality metrics
    relevance_score_avg NUMERIC(3,2),
    credibility_score_avg NUMERIC(3,2),
    diversity_score NUMERIC(3,2),

    -- Timestamps
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for response analysis
CREATE INDEX IF NOT EXISTS idx_knowledge_responses_query_id ON knowledge_responses (query_id);
CREATE INDEX IF NOT EXISTS idx_knowledge_responses_confidence ON knowledge_responses (confidence DESC);
CREATE INDEX IF NOT EXISTS idx_knowledge_responses_processing_time ON knowledge_responses (processing_time_ms);
CREATE INDEX IF NOT EXISTS idx_knowledge_responses_cache_used ON knowledge_responses (cache_used);
CREATE INDEX IF NOT EXISTS idx_knowledge_responses_created_at ON knowledge_responses (created_at DESC);

-- ============================================================================
-- Search Provider Health Table
-- ============================================================================
-- Tracks health and performance metrics for search providers

CREATE TABLE IF NOT EXISTS search_provider_health (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    provider_name VARCHAR(100) NOT NULL UNIQUE,

    -- Health status
    available BOOLEAN NOT NULL DEFAULT TRUE,
    last_health_check TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    consecutive_failures INTEGER NOT NULL DEFAULT 0,

    -- Performance metrics
    avg_response_time_ms NUMERIC(8,2),
    error_rate NUMERIC(5,4) DEFAULT 0,
    requests_this_minute INTEGER NOT NULL DEFAULT 0,
    requests_this_hour INTEGER NOT NULL DEFAULT 0,

    -- Rate limiting
    rate_limit_remaining INTEGER,
    rate_limit_reset_at TIMESTAMPTZ,

    -- Error tracking
    last_error_message TEXT,
    last_error_at TIMESTAMPTZ,

    -- Metadata
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for health monitoring
CREATE INDEX IF NOT EXISTS idx_search_provider_health_available ON search_provider_health (available);
CREATE INDEX IF NOT EXISTS idx_search_provider_health_last_check ON search_provider_health (last_health_check DESC);

-- ============================================================================
-- Knowledge Cache Table
-- ============================================================================
-- Caches query results and responses to reduce external API calls

CREATE TABLE IF NOT EXISTS knowledge_cache (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cache_key VARCHAR(64) NOT NULL UNIQUE,
    cache_type VARCHAR(20) NOT NULL CHECK (
        cache_type IN ('query_result', 'search_result', 'response')
    ),

    -- Cache content
    content JSONB NOT NULL,
    metadata JSONB NOT NULL DEFAULT '{}',

    -- Cache lifecycle
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    access_count INTEGER NOT NULL DEFAULT 0,
    last_accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Cache performance
    size_bytes INTEGER NOT NULL,
    compression_ratio NUMERIC(3,2) DEFAULT 1.0
);

-- Indexes for cache operations
CREATE INDEX IF NOT EXISTS idx_knowledge_cache_key ON knowledge_cache (cache_key);
CREATE INDEX IF NOT EXISTS idx_knowledge_cache_type ON knowledge_cache (cache_type);
CREATE INDEX IF NOT EXISTS idx_knowledge_cache_expires ON knowledge_cache (expires_at);
CREATE INDEX IF NOT EXISTS idx_knowledge_cache_access ON knowledge_cache (last_accessed_at DESC, access_count DESC);

-- ============================================================================
-- Functions and Triggers
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_knowledge_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers
CREATE TRIGGER update_search_provider_health_updated_at
    BEFORE UPDATE ON search_provider_health
    FOR EACH ROW EXECUTE FUNCTION update_knowledge_updated_at_column();

-- Function to automatically update cache access statistics
CREATE OR REPLACE FUNCTION update_cache_access_stats()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE knowledge_cache
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

-- Insert default search provider health records
INSERT INTO search_provider_health (provider_name, available)
VALUES
    ('google', true),
    ('bing', true),
    ('duckduckgo', true),
    ('arxiv', true),
    ('pubmed', true)
ON CONFLICT (provider_name) DO NOTHING;

-- ============================================================================
-- Views for Analytics
-- ============================================================================

-- View for query performance analysis
CREATE OR REPLACE VIEW query_performance AS
SELECT
    kq.id,
    kq.query_type,
    kq.priority,
    kq.status,
    kq.created_at,
    kq.processed_at,
    CASE
        WHEN kq.processed_at IS NOT NULL
        THEN EXTRACT(EPOCH FROM (kq.processed_at - kq.created_at)) * 1000
        ELSE NULL
    END as processing_time_ms,
    kr.confidence,
    kr.total_results_found,
    kr.results_filtered,
    kr.processing_time_ms as response_processing_time_ms,
    kr.cache_used,
    array_length(kr.providers_queried, 1) as providers_used
FROM knowledge_queries kq
LEFT JOIN knowledge_responses kr ON kq.id = kr.query_id;

-- View for result quality analysis
CREATE OR REPLACE VIEW result_quality_analysis AS
SELECT
    sr.query_id,
    COUNT(*) as total_results,
    AVG(sr.relevance_score) as avg_relevance,
    AVG(sr.credibility_score) as avg_credibility,
    COUNT(CASE WHEN sr.quality = 'high' THEN 1 END) as high_quality_count,
    COUNT(CASE WHEN sr.quality = 'medium' THEN 1 END) as medium_quality_count,
    COUNT(CASE WHEN sr.quality = 'low' THEN 1 END) as low_quality_count,
    COUNT(CASE WHEN sr.quality = 'unreliable' THEN 1 END) as unreliable_count,
    COUNT(DISTINCT sr.domain) as unique_domains,
    COUNT(DISTINCT sr.source_type) as unique_source_types,
    array_agg(DISTINCT sr.provider) as providers_used
FROM search_results sr
GROUP BY sr.query_id;

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON TABLE knowledge_queries IS 'Stores knowledge queries with processing metadata and status';
COMMENT ON TABLE search_results IS 'Individual search results with quality scores and content';
COMMENT ON TABLE knowledge_responses IS 'Aggregated responses with performance and quality metrics';
COMMENT ON TABLE search_provider_health IS 'Health monitoring and rate limiting for search providers';
COMMENT ON TABLE knowledge_cache IS 'Caching layer for queries and results to reduce API calls';

COMMENT ON COLUMN knowledge_queries.relevance_threshold IS 'Minimum relevance score for results (0-1)';
COMMENT ON COLUMN search_results.relevance_score IS 'How relevant this result is to the query (0-1)';
COMMENT ON COLUMN search_results.credibility_score IS 'Credibility assessment of the source (0-1)';
COMMENT ON COLUMN knowledge_responses.confidence IS 'Overall confidence in the response (0-1)';
COMMENT ON COLUMN knowledge_cache.cache_key IS 'Hash-based key for cache lookups';

-- ============================================================================
-- Rollback Section (for migration reversal)
-- ============================================================================

/*
-- Rollback commands (run in reverse order):
-- DROP VIEW IF EXISTS result_quality_analysis;
-- DROP VIEW IF EXISTS query_performance;
-- DROP TABLE IF EXISTS knowledge_cache;
-- DROP TABLE IF EXISTS search_provider_health;
-- DROP TABLE IF EXISTS knowledge_responses;
-- DROP TABLE IF EXISTS search_results;
-- DROP TABLE IF EXISTS knowledge_queries;
-- DROP FUNCTION IF EXISTS update_knowledge_updated_at_column();
-- DROP FUNCTION IF EXISTS update_cache_access_stats();
*/
