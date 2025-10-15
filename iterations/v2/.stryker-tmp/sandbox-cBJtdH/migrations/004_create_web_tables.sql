/**
 * Migration: Create Web Navigator Tables
 *
 * @author @darianrosebrook
 * @migration 004
 * @component ARBITER-008 (Web Navigator)
 *
 * Creates tables for storing web content, traversal sessions, cache, and rate limits
 * with proper indexing for efficient content retrieval and analysis.
 */

-- ============================================================================
-- Web Content Table
-- ============================================================================
-- Stores extracted web page content with metadata and quality assessment

CREATE TABLE IF NOT EXISTS web_content (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL,
    html TEXT,

-- Content identification
content_hash VARCHAR(64) NOT NULL UNIQUE,

-- Quality assessment
quality VARCHAR(20) NOT NULL DEFAULT 'unknown' CHECK (
    quality IN (
        'high',
        'medium',
        'low',
        'unknown'
    )
),

-- Metadata
metadata JSONB NOT NULL DEFAULT '{}',

-- Extraction information
extraction_type VARCHAR(20) NOT NULL CHECK (
    extraction_type IN (
        'full_page',
        'main_content',
        'specific_element',
        'metadata'
    )
),

-- Timestamps
extracted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
cached_until TIMESTAMPTZ NOT NULL,

-- Performance metrics
extraction_time_ms BIGINT NOT NULL,
    content_size_bytes INTEGER NOT NULL
);

-- Indexes for content retrieval
CREATE INDEX IF NOT EXISTS idx_web_content_url ON web_content (url);

CREATE INDEX IF NOT EXISTS idx_web_content_hash ON web_content (content_hash);

CREATE INDEX IF NOT EXISTS idx_web_content_quality ON web_content (quality);

CREATE INDEX IF NOT EXISTS idx_web_content_extracted_at ON web_content (extracted_at DESC);

CREATE INDEX IF NOT EXISTS idx_web_content_cached_until ON web_content (cached_until);

-- Full-text search on content
CREATE INDEX IF NOT EXISTS idx_web_content_fts ON web_content USING GIN (
    to_tsvector (
        'english',
        title || ' ' || content
    )
);

-- ============================================================================
-- Web Traversals Table
-- ============================================================================
-- Tracks link traversal sessions with configuration and statistics

CREATE TABLE IF NOT EXISTS web_traversals (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id VARCHAR(64) NOT NULL UNIQUE,
    start_url TEXT NOT NULL,

-- Configuration
max_depth INTEGER NOT NULL DEFAULT 3 CHECK (
    max_depth >= 1
    AND max_depth <= 10
),
max_pages INTEGER NOT NULL DEFAULT 50 CHECK (max_pages >= 1),
strategy VARCHAR(20) NOT NULL CHECK (
    strategy IN (
        'breadth_first',
        'depth_first',
        'relevance_based'
    )
),
same_domain_only BOOLEAN NOT NULL DEFAULT TRUE,
respect_robots_txt BOOLEAN NOT NULL DEFAULT TRUE,

-- Status
status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
    status IN (
        'pending',
        'running',
        'completed',
        'failed'
    )
),

-- Statistics
pages_visited INTEGER NOT NULL DEFAULT 0,
pages_skipped INTEGER NOT NULL DEFAULT 0,
errors_encountered INTEGER NOT NULL DEFAULT 0,
max_depth_reached INTEGER NOT NULL DEFAULT 0,
total_content_bytes BIGINT NOT NULL DEFAULT 0,
rate_limit_encounters INTEGER NOT NULL DEFAULT 0,

-- Timestamps
started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
completed_at TIMESTAMPTZ,

-- Error tracking
error_message TEXT,

-- Configuration metadata
config_metadata JSONB NOT NULL DEFAULT '{}' );

-- Indexes for traversal tracking
CREATE INDEX IF NOT EXISTS idx_web_traversals_session_id ON web_traversals (session_id);

CREATE INDEX IF NOT EXISTS idx_web_traversals_status ON web_traversals (status);

CREATE INDEX IF NOT EXISTS idx_web_traversals_start_url ON web_traversals (start_url);

CREATE INDEX IF NOT EXISTS idx_web_traversals_started_at ON web_traversals (started_at DESC);

-- ============================================================================
-- Web Traversal Nodes Table
-- ============================================================================
-- Stores individual nodes (URLs) visited during traversals

CREATE TABLE IF NOT EXISTS web_traversal_nodes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    traversal_id UUID NOT NULL REFERENCES web_traversals(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    depth INTEGER NOT NULL CHECK (depth >= 0),

-- Visit status
status VARCHAR(20) NOT NULL DEFAULT 'pending' CHECK (
    status IN (
        'pending',
        'visited',
        'skipped',
        'error'
    )
),

-- Content reference
content_id UUID REFERENCES web_content (id) ON DELETE SET NULL,

-- Parent node (for traversal graph)
parent_id UUID REFERENCES web_traversal_nodes (id) ON DELETE SET NULL,

-- Link information
link_text TEXT,
link_type VARCHAR(20) CHECK (
    link_type IN ('internal', 'external')
),

-- Timestamps
discovered_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
visited_at TIMESTAMPTZ,

-- Error tracking
error_message TEXT );

-- Indexes for node tracking
CREATE INDEX IF NOT EXISTS idx_web_traversal_nodes_traversal ON web_traversal_nodes (traversal_id);

CREATE INDEX IF NOT EXISTS idx_web_traversal_nodes_url ON web_traversal_nodes (url);

CREATE INDEX IF NOT EXISTS idx_web_traversal_nodes_status ON web_traversal_nodes (status);

CREATE INDEX IF NOT EXISTS idx_web_traversal_nodes_depth ON web_traversal_nodes (depth);

CREATE INDEX IF NOT EXISTS idx_web_traversal_nodes_parent ON web_traversal_nodes (parent_id);

CREATE INDEX IF NOT EXISTS idx_web_traversal_nodes_content ON web_traversal_nodes (content_id);

-- ============================================================================
-- Web Cache Table
-- ============================================================================
-- Caches web content with TTL and access tracking

CREATE TABLE IF NOT EXISTS web_cache (
    url TEXT PRIMARY KEY,
    content_id UUID NOT NULL REFERENCES web_content(id) ON DELETE CASCADE,

-- Cache lifecycle
cached_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
expires_at TIMESTAMPTZ NOT NULL,

-- Access tracking
hit_count INTEGER NOT NULL DEFAULT 0,
last_accessed TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Cache metadata
cache_size_bytes INTEGER NOT NULL,
    compression_used BOOLEAN NOT NULL DEFAULT FALSE
);

-- Indexes for cache operations
CREATE INDEX IF NOT EXISTS idx_web_cache_expires_at ON web_cache (expires_at);

CREATE INDEX IF NOT EXISTS idx_web_cache_last_accessed ON web_cache (last_accessed DESC);

CREATE INDEX IF NOT EXISTS idx_web_cache_hit_count ON web_cache (hit_count DESC);

CREATE INDEX IF NOT EXISTS idx_web_cache_content_id ON web_cache (content_id);

-- ============================================================================
-- Web Rate Limits Table
-- ============================================================================
-- Tracks rate limiting status per domain with backoff management

CREATE TABLE IF NOT EXISTS web_rate_limits (
    domain VARCHAR(255) PRIMARY KEY,

-- Rate limit status
status VARCHAR(20) NOT NULL DEFAULT 'ok' CHECK (
    status IN ('ok', 'throttled', 'blocked')
),

-- Window tracking
requests_in_window INTEGER NOT NULL DEFAULT 0,
window_start TIMESTAMPTZ NOT NULL DEFAULT NOW(),
window_end TIMESTAMPTZ NOT NULL,

-- Backoff management
backoff_until TIMESTAMPTZ,
backoff_count INTEGER NOT NULL DEFAULT 0,

-- Request tracking
last_request TIMESTAMPTZ NOT NULL DEFAULT NOW(),
total_requests BIGINT NOT NULL DEFAULT 0,

-- Error tracking
consecutive_errors INTEGER NOT NULL DEFAULT 0,
last_error_message TEXT,
last_error_at TIMESTAMPTZ,

-- Metadata
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for rate limit checks
CREATE INDEX IF NOT EXISTS idx_web_rate_limits_status ON web_rate_limits (status);

CREATE INDEX IF NOT EXISTS idx_web_rate_limits_backoff ON web_rate_limits (backoff_until);

CREATE INDEX IF NOT EXISTS idx_web_rate_limits_window_end ON web_rate_limits (window_end);

-- ============================================================================
-- Web Extraction Metrics Table
-- ============================================================================
-- Tracks performance metrics for content extraction operations

CREATE TABLE IF NOT EXISTS web_extraction_metrics (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    content_id UUID NOT NULL REFERENCES web_content(id) ON DELETE CASCADE,

-- Performance metrics
total_time_ms BIGINT NOT NULL,
fetch_time_ms BIGINT NOT NULL,
parse_time_ms BIGINT NOT NULL,
sanitize_time_ms BIGINT NOT NULL,

-- Response details
status_code INTEGER NOT NULL,
content_type VARCHAR(100),
content_length INTEGER,
redirect_count INTEGER NOT NULL DEFAULT 0,

-- Security checks
ssl_verified BOOLEAN NOT NULL,
malicious_detected BOOLEAN NOT NULL DEFAULT FALSE,
sanitization_applied BOOLEAN NOT NULL DEFAULT FALSE,

-- Timestamps
extracted_at TIMESTAMPTZ NOT NULL DEFAULT NOW() );

-- Indexes for metrics analysis
CREATE INDEX IF NOT EXISTS idx_web_extraction_metrics_content ON web_extraction_metrics (content_id);

CREATE INDEX IF NOT EXISTS idx_web_extraction_metrics_extracted_at ON web_extraction_metrics (extracted_at DESC);

CREATE INDEX IF NOT EXISTS idx_web_extraction_metrics_total_time ON web_extraction_metrics (total_time_ms);

CREATE INDEX IF NOT EXISTS idx_web_extraction_metrics_status_code ON web_extraction_metrics (status_code);

-- ============================================================================
-- Functions and Triggers
-- ============================================================================

-- Function to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_web_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply update triggers
CREATE TRIGGER update_web_rate_limits_updated_at
    BEFORE UPDATE ON web_rate_limits
    FOR EACH ROW EXECUTE FUNCTION update_web_updated_at_column();

-- Function to automatically update cache access statistics
CREATE OR REPLACE FUNCTION update_web_cache_access_stats()
RETURNS TRIGGER AS $$
BEGIN
    NEW.hit_count = OLD.hit_count + 1;
    NEW.last_accessed = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Note: Cache access trigger would be applied programmatically when reading

-- Function to clean up expired cache entries
CREATE OR REPLACE FUNCTION cleanup_expired_web_cache()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM web_cache
    WHERE expires_at < NOW();
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    RETURN deleted_count;
END;
$$ language 'plpgsql';

-- Function to update traversal statistics
CREATE OR REPLACE FUNCTION update_traversal_statistics()
RETURNS TRIGGER AS $$
BEGIN
    UPDATE web_traversals
    SET pages_visited = (
        SELECT COUNT(*) FROM web_traversal_nodes
        WHERE traversal_id = NEW.traversal_id AND status = 'visited'
    ),
    pages_skipped = (
        SELECT COUNT(*) FROM web_traversal_nodes
        WHERE traversal_id = NEW.traversal_id AND status = 'skipped'
    ),
    errors_encountered = (
        SELECT COUNT(*) FROM web_traversal_nodes
        WHERE traversal_id = NEW.traversal_id AND status = 'error'
    ),
    max_depth_reached = (
        SELECT COALESCE(MAX(depth), 0) FROM web_traversal_nodes
        WHERE traversal_id = NEW.traversal_id AND status = 'visited'
    )
    WHERE id = NEW.traversal_id;
    
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Apply traversal statistics trigger
CREATE TRIGGER update_traversal_stats_on_node_update
    AFTER INSERT OR UPDATE ON web_traversal_nodes
    FOR EACH ROW EXECUTE FUNCTION update_traversal_statistics();

-- ============================================================================
-- Views for Analytics
-- ============================================================================

-- View for cache performance analysis
CREATE OR REPLACE VIEW web_cache_performance AS
SELECT
    COUNT(*) as total_entries,
    SUM(hit_count) as total_hits,
    AVG(hit_count) as avg_hits_per_entry,
    SUM(cache_size_bytes) as total_cache_size_bytes,
    COUNT(
        CASE
            WHEN expires_at > NOW() THEN 1
        END
    ) as active_entries,
    COUNT(
        CASE
            WHEN expires_at <= NOW() THEN 1
        END
    ) as expired_entries,
    COUNT(
        CASE
            WHEN last_accessed > NOW() - INTERVAL '1 hour' THEN 1
        END
    ) as entries_accessed_last_hour,
    COUNT(
        CASE
            WHEN last_accessed > NOW() - INTERVAL '6 hours' THEN 1
        END
    ) as entries_accessed_last_6hours,
    COUNT(
        CASE
            WHEN last_accessed > NOW() - INTERVAL '12 hours' THEN 1
        END
    ) as entries_accessed_last_12hours,
    COUNT(
        CASE
            WHEN last_accessed > NOW() - INTERVAL '24 hours' THEN 1
        END
    ) as entries_accessed_last_24hours
FROM web_cache;

-- View for extraction performance analysis
CREATE OR REPLACE VIEW web_extraction_performance AS
SELECT
    DATE_TRUNC ('hour', extracted_at) as hour,
    COUNT(*) as extractions_count,
    AVG(total_time_ms) as avg_total_time_ms,
    PERCENTILE_CONT (0.50) WITHIN GROUP (
        ORDER BY total_time_ms
    ) as p50_total_time_ms,
    PERCENTILE_CONT (0.95) WITHIN GROUP (
        ORDER BY total_time_ms
    ) as p95_total_time_ms,
    PERCENTILE_CONT (0.99) WITHIN GROUP (
        ORDER BY total_time_ms
    ) as p99_total_time_ms,
    AVG(fetch_time_ms) as avg_fetch_time_ms,
    AVG(parse_time_ms) as avg_parse_time_ms,
    AVG(sanitize_time_ms) as avg_sanitize_time_ms,
    COUNT(
        CASE
            WHEN malicious_detected THEN 1
        END
    ) as malicious_detected_count,
    COUNT(
        CASE
            WHEN status_code >= 400 THEN 1
        END
    ) as error_responses_count
FROM web_extraction_metrics
GROUP BY
    DATE_TRUNC ('hour', extracted_at)
ORDER BY hour DESC;

-- View for traversal analysis
CREATE OR REPLACE VIEW web_traversal_analysis AS
SELECT
    wt.id,
    wt.session_id,
    wt.start_url,
    wt.strategy,
    wt.status,
    wt.pages_visited,
    wt.pages_skipped,
    wt.errors_encountered,
    wt.max_depth_reached,
    wt.total_content_bytes,
    wt.rate_limit_encounters,
    CASE
        WHEN wt.completed_at IS NOT NULL THEN EXTRACT(
            EPOCH
            FROM (
                    wt.completed_at - wt.started_at
                )
        ) * 1000
        ELSE NULL
    END as duration_ms,
    wt.started_at,
    wt.completed_at
FROM web_traversals wt;

-- View for rate limit monitoring
CREATE OR REPLACE VIEW web_rate_limit_status AS
SELECT
    domain,
    status,
    requests_in_window,
    backoff_until,
    backoff_count,
    consecutive_errors,
    total_requests,
    last_request,
    CASE
        WHEN backoff_until IS NOT NULL
        AND backoff_until > NOW() THEN EXTRACT(
            EPOCH
            FROM (backoff_until - NOW())
        )
        ELSE 0
    END as seconds_until_backoff_expires
FROM web_rate_limits
WHERE
    status != 'ok'
    OR backoff_until > NOW()
ORDER BY last_request DESC;

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON
TABLE web_content IS 'Stores extracted web page content with metadata and quality scores';

COMMENT ON
TABLE web_traversals IS 'Tracks link traversal sessions with configuration and statistics';

COMMENT ON
TABLE web_traversal_nodes IS 'Individual nodes (URLs) visited during traversals with graph structure';

COMMENT ON
TABLE web_cache IS 'Caches web content with TTL and access tracking (24h default)';

COMMENT ON
TABLE web_rate_limits IS 'Rate limiting tracking per domain with backoff management';

COMMENT ON
TABLE web_extraction_metrics IS 'Performance metrics for content extraction operations';

COMMENT ON COLUMN web_content.content_hash IS 'SHA-256 hash for duplicate detection';

COMMENT ON COLUMN web_content.cached_until IS 'Cache expiration (typically 24h from extraction)';

COMMENT ON COLUMN web_traversals.same_domain_only IS 'Restrict traversal to starting domain only';

COMMENT ON COLUMN web_traversals.respect_robots_txt IS 'Follow robots.txt directives';

COMMENT ON COLUMN web_cache.hit_count IS 'Number of times this cached content was accessed';

COMMENT ON COLUMN web_rate_limits.backoff_until IS 'Timestamp when backoff period ends';

-- ============================================================================
-- Rollback Section (for migration reversal)
-- ============================================================================

/*
-- Rollback commands (run in reverse order):
-- DROP VIEW IF EXISTS web_rate_limit_status;
-- DROP VIEW IF EXISTS web_traversal_analysis;
-- DROP VIEW IF EXISTS web_extraction_performance;
-- DROP VIEW IF EXISTS web_cache_performance;
-- DROP TRIGGER IF EXISTS update_traversal_stats_on_node_update ON web_traversal_nodes;
-- DROP TRIGGER IF EXISTS update_web_rate_limits_updated_at ON web_rate_limits;
-- DROP FUNCTION IF EXISTS update_traversal_statistics();
-- DROP FUNCTION IF EXISTS cleanup_expired_web_cache();
-- DROP FUNCTION IF EXISTS update_web_cache_access_stats();
-- DROP FUNCTION IF EXISTS update_web_updated_at_column();
-- DROP TABLE IF EXISTS web_extraction_metrics;
-- DROP TABLE IF EXISTS web_rate_limits;
-- DROP TABLE IF EXISTS web_cache;
-- DROP TABLE IF EXISTS web_traversal_nodes;
-- DROP TABLE IF EXISTS web_traversals;
-- DROP TABLE IF EXISTS web_content;
*/