-- Migration 005: Add Source Integrity Verification Tables
-- Adds tables for source integrity verification and hash storage

-- Source integrity records table
CREATE TABLE source_integrity_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_id VARCHAR(255) NOT NULL,
    source_type VARCHAR(50) NOT NULL CHECK (
        source_type IN ('file', 'url', 'content', 'code', 'document')
    ),
    content_hash VARCHAR(64) NOT NULL, -- SHA-256 hash as hex string
    content_size BIGINT NOT NULL,
    hash_algorithm VARCHAR(20) NOT NULL DEFAULT 'sha256',
    integrity_status VARCHAR(20) NOT NULL CHECK (
        integrity_status IN ('verified', 'tampered', 'unknown', 'pending')
    ),
    tampering_indicators JSONB NOT NULL DEFAULT '[]',
    verification_metadata JSONB NOT NULL DEFAULT '{}',
    first_seen_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    last_verified_at TIMESTAMP WITH TIME ZONE,
    verification_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Ensure unique source_id per source_type
    UNIQUE(source_id, source_type)
);

-- Source integrity verification history
CREATE TABLE source_integrity_verifications (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_integrity_id UUID NOT NULL REFERENCES source_integrity_records(id) ON DELETE CASCADE,
    verification_type VARCHAR(50) NOT NULL CHECK (
        verification_type IN ('initial', 'periodic', 'on_access', 'manual', 'automated')
    ),
    verification_result VARCHAR(20) NOT NULL CHECK (
        verification_result IN ('passed', 'failed', 'warning', 'error')
    ),
    calculated_hash VARCHAR(64) NOT NULL,
    stored_hash VARCHAR(64) NOT NULL,
    hash_match BOOLEAN NOT NULL,
    tampering_detected BOOLEAN NOT NULL DEFAULT false,
    verification_details JSONB NOT NULL DEFAULT '{}',
    verified_by VARCHAR(255), -- system component or user
    verification_duration_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Source integrity alerts and notifications
CREATE TABLE source_integrity_alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_integrity_id UUID NOT NULL REFERENCES source_integrity_records(id) ON DELETE CASCADE,
    alert_type VARCHAR(50) NOT NULL CHECK (
        alert_type IN ('tampering_detected', 'hash_mismatch', 'verification_failed', 'integrity_unknown')
    ),
    severity VARCHAR(20) NOT NULL CHECK (
        severity IN ('low', 'medium', 'high', 'critical')
    ),
    alert_message TEXT NOT NULL,
    alert_data JSONB NOT NULL DEFAULT '{}',
    acknowledged BOOLEAN NOT NULL DEFAULT false,
    acknowledged_by VARCHAR(255),
    acknowledged_at TIMESTAMP WITH TIME ZONE,
    resolved BOOLEAN NOT NULL DEFAULT false,
    resolved_by VARCHAR(255),
    resolved_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_source_integrity_records_source_id ON source_integrity_records(source_id);
CREATE INDEX idx_source_integrity_records_source_type ON source_integrity_records(source_type);
CREATE INDEX idx_source_integrity_records_content_hash ON source_integrity_records(content_hash);
CREATE INDEX idx_source_integrity_records_integrity_status ON source_integrity_records(integrity_status);
CREATE INDEX idx_source_integrity_records_last_verified ON source_integrity_records(last_verified_at);
CREATE INDEX idx_source_integrity_records_created_at ON source_integrity_records(created_at);

CREATE INDEX idx_source_integrity_verifications_source_id ON source_integrity_verifications(source_integrity_id);
CREATE INDEX idx_source_integrity_verifications_verification_type ON source_integrity_verifications(verification_type);
CREATE INDEX idx_source_integrity_verifications_verification_result ON source_integrity_verifications(verification_result);
CREATE INDEX idx_source_integrity_verifications_created_at ON source_integrity_verifications(created_at);
CREATE INDEX idx_source_integrity_verifications_hash_match ON source_integrity_verifications(hash_match);

CREATE INDEX idx_source_integrity_alerts_source_id ON source_integrity_alerts(source_integrity_id);
CREATE INDEX idx_source_integrity_alerts_alert_type ON source_integrity_alerts(alert_type);
CREATE INDEX idx_source_integrity_alerts_severity ON source_integrity_alerts(severity);
CREATE INDEX idx_source_integrity_alerts_acknowledged ON source_integrity_alerts(acknowledged);
CREATE INDEX idx_source_integrity_alerts_resolved ON source_integrity_alerts(resolved);
CREATE INDEX idx_source_integrity_alerts_created_at ON source_integrity_alerts(created_at);

-- Triggers for automatic timestamp updates
CREATE TRIGGER update_source_integrity_records_updated_at 
    BEFORE UPDATE ON source_integrity_records 
    FOR EACH ROW 
    EXECUTE FUNCTION update_updated_at_column();

-- Function to calculate content hash
CREATE OR REPLACE FUNCTION calculate_content_hash(
    p_content TEXT,
    p_algorithm VARCHAR(20) DEFAULT 'sha256'
)
RETURNS VARCHAR(64) AS $$
BEGIN
    -- For now, return a placeholder hash calculation
    -- In production, this would use proper cryptographic functions
    RETURN encode(digest(p_content, p_algorithm), 'hex');
EXCEPTION
    WHEN OTHERS THEN
        -- Fallback to a simple hash if digest function fails
        RETURN encode(digest(p_content, 'sha256'), 'hex');
END;
$$ LANGUAGE plpgsql;

-- Function to verify source integrity
CREATE OR REPLACE FUNCTION verify_source_integrity(
    p_source_id VARCHAR(255),
    p_source_type VARCHAR(50),
    p_content TEXT,
    p_algorithm VARCHAR(20) DEFAULT 'sha256'
)
RETURNS JSONB AS $$
DECLARE
    v_calculated_hash VARCHAR(64);
    v_stored_record RECORD;
    v_result JSONB;
    v_hash_match BOOLEAN;
    v_tampering_detected BOOLEAN := false;
BEGIN
    -- Calculate hash of provided content
    v_calculated_hash := calculate_content_hash(p_content, p_algorithm);
    
    -- Look up stored record
    SELECT * INTO v_stored_record
    FROM source_integrity_records
    WHERE source_id = p_source_id AND source_type = p_source_type;
    
    -- Determine if hashes match
    IF v_stored_record.id IS NULL THEN
        -- New source, create record
        INSERT INTO source_integrity_records (
            source_id, source_type, content_hash, content_size, 
            hash_algorithm, integrity_status, verification_count
        ) VALUES (
            p_source_id, p_source_type, v_calculated_hash, 
            length(p_content), p_algorithm, 'verified', 1
        );
        
        v_hash_match := true;
        v_tampering_detected := false;
    ELSE
        -- Existing source, compare hashes
        v_hash_match := (v_calculated_hash = v_stored_record.content_hash);
        v_tampering_detected := NOT v_hash_match;
        
        -- Update verification count and last verified timestamp
        UPDATE source_integrity_records
        SET 
            verification_count = verification_count + 1,
            last_verified_at = NOW(),
            integrity_status = CASE 
                WHEN v_hash_match THEN 'verified'
                ELSE 'tampered'
            END
        WHERE id = v_stored_record.id;
    END IF;
    
    -- Record verification attempt
    INSERT INTO source_integrity_verifications (
        source_integrity_id, verification_type, verification_result,
        calculated_hash, stored_hash, hash_match, tampering_detected
    ) VALUES (
        COALESCE(v_stored_record.id, (
            SELECT id FROM source_integrity_records 
            WHERE source_id = p_source_id AND source_type = p_source_type
        )),
        'on_access', 
        CASE 
            WHEN v_hash_match THEN 'passed'
            ELSE 'failed'
        END,
        v_calculated_hash,
        COALESCE(v_stored_record.content_hash, v_calculated_hash),
        v_hash_match,
        v_tampering_detected
    );
    
    -- Create alert if tampering detected
    IF v_tampering_detected THEN
        INSERT INTO source_integrity_alerts (
            source_integrity_id, alert_type, severity, alert_message
        ) VALUES (
            v_stored_record.id,
            'tampering_detected',
            'high',
            'Source content hash mismatch detected - possible tampering'
        );
    END IF;
    
    -- Build result
    v_result := jsonb_build_object(
        'verified', v_hash_match,
        'tampering_detected', v_tampering_detected,
        'calculated_hash', v_calculated_hash,
        'stored_hash', COALESCE(v_stored_record.content_hash, v_calculated_hash),
        'integrity_status', CASE 
            WHEN v_hash_match THEN 'verified'
            ELSE 'tampered'
        END,
        'verification_timestamp', NOW()
    );
    
    RETURN v_result;
END;
$$ LANGUAGE plpgsql;

-- Function to get source integrity statistics
CREATE OR REPLACE FUNCTION get_source_integrity_statistics(
    p_time_range_start TIMESTAMP WITH TIME ZONE DEFAULT NULL,
    p_time_range_end TIMESTAMP WITH TIME ZONE DEFAULT NULL
)
RETURNS JSONB AS $$
DECLARE
    v_result JSONB;
    v_time_filter TEXT := '';
BEGIN
    -- Build time filter if provided
    IF p_time_range_start IS NOT NULL AND p_time_range_end IS NOT NULL THEN
        v_time_filter := 'WHERE created_at BETWEEN $1 AND $2';
    ELSIF p_time_range_start IS NOT NULL THEN
        v_time_filter := 'WHERE created_at >= $1';
    ELSIF p_time_range_end IS NOT NULL THEN
        v_time_filter := 'WHERE created_at <= $1';
    END IF;
    
    -- Get statistics
    EXECUTE format('
        SELECT jsonb_build_object(
            ''total_sources'', COUNT(*),
            ''verified_sources'', COUNT(*) FILTER (WHERE integrity_status = ''verified''),
            ''tampered_sources'', COUNT(*) FILTER (WHERE integrity_status = ''tampered''),
            ''unknown_sources'', COUNT(*) FILTER (WHERE integrity_status = ''unknown''),
            ''pending_sources'', COUNT(*) FILTER (WHERE integrity_status = ''pending''),
            ''total_verifications'', SUM(verification_count),
            ''avg_verification_count'', AVG(verification_count),
            ''last_verification'', MAX(last_verified_at)
        )
        FROM source_integrity_records %s
    ', v_time_filter)
    INTO v_result
    USING p_time_range_start, p_time_range_end;
    
    RETURN v_result;
END;
$$ LANGUAGE plpgsql;
