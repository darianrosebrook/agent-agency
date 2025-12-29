-- Migration 018: Create Waivers Table
-- Creates the waivers table required by UnifiedOrchestrator planning system
-- This table must be created before UnifiedOrchestrator can initialize
-- @author @darianrosebrook

-- ============================================================================
-- WAIVERS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS waivers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    reason VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    gates JSONB NOT NULL DEFAULT '[]'::jsonb,
    approved_by VARCHAR(255) NOT NULL,
    impact_level VARCHAR(50) NOT NULL CHECK (impact_level IN ('low', 'medium', 'high', 'critical')),
    mitigation_plan TEXT,
    expires_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'expired', 'revoked', 'pending')),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for waivers
CREATE INDEX IF NOT EXISTS idx_waivers_status ON waivers(status);
CREATE INDEX IF NOT EXISTS idx_waivers_approved_by ON waivers(approved_by);
CREATE INDEX IF NOT EXISTS idx_waivers_impact_level ON waivers(impact_level);
CREATE INDEX IF NOT EXISTS idx_waivers_expires_at ON waivers(expires_at);
CREATE INDEX IF NOT EXISTS idx_waivers_created_at ON waivers(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_waivers_updated_at ON waivers(updated_at DESC);

-- GIN index for JSONB queries
CREATE INDEX IF NOT EXISTS idx_waivers_gates ON waivers USING GIN(gates);
CREATE INDEX IF NOT EXISTS idx_waivers_metadata ON waivers USING GIN(metadata);

-- Composite index for active waivers query
CREATE INDEX IF NOT EXISTS idx_waivers_status_expires ON waivers(status, expires_at)
    WHERE status = 'active';

-- Comments
COMMENT ON TABLE waivers IS 'Quality gate waivers for exceptional circumstances';
COMMENT ON COLUMN waivers.gates IS 'Array of quality gate names that are waived';
COMMENT ON COLUMN waivers.impact_level IS 'Risk impact level: low, medium, high, or critical';
COMMENT ON COLUMN waivers.mitigation_plan IS 'Plan to mitigate risks introduced by this waiver';
COMMENT ON COLUMN waivers.expires_at IS 'When this waiver expires (NULL for indefinite)';
COMMENT ON COLUMN waivers.status IS 'Waiver status: active, expired, revoked, or pending';

