-- Migration 004: Add Provenance Tables
-- Adds tables for provenance record storage and management

-- Provenance records table
CREATE TABLE provenance_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    verdict_id UUID NOT NULL,
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    decision_type VARCHAR(50) NOT NULL CHECK (
        decision_type IN ('accept', 'reject', 'require_modification', 'need_investigation')
    ),
    decision_data JSONB NOT NULL,
    consensus_score DECIMAL(3, 2) NOT NULL,
    judge_verdicts JSONB NOT NULL DEFAULT '{}',
    caws_compliance JSONB NOT NULL,
    claim_verification JSONB,
    git_commit_hash VARCHAR(40),
    git_trailer TEXT NOT NULL,
    signature TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for provenance records
CREATE INDEX idx_provenance_records_verdict_id ON provenance_records(verdict_id);
CREATE INDEX idx_provenance_records_task_id ON provenance_records(task_id);
CREATE INDEX idx_provenance_records_decision_type ON provenance_records(decision_type);
CREATE INDEX idx_provenance_records_timestamp ON provenance_records(timestamp);
CREATE INDEX idx_provenance_records_git_commit_hash ON provenance_records(git_commit_hash);

-- Trigger for automatic timestamp updates
CREATE TRIGGER update_provenance_records_updated_at 
    BEFORE UPDATE ON provenance_records 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Function to get provenance statistics
CREATE OR REPLACE FUNCTION get_provenance_statistics(
    p_time_range_start TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_time_range_end TIMESTAMP WITH TIME ZONE DEFAULT NULL
)
RETURNS JSONB AS $$
DECLARE
    result JSONB;
    time_filter TEXT := '';
BEGIN
    -- Build time filter if provided
    IF p_time_range_start IS NOT NULL AND p_time_range_end IS NOT NULL THEN
        time_filter := 'WHERE timestamp BETWEEN $1 AND $2';
    END IF;

    -- Execute dynamic query to get statistics
    EXECUTE format('
        SELECT jsonb_build_object(
            ''total_records'', COUNT(*),
            ''total_verdicts'', COUNT(DISTINCT verdict_id),
            ''acceptance_rate'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN 
                        COUNT(CASE WHEN decision_type = ''accept'' THEN 1 END)::DECIMAL / COUNT(*)
                    ELSE 0 
                END,
            ''average_consensus_score'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN AVG(consensus_score)
                    ELSE 0 
                END,
            ''average_compliance_score'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN AVG((caws_compliance->>''compliance_score'')::DECIMAL)
                    ELSE 0 
                END,
            ''average_verification_quality'', 
                CASE 
                    WHEN COUNT(*) > 0 THEN AVG((claim_verification->>''verification_quality'')::DECIMAL)
                    ELSE 0 
                END,
            ''most_active_judge'', 
                COALESCE(
                    (SELECT judge_id 
                     FROM (
                         SELECT jsonb_object_keys(judge_verdicts) as judge_id
                         FROM provenance_records %s
                     ) judge_counts
                     GROUP BY judge_id 
                     ORDER BY COUNT(*) DESC 
                     LIMIT 1), 
                    ''Unknown''
                ),
            ''common_violations'', 
                COALESCE(
                    (SELECT jsonb_agg(
                        jsonb_build_object(
                            ''rule'', rule,
                            ''count'', count,
                            ''severity_distribution'', severity_distribution,
                            ''average_resolution_time_ms'', average_resolution_time_ms
                        )
                    )
                    FROM (
                        SELECT 
                            violation->>''rule'' as rule,
                            COUNT(*) as count,
                            jsonb_build_object() as severity_distribution,
                            0.0 as average_resolution_time_ms
                        FROM provenance_records %s,
                             jsonb_array_elements(caws_compliance->''violations'') as violation
                        GROUP BY violation->>''rule''
                        ORDER BY count DESC
                        LIMIT 10
                    ) violation_stats), 
                    ''[]''::jsonb
                ),
            ''time_range'', jsonb_build_object(
                ''start'', COALESCE(MIN(timestamp), NOW()),
                ''end'', COALESCE(MAX(timestamp), NOW())
            )
        )
        FROM provenance_records %s
    ', time_filter, time_filter, time_filter)
    USING p_time_range_start, p_time_range_end, p_time_range_start, p_time_range_end
    INTO result;

    RETURN COALESCE(result, '{}'::jsonb);
END;
$$ LANGUAGE plpgsql;

-- Function to query provenance records with filters
CREATE OR REPLACE FUNCTION query_provenance_records(
    p_task_id UUID DEFAULT NULL,
    p_verdict_id UUID DEFAULT NULL,
    p_decision_type VARCHAR(50) DEFAULT NULL,
    p_time_range_start TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_time_range_end TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_judge_id VARCHAR(255) DEFAULT NULL,
    p_compliance_status VARCHAR(50) DEFAULT NULL,
    p_limit INTEGER DEFAULT 1000,
    p_offset INTEGER DEFAULT 0
)
RETURNS TABLE (
    id UUID,
    verdict_id UUID,
    task_id UUID,
    decision_type VARCHAR(50),
    decision_data JSONB,
    consensus_score DECIMAL(3, 2),
    judge_verdicts JSONB,
    caws_compliance JSONB,
    claim_verification JSONB,
    git_commit_hash VARCHAR(40),
    git_trailer TEXT,
    signature TEXT,
    timestamp TIMESTAMP WITH TIME ZONE,
    metadata JSONB,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        pr.id,
        pr.verdict_id,
        pr.task_id,
        pr.decision_type,
        pr.decision_data,
        pr.consensus_score,
        pr.judge_verdicts,
        pr.caws_compliance,
        pr.claim_verification,
        pr.git_commit_hash,
        pr.git_trailer,
        pr.signature,
        pr.timestamp,
        pr.metadata,
        pr.created_at,
        pr.updated_at
    FROM provenance_records pr
    WHERE 
        (p_task_id IS NULL OR pr.task_id = p_task_id)
        AND (p_verdict_id IS NULL OR pr.verdict_id = p_verdict_id)
        AND (p_decision_type IS NULL OR pr.decision_type = p_decision_type)
        AND (p_time_range_start IS NULL OR pr.timestamp >= p_time_range_start)
        AND (p_time_range_end IS NULL OR pr.timestamp <= p_time_range_end)
        AND (p_judge_id IS NULL OR pr.judge_verdicts ? p_judge_id)
        AND (p_compliance_status IS NULL OR 
             (p_compliance_status = 'compliant' AND (pr.caws_compliance->>'is_compliant')::BOOLEAN = true) OR
             (p_compliance_status = 'non_compliant' AND (pr.caws_compliance->>'is_compliant')::BOOLEAN = false) OR
             (p_compliance_status = 'partial_compliance' AND (pr.caws_compliance->>'compliance_score')::DECIMAL < 1.0 AND (pr.caws_compliance->>'is_compliant')::BOOLEAN = true))
    ORDER BY pr.timestamp DESC
    LIMIT p_limit
    OFFSET p_offset;
END;
$$ LANGUAGE plpgsql;

-- Comments for documentation
COMMENT ON TABLE provenance_records IS 'Complete provenance records for CAWS verdicts with full audit trail';
COMMENT ON COLUMN provenance_records.decision_data IS 'Full decision data including confidence, summary, and reasoning';
COMMENT ON COLUMN provenance_records.judge_verdicts IS 'Individual judge verdicts contributing to the final decision';
COMMENT ON COLUMN provenance_records.caws_compliance IS 'CAWS compliance data including violations and waivers';
COMMENT ON COLUMN provenance_records.claim_verification IS 'Claim verification data and evidence quality scores';
COMMENT ON COLUMN provenance_records.git_trailer IS 'Git trailer for commit attribution and tracking';
COMMENT ON COLUMN provenance_records.signature IS 'Cryptographic signature for integrity verification';
