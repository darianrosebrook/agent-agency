-- Migration 015: Create Observation & API Tables
-- Creates database tables for observation, monitoring, and API functionality:
-- saved_queries, provenance_entries, audit_trail_entries

-- ===========================================
-- SAVED QUERIES TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS saved_queries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    query_sql TEXT NOT NULL,
    parameters TEXT,
    created_by VARCHAR(255) NOT NULL DEFAULT 'system',
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for saved_queries
CREATE INDEX IF NOT EXISTS idx_saved_queries_name ON saved_queries(name);
CREATE INDEX IF NOT EXISTS idx_saved_queries_created_by ON saved_queries(created_by);
CREATE INDEX IF NOT EXISTS idx_saved_queries_is_public ON saved_queries(is_public);
CREATE INDEX IF NOT EXISTS idx_saved_queries_created_at ON saved_queries(created_at);
CREATE INDEX IF NOT EXISTS idx_saved_queries_updated_at ON saved_queries(updated_at DESC);

-- ===========================================
-- PROVENANCE ENTRIES TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS provenance_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL,
    action VARCHAR(100) NOT NULL,
    actor VARCHAR(255) NOT NULL,
    resource_id UUID,
    resource_type VARCHAR(100),
    change_summary TEXT NOT NULL,
    timestamp TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb
);

-- Indexes for provenance_entries
CREATE INDEX IF NOT EXISTS idx_provenance_entries_task_id ON provenance_entries(task_id);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_action ON provenance_entries(action);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_actor ON provenance_entries(actor);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_resource_id ON provenance_entries(resource_id);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_resource_type ON provenance_entries(resource_type);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_timestamp ON provenance_entries(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_created_at ON provenance_entries(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_provenance_entries_metadata ON provenance_entries USING GIN(metadata);
-- Index for commit_hash queries (stored in metadata as text)
-- Using btree index on extracted text value for commit_hash lookups
CREATE INDEX IF NOT EXISTS idx_provenance_entries_commit_hash ON provenance_entries ((metadata->>'commit_hash')) WHERE metadata->>'commit_hash' IS NOT NULL;

-- Foreign key to tasks table (if it exists)
-- Note: This will fail if tasks table doesn't exist, but that's expected during migration ordering
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_name = 'tasks') THEN
        ALTER TABLE provenance_entries 
        ADD CONSTRAINT fk_provenance_entries_task_id 
        FOREIGN KEY (task_id) REFERENCES tasks(id) ON DELETE CASCADE;
    END IF;
EXCEPTION
    WHEN duplicate_object THEN NULL;
END $$;

-- ===========================================
-- AUDIT TRAIL ENTRIES TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS audit_trail_entries (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    entity_type VARCHAR(100) NOT NULL,
    entity_id UUID NOT NULL,
    action VARCHAR(100) NOT NULL,
    details JSONB NOT NULL DEFAULT '{}'::jsonb,
    user_id VARCHAR(255),
    ip_address VARCHAR(45), -- IPv6 max length
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for audit_trail_entries
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_entity_type ON audit_trail_entries(entity_type);
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_entity_id ON audit_trail_entries(entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_action ON audit_trail_entries(action);
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_user_id ON audit_trail_entries(user_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_created_at ON audit_trail_entries(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_entity ON audit_trail_entries(entity_type, entity_id);
CREATE INDEX IF NOT EXISTS idx_audit_trail_entries_details ON audit_trail_entries USING GIN(details);

-- ===========================================
-- AUDIT LOGS TABLE (Alternative/Simplified)
-- ===========================================

-- This table is referenced in test code and may be used for simpler audit logging
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(100) NOT NULL,
    event_data JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for audit_logs
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_type ON audit_logs(event_type);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_event_data ON audit_logs USING GIN(event_data);

-- ===========================================
-- TRIGGERS FOR UPDATED_AT TIMESTAMPS
-- ===========================================

-- Reuse existing update_updated_at_column function
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create trigger for saved_queries updated_at
CREATE TRIGGER update_saved_queries_updated_at BEFORE UPDATE ON saved_queries FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- ===========================================
-- LOG MIGRATION
-- ===========================================

INSERT INTO migration_log (version, description, applied_at)
VALUES ('015', 'Create observation tables (saved_queries, provenance_entries, audit_trail_entries, audit_logs)', NOW())
ON CONFLICT (version) DO NOTHING;

