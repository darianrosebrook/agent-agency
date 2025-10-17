-- Migration: Add External Knowledge Sources
-- Version: 017
-- Description: Add support for external knowledge sources (Wikidata, WordNet) to existing schema
-- Author: @darianrosebrook
-- Date: 2025-01-27
--
-- This migration adds support for external knowledge sources without creating new tables.
-- Uses existing agent_capabilities_graph table with EXTERNAL_KNOWLEDGE capability type.

BEGIN;

-- ============================================================================
-- ADD EXTERNAL KNOWLEDGE SUPPORT
-- ============================================================================

-- Add external knowledge to existing entity_type enum (if not already added in 016)
DO $$ BEGIN
    ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'EXTERNAL_KNOWLEDGE';
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- ============================================================================
-- UPDATE HYBRID SEARCH VIEW (ADD EXTERNAL KNOWLEDGE)
-- ============================================================================

-- The view extension is already handled in migration 016
-- This migration ensures the view includes external knowledge properly

-- ============================================================================
-- ADD INDEXES FOR EXTERNAL KNOWLEDGE
-- ============================================================================

-- Index for external knowledge sources (subset of existing indexes)
CREATE INDEX idx_external_knowledge_source ON agent_capabilities_graph USING GIN ((metadata ->> 'source'))
WHERE
    capability_type = 'EXTERNAL_KNOWLEDGE';

CREATE INDEX idx_external_knowledge_entity_id ON agent_capabilities_graph USING GIN ((metadata ->> 'entity_id'))
WHERE
    capability_type = 'EXTERNAL_KNOWLEDGE';

-- Index for knowledge confidence (important for decay system)
CREATE INDEX idx_external_knowledge_confidence ON agent_capabilities_graph (confidence DESC)
WHERE
    capability_type = 'EXTERNAL_KNOWLEDGE';

-- ============================================================================
-- ADD KNOWLEDGE MANAGEMENT FUNCTIONS
-- ============================================================================

-- Function to check if external knowledge is indexed
CREATE OR REPLACE FUNCTION is_knowledge_indexed(
    p_source VARCHAR(50),
    p_entity_id VARCHAR(255)
) RETURNS BOOLEAN AS $$
BEGIN
    RETURN EXISTS (
        SELECT 1 FROM agent_capabilities_graph
        WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
            AND metadata->>'source' = p_source
            AND metadata->>'entity_id' = p_entity_id
    );
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to get knowledge entry by source and entity_id
CREATE OR REPLACE FUNCTION get_knowledge_entry(
    p_source VARCHAR(50),
    p_entity_id VARCHAR(255)
) RETURNS TABLE(
    id UUID,
    capability_name TEXT,
    embedding vector(768),
    confidence DECIMAL(3,2),
    metadata JSONB,
    last_updated TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        acg.id,
        acg.capability_name,
        acg.embedding,
        acg.confidence,
        acg.metadata,
        acg.last_updated
    FROM agent_capabilities_graph acg
    WHERE acg.capability_type = 'EXTERNAL_KNOWLEDGE'
        AND acg.metadata->>'source' = p_source
        AND acg.metadata->>'entity_id' = p_entity_id;
END;
$$ LANGUAGE plpgsql STABLE;

-- Function to update knowledge confidence
CREATE OR REPLACE FUNCTION update_knowledge_confidence(
    p_id UUID,
    p_confidence_delta DECIMAL(3,2)
) RETURNS void AS $$
BEGIN
    UPDATE agent_capabilities_graph
    SET
        confidence = GREATEST(0.0, LEAST(1.0, confidence + p_confidence_delta)),
        last_updated = NOW()
    WHERE id = p_id;
END;
$$ LANGUAGE plpgsql;

-- Function to mark knowledge as replaced (immutable updates)
CREATE OR REPLACE FUNCTION replace_knowledge_entry(
    p_source VARCHAR(50),
    p_entity_id VARCHAR(255),
    p_new_id UUID,
    p_decay_rate DECIMAL(5,4) DEFAULT 0.95
) RETURNS void AS $$
BEGIN
    UPDATE agent_capabilities_graph
    SET
        metadata = metadata || jsonb_build_object(
            'replaced_by', p_new_id,
            'decay_rate', p_decay_rate,
            'deactivated_at', NOW()
        ),
        validation_status = 'rejected',
        last_updated = NOW()
    WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata->>'source' = p_source
        AND metadata->>'entity_id' = p_entity_id
        AND validation_status = 'validated';
END;
$$ LANGUAGE plpgsql;

-- Function to apply decay to old knowledge entries
CREATE OR REPLACE FUNCTION apply_knowledge_decay() RETURNS void AS $$
BEGIN
    UPDATE agent_capabilities_graph
    SET
        confidence = GREATEST(0.0, confidence * COALESCE((metadata->>'decay_rate')::DECIMAL, 0.95)),
        last_updated = NOW()
    WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
        AND metadata ? 'decay_rate'
        AND metadata ? 'deactivated_at'
        AND validation_status = 'rejected';
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- ADD STATISTICS FUNCTIONS
-- ============================================================================

-- Function to get knowledge base statistics
CREATE OR REPLACE FUNCTION get_knowledge_stats() RETURNS TABLE(
    source VARCHAR(50),
    total_entries BIGINT,
    active_entries BIGINT,
    avg_confidence DECIMAL,
    last_updated TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT
        metadata->>'source' as source,
        COUNT(*) as total_entries,
        COUNT(*) FILTER (WHERE validation_status = 'validated') as active_entries,
        AVG(confidence) as avg_confidence,
        MAX(last_updated) as last_updated
    FROM agent_capabilities_graph
    WHERE capability_type = 'EXTERNAL_KNOWLEDGE'
    GROUP BY metadata->>'source';
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON TYPE entity_type IS 'Extended entity types including external knowledge sources';

COMMENT ON FUNCTION is_knowledge_indexed IS 'Check if a specific knowledge entity is indexed';

COMMENT ON FUNCTION get_knowledge_entry IS 'Retrieve knowledge entry by source and entity ID';

COMMENT ON FUNCTION update_knowledge_confidence IS 'Update confidence score for knowledge entries';

COMMENT ON FUNCTION replace_knowledge_entry IS 'Mark knowledge entry as replaced with immutable updates';

COMMENT ON FUNCTION apply_knowledge_decay IS 'Apply decay to replaced knowledge entries';

COMMENT ON FUNCTION get_knowledge_stats IS 'Get statistics for external knowledge sources';

-- ============================================================================
-- USAGE EXAMPLES
-- ============================================================================

-- Example 1: Check if Wikidata lexeme is indexed
-- SELECT is_knowledge_indexed('wikidata', 'L12345');

-- Example 2: Get WordNet synset entry
-- SELECT * FROM get_knowledge_entry('wordnet', 'synset-123');

-- Example 3: Update knowledge confidence after successful use
-- SELECT update_knowledge_confidence('uuid-here', 0.05);

-- Example 4: Replace knowledge entry
-- SELECT replace_knowledge_entry('wikidata', 'L12345', 'new-uuid-here', 0.95);

-- Example 5: Apply decay to old entries
-- SELECT apply_knowledge_decay();

-- Example 6: Get knowledge statistics
-- SELECT * FROM get_knowledge_stats();

COMMIT;

-- ============================================================================
-- POST-MIGRATION SETUP
-- ============================================================================

-- Schedule decay application (use pg_cron or application scheduler)
-- SELECT cron.schedule('apply-knowledge-decay', '0 */6 * * *', 'SELECT apply_knowledge_decay();');

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================