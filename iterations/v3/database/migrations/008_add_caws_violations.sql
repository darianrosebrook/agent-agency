-- Add CAWS violations table for compliance tracking
-- This table stores CAWS compliance violations detected by workers

CREATE TABLE IF NOT EXISTS caws_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    violation_code TEXT NOT NULL,
    severity TEXT NOT NULL CHECK (severity IN ('low', 'medium', 'high', 'critical')),
    description TEXT NOT NULL,
    file_path TEXT,
    line_number INTEGER,
    column_number INTEGER,
    rule_id TEXT NOT NULL,
    constitutional_reference TEXT,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'resolved', 'waived', 'dismissed')),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    metadata JSONB DEFAULT '{}'::jsonb,

    -- Indexes for efficient querying
    INDEX idx_caws_violations_task_id (task_id),
    INDEX idx_caws_violations_rule_id (rule_id),
    INDEX idx_caws_violations_status (status),
    INDEX idx_caws_violations_created_at (created_at),
    INDEX idx_caws_violations_severity (severity),

    -- Foreign key constraint (assuming tasks table exists)
    CONSTRAINT fk_caws_violations_task_id
        FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE,

    -- Ensure resolved_at is set when status is resolved
    CONSTRAINT chk_resolved_at_when_resolved
        CHECK ((status = 'resolved' AND resolved_at IS NOT NULL) OR status != 'resolved')
);

-- Add comments for documentation
COMMENT ON TABLE caws_violations IS 'Stores CAWS compliance violations detected by workers for audit and tracking purposes';
COMMENT ON COLUMN caws_violations.violation_code IS 'The specific CAWS rule that was violated';
COMMENT ON COLUMN caws_violations.constitutional_reference IS 'Reference to constitutional document supporting the rule';
COMMENT ON COLUMN caws_violations.metadata IS 'Additional structured metadata about the violation';
