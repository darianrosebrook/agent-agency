-- Migration: Add Multi-Tenant Isolation with Row Level Security
-- Version: 007
-- Description: Implements database-level tenant isolation with RLS policies and privacy configurations
-- Author: @darianrosebrook
-- Date: 2025-10-12
--
-- This migration adds:
-- - Tenant management infrastructure
-- - Tenant columns to existing tables
-- - Row Level Security (RLS) policies
-- - Privacy configuration for federated learning
-- - Access control and audit logging

BEGIN;

-- ============================================================================
-- CUSTOM TYPES FOR MULTI-TENANCY
-- ============================================================================

-- Tenant isolation levels
CREATE TYPE isolation_level AS ENUM (
    'strict',      -- No data sharing between tenants
    'shared',      -- Explicit sharing rules apply
    'federated'    -- Cross-tenant learning allowed with privacy preservation
);

-- Privacy levels for federated learning
CREATE TYPE privacy_level AS ENUM (
    'basic',        -- Basic anonymization
    'differential', -- Differential privacy with noise
    'secure'        -- Secure multi-party computation
);

-- Data retention policies
CREATE TYPE retention_policy AS ENUM (
    'delete',   -- Delete after retention period
    'archive',  -- Move to cold storage
    'retain'    -- Keep indefinitely
);

-- ============================================================================
-- TENANT MANAGEMENT TABLES
-- ============================================================================

CREATE TABLE tenants (
    id VARCHAR(255) PRIMARY KEY,
    project_id VARCHAR(255) NOT NULL,
    name VARCHAR(255) NOT NULL,

-- Isolation configuration
isolation_level isolation_level NOT NULL DEFAULT 'strict',

-- Access control
access_policies JSONB DEFAULT '[]'::jsonb,
    sharing_rules JSONB DEFAULT '[]'::jsonb,

-- Data retention
data_retention JSONB NOT NULL DEFAULT '{
        "policy": "archive",
        "retention_days": 730,
        "archive_after_days": 90
    }'::jsonb,

-- Security settings
encryption_enabled BOOLEAN DEFAULT false,
audit_logging BOOLEAN DEFAULT true,

-- Configuration
config JSONB DEFAULT '{}'::jsonb,

-- Temporal
created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

-- Constraints
CONSTRAINT unique_tenant_name UNIQUE (project_id, name) );

-- Indexes for tenants
CREATE INDEX idx_tenants_project ON tenants (project_id);

CREATE INDEX idx_tenants_isolation ON tenants (isolation_level);

-- ============================================================================
-- PRIVACY CONFIGURATION FOR FEDERATED LEARNING
-- ============================================================================

CREATE TABLE tenant_privacy_config (
    tenant_id VARCHAR(255) PRIMARY KEY REFERENCES tenants(id) ON DELETE CASCADE,

-- Privacy level
privacy_level privacy_level NOT NULL DEFAULT 'differential',

-- Differential privacy parameters
noise_magnitude DECIMAL(5, 4) DEFAULT 0.01 CHECK (noise_magnitude >= 0.0),
k_anonymity INTEGER DEFAULT 5 CHECK (k_anonymity >= 2),
epsilon DECIMAL(5, 4) DEFAULT 1.0 CHECK (epsilon > 0.0),
delta DECIMAL(10, 8) DEFAULT 0.00001 CHECK (delta >= 0.0),

-- Data sharing preferences
allow_cross_tenant_learning BOOLEAN DEFAULT false,
    allowed_tenant_groups TEXT[] DEFAULT '{}',

-- Temporal
updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW() );

-- ============================================================================
-- ADD TENANT_ID TO EXISTING TABLES
-- ============================================================================

-- Add tenant_id to agent_profiles (if not exists)
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'agent_profiles' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE agent_profiles ADD COLUMN tenant_id VARCHAR(255);
    END IF;
END $$;

-- Add tenant_id to performance_events
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'performance_events' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE performance_events ADD COLUMN tenant_id VARCHAR(255);
    END IF;
END $$;

-- Add tenant_id to benchmark_datasets
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'benchmark_datasets' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE benchmark_datasets ADD COLUMN tenant_id VARCHAR(255);
    END IF;
END $$;

-- Add tenant_id to agent_capabilities_graph
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'agent_capabilities_graph' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE agent_capabilities_graph ADD COLUMN tenant_id VARCHAR(255);
    END IF;
END $$;

-- Add tenant_id to performance_anomalies
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'performance_anomalies' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE performance_anomalies ADD COLUMN tenant_id VARCHAR(255);
    END IF;
END $$;

-- Add tenant_id to rl_training_batches
DO $$ 
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM information_schema.columns 
        WHERE table_name = 'rl_training_batches' AND column_name = 'tenant_id'
    ) THEN
        ALTER TABLE rl_training_batches ADD COLUMN tenant_id VARCHAR(255);
    END IF;
END $$;

-- ============================================================================
-- CREATE DEFAULT TENANT FOR EXISTING DATA
-- ============================================================================

-- Insert default tenant
INSERT INTO
    tenants (
        id,
        project_id,
        name,
        isolation_level,
        audit_logging
    )
VALUES (
        'default-tenant',
        'default-project',
        'Default Tenant',
        'strict',
        true
    ) ON CONFLICT (id) DO NOTHING;

-- Backfill tenant_id for existing records
UPDATE agent_profiles
SET
    tenant_id = 'default-tenant'
WHERE
    tenant_id IS NULL;

UPDATE performance_events
SET
    tenant_id = 'default-tenant'
WHERE
    tenant_id IS NULL;

UPDATE benchmark_datasets
SET
    tenant_id = 'default-tenant'
WHERE
    tenant_id IS NULL;

UPDATE agent_capabilities_graph
SET
    tenant_id = 'default-tenant'
WHERE
    tenant_id IS NULL;

UPDATE performance_anomalies
SET
    tenant_id = 'default-tenant'
WHERE
    tenant_id IS NULL;

UPDATE rl_training_batches
SET
    tenant_id = 'default-tenant'
WHERE
    tenant_id IS NULL;

-- Make tenant_id NOT NULL after backfill
ALTER TABLE agent_profiles ALTER COLUMN tenant_id SET NOT NULL;

ALTER TABLE performance_events ALTER COLUMN tenant_id SET NOT NULL;

ALTER TABLE benchmark_datasets ALTER COLUMN tenant_id SET NOT NULL;

ALTER TABLE agent_capabilities_graph
ALTER COLUMN tenant_id
SET
    NOT NULL;

ALTER TABLE performance_anomalies
ALTER COLUMN tenant_id
SET
    NOT NULL;

ALTER TABLE rl_training_batches ALTER COLUMN tenant_id SET NOT NULL;

-- Add foreign key constraints
ALTER TABLE agent_profiles
ADD CONSTRAINT fk_agent_profiles_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id) ON DELETE RESTRICT;

ALTER TABLE performance_events
ADD CONSTRAINT fk_performance_events_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id) ON DELETE RESTRICT;

ALTER TABLE benchmark_datasets
ADD CONSTRAINT fk_benchmark_datasets_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id) ON DELETE RESTRICT;

ALTER TABLE agent_capabilities_graph
ADD CONSTRAINT fk_capabilities_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id) ON DELETE RESTRICT;

ALTER TABLE performance_anomalies
ADD CONSTRAINT fk_anomalies_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id) ON DELETE RESTRICT;

ALTER TABLE rl_training_batches
ADD CONSTRAINT fk_rl_batches_tenant FOREIGN KEY (tenant_id) REFERENCES tenants (id) ON DELETE RESTRICT;

-- Create indexes on tenant_id for performance
CREATE INDEX idx_agent_profiles_tenant ON agent_profiles (tenant_id);

CREATE INDEX idx_performance_events_tenant ON performance_events (tenant_id);

CREATE INDEX idx_benchmark_datasets_tenant ON benchmark_datasets (tenant_id);

CREATE INDEX idx_capabilities_tenant ON agent_capabilities_graph (tenant_id);

CREATE INDEX idx_anomalies_tenant ON performance_anomalies (tenant_id);

CREATE INDEX idx_rl_batches_tenant ON rl_training_batches (tenant_id);

-- ============================================================================
-- ROW LEVEL SECURITY POLICIES
-- ============================================================================

-- Enable RLS on tenant-scoped tables
ALTER TABLE agent_profiles ENABLE ROW LEVEL SECURITY;

ALTER TABLE performance_events ENABLE ROW LEVEL SECURITY;

ALTER TABLE benchmark_datasets ENABLE ROW LEVEL SECURITY;

ALTER TABLE agent_capabilities_graph ENABLE ROW LEVEL SECURITY;

ALTER TABLE performance_anomalies ENABLE ROW LEVEL SECURITY;

ALTER TABLE rl_training_batches ENABLE ROW LEVEL SECURITY;

-- Policy for strict isolation mode
CREATE POLICY tenant_strict_isolation ON agent_profiles
    USING (
        tenant_id = current_setting('app.current_tenant', true)::VARCHAR
        AND EXISTS (
            SELECT 1 FROM tenants 
            WHERE id = agent_profiles.tenant_id 
            AND isolation_level = 'strict'
        )
    );

-- Policy for shared isolation mode
CREATE POLICY tenant_shared_access ON agent_profiles
    USING (
        tenant_id = current_setting('app.current_tenant', true)::VARCHAR
        OR (
            EXISTS (
                SELECT 1 FROM tenants 
                WHERE id = agent_profiles.tenant_id 
                AND isolation_level = 'shared'
            )
            AND tenant_id IN (
                SELECT unnest(
                    COALESCE(
                        (sharing_rules->>'allowed_tenants')::TEXT[],
                        '{}'::TEXT[]
                    )
                )
                FROM tenants 
                WHERE id = current_setting('app.current_tenant', true)::VARCHAR
            )
        )
    );

-- Policy for federated isolation mode (performance data can be aggregated)
CREATE POLICY tenant_federated_access ON performance_events
    USING (
        tenant_id = current_setting('app.current_tenant', true)::VARCHAR
        OR EXISTS (
            SELECT 1 FROM tenants t1
            WHERE t1.id = current_setting('app.current_tenant', true)::VARCHAR
            AND t1.isolation_level = 'federated'
            AND EXISTS (
                SELECT 1 FROM tenants t2
                WHERE t2.id = performance_events.tenant_id
                AND t2.isolation_level = 'federated'
            )
        )
    );

-- Apply similar policies to other tables
CREATE POLICY tenant_benchmark_isolation ON benchmark_datasets
    USING (tenant_id = current_setting('app.current_tenant', true)::VARCHAR);

CREATE POLICY tenant_capability_isolation ON agent_capabilities_graph
    USING (tenant_id = current_setting('app.current_tenant', true)::VARCHAR);

CREATE POLICY tenant_anomaly_isolation ON performance_anomalies
    USING (tenant_id = current_setting('app.current_tenant', true)::VARCHAR);

CREATE POLICY tenant_rl_batch_isolation ON rl_training_batches
    USING (tenant_id = current_setting('app.current_tenant', true)::VARCHAR);

-- ============================================================================
-- PRIVACY-PRESERVING FUNCTIONS
-- ============================================================================

-- Function to apply differential privacy noise
CREATE OR REPLACE FUNCTION add_dp_noise(
    value DECIMAL,
    p_tenant_id VARCHAR(255)
) RETURNS DECIMAL AS $$
DECLARE
    config RECORD;
    noise DECIMAL;
BEGIN
    -- Get privacy config
    SELECT * INTO config 
    FROM tenant_privacy_config 
    WHERE tenant_id = p_tenant_id;
    
    -- If no config or basic privacy, return as-is
    IF NOT FOUND OR config.privacy_level = 'basic' THEN
        RETURN value;
    END IF;
    
    -- Add Laplace noise for differential privacy
    -- noise = Lap(0, sensitivity/epsilon)
    noise := config.noise_magnitude * (random() - 0.5) * 2 / NULLIF(config.epsilon, 0);
    
    RETURN value + noise;
END;
$$ LANGUAGE plpgsql VOLATILE;

-- Function to check k-anonymity compliance
CREATE OR REPLACE FUNCTION check_k_anonymity(
    group_size INTEGER,
    p_tenant_id VARCHAR(255)
) RETURNS BOOLEAN AS $$
DECLARE
    min_k INTEGER;
BEGIN
    SELECT k_anonymity INTO min_k
    FROM tenant_privacy_config
    WHERE tenant_id = p_tenant_id;
    
    -- If no config, use default k=5
    IF NOT FOUND THEN
        min_k := 5;
    END IF;
    
    RETURN group_size >= min_k;
END;
$$ LANGUAGE plpgsql STABLE;

-- ============================================================================
-- AUDIT LOGGING FOR TENANT ACCESS
-- ============================================================================

CREATE TABLE tenant_access_log (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    tenant_id VARCHAR(255) NOT NULL REFERENCES tenants(id) ON DELETE CASCADE,

-- Access details
accessor_tenant_id VARCHAR(255),
access_type VARCHAR(50) NOT NULL CHECK (
    access_type IN (
        'read',
        'write',
        'aggregate',
        'federated'
    )
),
table_name VARCHAR(100) NOT NULL,

-- Context
query_hash VARCHAR(64), row_count INTEGER,

-- User context
user_id VARCHAR(255), ip_address INET,

-- Temporal
accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW() );

CREATE INDEX idx_tenant_access_log_tenant ON tenant_access_log (tenant_id);

CREATE INDEX idx_tenant_access_log_accessed ON tenant_access_log (accessed_at DESC);

-- Trigger to log tenant access
CREATE OR REPLACE FUNCTION log_tenant_access()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO tenant_access_log (
        tenant_id,
        accessor_tenant_id,
        access_type,
        table_name,
        user_id
    ) VALUES (
        NEW.tenant_id,
        current_setting('app.current_tenant', true),
        TG_OP,
        TG_TABLE_NAME,
        current_setting('app.current_user', true)
    );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Apply audit trigger to sensitive tables (optional, can be enabled per tenant)
-- CREATE TRIGGER audit_agent_access
--     AFTER SELECT ON agent_profiles
--     FOR EACH ROW
--     WHEN (EXISTS (SELECT 1 FROM tenants WHERE id = NEW.tenant_id AND audit_logging = true))
--     EXECUTE FUNCTION log_tenant_access();

-- ============================================================================
-- TENANT MANAGEMENT FUNCTIONS
-- ============================================================================

-- Function to create a new tenant
CREATE OR REPLACE FUNCTION create_tenant(
    p_tenant_id VARCHAR(255),
    p_project_id VARCHAR(255),
    p_name VARCHAR(255),
    p_isolation_level isolation_level DEFAULT 'strict'
) RETURNS VARCHAR(255) AS $$
BEGIN
    INSERT INTO tenants (id, project_id, name, isolation_level)
    VALUES (p_tenant_id, p_project_id, p_name, p_isolation_level);
    
    -- Create default privacy config
    INSERT INTO tenant_privacy_config (tenant_id)
    VALUES (p_tenant_id);
    
    RETURN p_tenant_id;
END;
$$ LANGUAGE plpgsql;

-- Function to update tenant isolation level
CREATE OR REPLACE FUNCTION update_tenant_isolation(
    p_tenant_id VARCHAR(255),
    p_new_level isolation_level
) RETURNS BOOLEAN AS $$
DECLARE
    affected_rows INTEGER;
BEGIN
    UPDATE tenants 
    SET isolation_level = p_new_level,
        updated_at = NOW()
    WHERE id = p_tenant_id;
    
    GET DIAGNOSTICS affected_rows = ROW_COUNT;
    RETURN affected_rows > 0;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- VIEWS FOR TENANT MANAGEMENT
-- ============================================================================

-- View for tenant statistics
CREATE VIEW tenant_statistics AS
SELECT
    t.id as tenant_id,
    t.name,
    t.isolation_level,
    t.audit_logging,
    COUNT(DISTINCT ap.id) as agent_count,
    COUNT(DISTINCT pe.id) as event_count,
    COUNT(DISTINCT bd.id) as benchmark_count,
    t.created_at
FROM
    tenants t
    LEFT JOIN agent_profiles ap ON t.id = ap.tenant_id
    LEFT JOIN performance_events pe ON t.id = pe.tenant_id
    LEFT JOIN benchmark_datasets bd ON t.id = bd.tenant_id
GROUP BY
    t.id,
    t.name,
    t.isolation_level,
    t.audit_logging,
    t.created_at;

-- View for cross-tenant access patterns
CREATE VIEW cross_tenant_access_summary AS
SELECT
    tenant_id,
    accessor_tenant_id,
    access_type,
    COUNT(*) as access_count,
    MIN(accessed_at) as first_access,
    MAX(accessed_at) as last_access
FROM tenant_access_log
WHERE
    accessor_tenant_id != tenant_id
GROUP BY
    tenant_id,
    accessor_tenant_id,
    access_type;

-- ============================================================================
-- COMMENTS FOR DOCUMENTATION
-- ============================================================================

COMMENT ON
TABLE tenants IS 'Tenant management with configurable isolation levels and access policies';

COMMENT ON
TABLE tenant_privacy_config IS 'Privacy configuration for federated learning with differential privacy parameters';

COMMENT ON
TABLE tenant_access_log IS 'Audit log of cross-tenant data access for compliance and security monitoring';

COMMENT ON COLUMN tenants.isolation_level IS 'Isolation mode: strict (no sharing), shared (explicit rules), federated (privacy-preserved learning)';

COMMENT ON COLUMN tenant_privacy_config.epsilon IS 'Privacy budget for differential privacy (lower = more private)';

COMMENT ON COLUMN tenant_privacy_config.k_anonymity IS 'Minimum group size for k-anonymity (minimum 2)';

COMMIT;

-- ============================================================================
-- POST-MIGRATION NOTES
-- ============================================================================

-- To set tenant context in application code:
-- await client.query(`SET LOCAL app.current_tenant = $1`, [tenantId]);

-- To check if RLS is working:
-- SET app.current_tenant = 'tenant-A';
-- SELECT COUNT(*) FROM agent_profiles; -- Should only see tenant A data

-- To create a new tenant:
-- SELECT create_tenant('new-tenant-id', 'project-id', 'Tenant Name', 'strict');

-- ============================================================================
-- MIGRATION COMPLETE
-- ============================================================================
