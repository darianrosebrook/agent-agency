-- Migration 019: Create Rules & Governance Tables
-- Creates database tables for CAWS rules, violations, and specifications management
-- @author @darianrosebrook

-- ============================================================================
-- CAWS RULES TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS caws_rules (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    rule_type VARCHAR(100) NOT NULL CHECK (rule_type IN ('budget', 'security', 'quality', 'compliance', 'performance', 'custom')),
    severity VARCHAR(50) NOT NULL CHECK (severity IN ('info', 'warning', 'error', 'critical')),
    file_patterns JSONB NOT NULL DEFAULT '[]'::jsonb,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    constitutional_reference VARCHAR(500),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for caws_rules
CREATE INDEX IF NOT EXISTS idx_caws_rules_name ON caws_rules(name);
CREATE INDEX IF NOT EXISTS idx_caws_rules_rule_type ON caws_rules(rule_type);
CREATE INDEX IF NOT EXISTS idx_caws_rules_severity ON caws_rules(severity);
CREATE INDEX IF NOT EXISTS idx_caws_rules_is_active ON caws_rules(is_active);
CREATE INDEX IF NOT EXISTS idx_caws_rules_created_at ON caws_rules(created_at);
CREATE INDEX IF NOT EXISTS idx_caws_rules_updated_at ON caws_rules(updated_at DESC);

-- ============================================================================
-- CAWS VIOLATIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS caws_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    violation_code VARCHAR(100) NOT NULL,
    severity VARCHAR(50) NOT NULL CHECK (severity IN ('info', 'warning', 'error', 'critical')),
    description TEXT NOT NULL,
    file_path VARCHAR(1000),
    line_number INTEGER CHECK (line_number > 0),
    column_number INTEGER CHECK (column_number > 0),
    rule_id VARCHAR(255) NOT NULL REFERENCES caws_rules(id) ON DELETE CASCADE,
    constitutional_reference VARCHAR(500),
    status VARCHAR(50) NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'acknowledged', 'resolved', 'ignored')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for caws_violations
CREATE INDEX IF NOT EXISTS idx_caws_violations_task_id ON caws_violations(task_id);
CREATE INDEX IF NOT EXISTS idx_caws_violations_violation_code ON caws_violations(violation_code);
CREATE INDEX IF NOT EXISTS idx_caws_violations_severity ON caws_violations(severity);
CREATE INDEX IF NOT EXISTS idx_caws_violations_rule_id ON caws_violations(rule_id);
CREATE INDEX IF NOT EXISTS idx_caws_violations_status ON caws_violations(status);
CREATE INDEX IF NOT EXISTS idx_caws_violations_created_at ON caws_violations(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_caws_violations_resolved_at ON caws_violations(resolved_at);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_caws_violations_task_status ON caws_violations(task_id, status);
CREATE INDEX IF NOT EXISTS idx_caws_violations_rule_status ON caws_violations(rule_id, status);

-- ============================================================================
-- CAWS SPECIFICATIONS TABLE
-- ============================================================================

CREATE TABLE IF NOT EXISTS caws_specifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    version VARCHAR(50) NOT NULL,
    description TEXT,
    rules JSONB NOT NULL DEFAULT '[]'::jsonb,
    config JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for caws_specifications
CREATE INDEX IF NOT EXISTS idx_caws_specifications_name ON caws_specifications(name);
CREATE INDEX IF NOT EXISTS idx_caws_specifications_version ON caws_specifications(version);
CREATE INDEX IF NOT EXISTS idx_caws_specifications_is_active ON caws_specifications(is_active);
CREATE INDEX IF NOT EXISTS idx_caws_specifications_created_at ON caws_specifications(created_at);
CREATE INDEX IF NOT EXISTS idx_caws_specifications_updated_at ON caws_specifications(updated_at DESC);

-- Unique constraint on name + version
CREATE UNIQUE INDEX IF NOT EXISTS idx_caws_specifications_name_version ON caws_specifications(name, version);

-- ============================================================================
-- RULE TEMPLATES TABLE (for reusable rule configurations)
-- ============================================================================

CREATE TABLE IF NOT EXISTS rule_templates (
    id VARCHAR(255) PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,
    rule_type VARCHAR(100) NOT NULL,
    template_config JSONB NOT NULL DEFAULT '{}'::jsonb,
    example_config JSONB,
    is_system BOOLEAN NOT NULL DEFAULT FALSE,
    created_by VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for rule_templates
CREATE INDEX IF NOT EXISTS idx_rule_templates_name ON rule_templates(name);
CREATE INDEX IF NOT EXISTS idx_rule_templates_rule_type ON rule_templates(rule_type);
CREATE INDEX IF NOT EXISTS idx_rule_templates_is_system ON rule_templates(is_system);
CREATE INDEX IF NOT EXISTS idx_rule_templates_created_at ON rule_templates(created_at);

-- ============================================================================
-- RULE ENFORCEMENT STATUS TABLE (tracks enforcement state)
-- ============================================================================

CREATE TABLE IF NOT EXISTS rule_enforcement_status (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id VARCHAR(255) NOT NULL REFERENCES caws_rules(id) ON DELETE CASCADE,
    task_id UUID REFERENCES tasks(id) ON DELETE CASCADE,
    enforcement_state VARCHAR(50) NOT NULL DEFAULT 'active' CHECK (enforcement_state IN ('active', 'paused', 'disabled', 'overridden')),
    paused_until TIMESTAMP WITH TIME ZONE,
    paused_reason TEXT,
    override_reason TEXT,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for rule_enforcement_status
CREATE INDEX IF NOT EXISTS idx_rule_enforcement_status_rule_id ON rule_enforcement_status(rule_id);
CREATE INDEX IF NOT EXISTS idx_rule_enforcement_status_task_id ON rule_enforcement_status(task_id);
CREATE INDEX IF NOT EXISTS idx_rule_enforcement_status_state ON rule_enforcement_status(enforcement_state);
CREATE INDEX IF NOT EXISTS idx_rule_enforcement_status_paused_until ON rule_enforcement_status(paused_until);

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_rule_enforcement_status_rule_state ON rule_enforcement_status(rule_id, enforcement_state);

-- ============================================================================
-- RULE HISTORY TABLE (audit trail for rule changes)
-- ============================================================================

CREATE TABLE IF NOT EXISTS rule_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_id VARCHAR(255) NOT NULL REFERENCES caws_rules(id) ON DELETE CASCADE,
    action VARCHAR(50) NOT NULL CHECK (action IN ('created', 'updated', 'activated', 'deactivated', 'deleted')),
    changed_by VARCHAR(255) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    change_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for rule_history
CREATE INDEX IF NOT EXISTS idx_rule_history_rule_id ON rule_history(rule_id);
CREATE INDEX IF NOT EXISTS idx_rule_history_action ON rule_history(action);
CREATE INDEX IF NOT EXISTS idx_rule_history_changed_by ON rule_history(changed_by);
CREATE INDEX IF NOT EXISTS idx_rule_history_created_at ON rule_history(created_at DESC);

-- ============================================================================
-- COMMENTS
-- ============================================================================

COMMENT ON TABLE caws_rules IS 'CAWS rules that define quality gates and compliance requirements';
COMMENT ON TABLE caws_violations IS 'Recorded violations of CAWS rules for tasks';
COMMENT ON TABLE caws_specifications IS 'CAWS specification versions containing rule sets';
COMMENT ON TABLE rule_templates IS 'Reusable rule templates for common patterns';
COMMENT ON TABLE rule_enforcement_status IS 'Tracks enforcement state of rules (active/paused/disabled)';
COMMENT ON TABLE rule_history IS 'Audit trail of rule changes for compliance';

COMMENT ON COLUMN caws_rules.file_patterns IS 'JSON array of file patterns this rule applies to (e.g., ["**/*.rs", "**/*.ts"])';
COMMENT ON COLUMN caws_rules.config IS 'Rule-specific configuration JSON';
COMMENT ON COLUMN caws_violations.metadata IS 'Additional violation context and metadata';
COMMENT ON COLUMN rule_enforcement_status.paused_until IS 'Timestamp when paused enforcement will resume (NULL if indefinite)';

