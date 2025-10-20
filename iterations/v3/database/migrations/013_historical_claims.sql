-- Migration: Historical Claims Schema
-- Version: 013
-- Description: Add historical claims table for claim extraction and verification
-- Author: @darianrosebrook
-- Date: 2025-10-21
--
-- This migration creates:
-- - Historical claims table for storing verified claims
-- - Claim verification statuses and types
-- - Cross-references and metadata storage
-- - Performance indexes for claim lookups

BEGIN;

-- ============================================================================
-- CLAIM VERIFICATION STATUS ENUM
-- ============================================================================

CREATE TYPE claim_verification_status AS ENUM (
    'unverified',
    'pending_verification',
    'verified_true',
    'verified_false',
    'partially_verified',
    'conflicting_evidence',
    'needs_more_data'
);

-- ============================================================================
-- CLAIM TYPE ENUM
-- ============================================================================

CREATE TYPE claim_type AS ENUM (
    'factual_statement',
    'causal_relationship',
    'temporal_sequence',
    'quantitative_claim',
    'comparative_claim',
    'definitional_claim',
    'methodological_claim',
    'ethical_claim',
    'hypothetical_claim'
);

-- ============================================================================
-- HISTORICAL CLAIMS TABLE
-- ============================================================================

CREATE TABLE historical_claims (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    claim_text TEXT NOT NULL,
    normalized_text TEXT, -- Normalized version for fuzzy matching
    claim_hash TEXT NOT NULL UNIQUE, -- SHA-256 hash for deduplication

    -- Verification status
    verification_status claim_verification_status DEFAULT 'unverified',
    confidence_score NUMERIC(3,2), -- 0.00 to 1.00

    -- Source tracking
    source_count INTEGER DEFAULT 0,
    primary_source TEXT,
    source_references TEXT[], -- Array of source identifiers

    -- Temporal tracking
    first_seen_at TIMESTAMPTZ DEFAULT NOW(),
    last_verified_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),

    -- Classification
    claim_type claim_type,
    domain_tags TEXT[], -- e.g., ['software', 'ai', 'testing']

    -- Relationships
    related_entities TEXT[], -- Entity names mentioned in claim
    cross_references UUID[], -- IDs of related claims
    parent_claim_id UUID REFERENCES historical_claims(id), -- For claim hierarchies

    -- Evidence and validation
    validation_metadata JSONB, -- Validation results, confidence factors
    evidence_summary JSONB, -- Summary of supporting/rejecting evidence

    -- Usage tracking
    access_count INTEGER DEFAULT 0,
    last_accessed_at TIMESTAMPTZ,

    -- Constraints
    CONSTRAINT valid_confidence CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    CONSTRAINT valid_source_count CHECK (source_count >= 0),
    CONSTRAINT valid_access_count CHECK (access_count >= 0)
);

-- ============================================================================
-- INDEXES FOR PERFORMANCE
-- ============================================================================

-- Primary lookup indexes
CREATE INDEX idx_historical_claims_hash ON historical_claims(claim_hash);
CREATE INDEX idx_historical_claims_text ON historical_claims USING gin(to_tsvector('english', claim_text));
CREATE INDEX idx_historical_claims_normalized ON historical_claims USING gin(to_tsvector('english', normalized_text)) WHERE normalized_text IS NOT NULL;
CREATE INDEX idx_historical_claims_status ON historical_claims(verification_status);
CREATE INDEX idx_historical_claims_type ON historical_claims(claim_type);
CREATE INDEX idx_historical_claims_confidence ON historical_claims(confidence_score DESC);

-- Temporal indexes
CREATE INDEX idx_historical_claims_created ON historical_claims(created_at DESC);
CREATE INDEX idx_historical_claims_verified ON historical_claims(last_verified_at DESC);
CREATE INDEX idx_historical_claims_accessed ON historical_claims(last_accessed_at DESC);

-- Relationship indexes
CREATE INDEX idx_historical_claims_parent ON historical_claims(parent_claim_id) WHERE parent_claim_id IS NOT NULL;
CREATE INDEX idx_historical_claims_entities ON historical_claims USING gin(related_entities) WHERE related_entities IS NOT NULL;
CREATE INDEX idx_historical_claims_tags ON historical_claims USING gin(domain_tags) WHERE domain_tags IS NOT NULL;
CREATE INDEX idx_historical_claims_references ON historical_claims USING gin(cross_references) WHERE cross_references IS NOT NULL;

-- ============================================================================
-- CLAIM SIMILARITY SEARCH FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION find_similar_claims(
    p_claim_text TEXT,
    p_similarity_threshold FLOAT DEFAULT 0.7,
    p_limit INTEGER DEFAULT 10,
    p_min_confidence NUMERIC DEFAULT 0.0
)
RETURNS TABLE(
    id UUID,
    claim_text TEXT,
    similarity_score FLOAT,
    confidence_score NUMERIC,
    verification_status claim_verification_status,
    claim_type claim_type
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        hc.id,
        hc.claim_text,
        (1 - (hc.normalized_text <=> p_claim_text))::FLOAT as similarity_score,
        hc.confidence_score,
        hc.verification_status,
        hc.claim_type
    FROM historical_claims hc
    WHERE hc.confidence_score >= p_min_confidence
        AND hc.normalized_text IS NOT NULL
        AND (1 - (hc.normalized_text <=> p_claim_text)) >= p_similarity_threshold
    ORDER BY (hc.normalized_text <=> p_claim_text)
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- CLAIM VERIFICATION UPDATE FUNCTION
-- ============================================================================

CREATE OR REPLACE FUNCTION update_claim_verification(
    p_claim_id UUID,
    p_new_status claim_verification_status,
    p_new_confidence NUMERIC DEFAULT NULL,
    p_validation_metadata JSONB DEFAULT NULL
)
RETURNS BOOLEAN AS $$
DECLARE
    updated_rows INTEGER;
BEGIN
    UPDATE historical_claims
    SET
        verification_status = p_new_status,
        confidence_score = COALESCE(p_new_confidence, confidence_score),
        validation_metadata = COALESCE(p_validation_metadata, validation_metadata),
        last_verified_at = NOW(),
        updated_at = NOW()
    WHERE id = p_claim_id;

    GET DIAGNOSTICS updated_rows = ROW_COUNT;
    RETURN updated_rows > 0;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- CLAIM ACCESS TRACKING
-- ============================================================================

CREATE OR REPLACE FUNCTION record_claim_access(p_claim_id UUID)
RETURNS VOID AS $$
BEGIN
    UPDATE historical_claims
    SET
        access_count = access_count + 1,
        last_accessed_at = NOW()
    WHERE id = p_claim_id;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TABLE historical_claims IS
'Storage for historical claims with verification status, confidence scores, and cross-references';

COMMENT ON TYPE claim_verification_status IS
'Enumeration of possible claim verification states from unverified to fully verified';

COMMENT ON TYPE claim_type IS
'Classification of claim types for better organization and retrieval';

COMMENT ON FUNCTION find_similar_claims IS
'Find claims similar to input text using vector similarity search';

COMMENT ON FUNCTION update_claim_verification IS
'Update claim verification status with metadata tracking';

COMMENT ON FUNCTION record_claim_access IS
'Track claim usage for popularity and recency analysis';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Find similar claims
-- SELECT * FROM find_similar_claims('Machine learning models require large datasets', 0.6, 5);

-- Example 2: Update claim verification
-- SELECT update_claim_verification('uuid-here'::uuid, 'verified_true', 0.95);

-- Example 3: Record claim access
-- SELECT record_claim_access('uuid-here'::uuid);

-- Example 4: Get claims by verification status
-- SELECT * FROM historical_claims WHERE verification_status = 'verified_true' ORDER BY confidence_score DESC LIMIT 10;

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- 1. Consider adding trigram indexes for fuzzy text matching on claim_text
--    CREATE EXTENSION IF NOT EXISTS pg_trgm;
--    CREATE INDEX idx_claims_text_trgm ON historical_claims USING gin(claim_text gin_trgm_ops);

-- 2. For high-volume claim ingestion, consider partitioning by created_at:
--    CREATE TABLE historical_claims_y2025 PARTITION OF historical_claims FOR VALUES FROM ('2025-01-01') TO ('2026-01-01');

-- 3. Set up periodic cleanup of low-confidence, unverified claims:
--    DELETE FROM historical_claims WHERE verification_status = 'unverified' AND created_at < NOW() - INTERVAL '90 days';

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
