-- Migration 021: Create Rate Limit Tables
-- Creates database tables for MCP server authentication rate limiting persistence:
-- rate_limit_blocks, rate_limit_suspicious
-- @author @darianrosebrook

-- ============================================================================
-- RATE LIMIT BLOCKS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS rate_limit_blocks (
    ip VARCHAR(255) PRIMARY KEY,
    blocked_until TIMESTAMP WITH TIME ZONE NOT NULL,
    risk_score INTEGER NOT NULL DEFAULT 0 CHECK (risk_score >= 0 AND risk_score <= 100),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for rate_limit_blocks
CREATE INDEX IF NOT EXISTS idx_rate_limit_blocks_blocked_until ON rate_limit_blocks(blocked_until);
CREATE INDEX IF NOT EXISTS idx_rate_limit_blocks_risk_score ON rate_limit_blocks(risk_score);
CREATE INDEX IF NOT EXISTS idx_rate_limit_blocks_created_at ON rate_limit_blocks(created_at);

-- ============================================================================
-- RATE LIMIT SUSPICIOUS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS rate_limit_suspicious (
    ip VARCHAR(255) PRIMARY KEY,
    risk_score INTEGER NOT NULL DEFAULT 0 CHECK (risk_score >= 0 AND risk_score <= 100),
    first_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_seen TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for rate_limit_suspicious
CREATE INDEX IF NOT EXISTS idx_rate_limit_suspicious_risk_score ON rate_limit_suspicious(risk_score);
CREATE INDEX IF NOT EXISTS idx_rate_limit_suspicious_last_seen ON rate_limit_suspicious(last_seen DESC);
CREATE INDEX IF NOT EXISTS idx_rate_limit_suspicious_first_seen ON rate_limit_suspicious(first_seen);

-- ============================================================================
-- TRIGGERS
-- ============================================================================

-- Auto-update updated_at timestamp for rate_limit_blocks
CREATE OR REPLACE FUNCTION update_rate_limit_blocks_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_rate_limit_blocks_updated_at
    BEFORE UPDATE ON rate_limit_blocks
    FOR EACH ROW
    EXECUTE FUNCTION update_rate_limit_blocks_updated_at();

-- Auto-update updated_at and last_seen timestamp for rate_limit_suspicious
CREATE OR REPLACE FUNCTION update_rate_limit_suspicious_timestamps()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    NEW.last_seen = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_rate_limit_suspicious_timestamps
    BEFORE UPDATE ON rate_limit_suspicious
    FOR EACH ROW
    EXECUTE FUNCTION update_rate_limit_suspicious_timestamps();





