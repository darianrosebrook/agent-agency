-- Enable pgvector extension for vector storage
-- Migration: 001_enable_pgvector.sql

-- Enable the pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Create a function to verify pgvector is working
CREATE OR REPLACE FUNCTION verify_pgvector()
RETURNS BOOLEAN AS $$
BEGIN
    -- Test vector operations
    SELECT '[1,2,3]'::vector <-> '[4,5,6]'::vector;
    RETURN TRUE;
EXCEPTION
    WHEN OTHERS THEN
        RETURN FALSE;
END;
$$ LANGUAGE plpgsql;

-- Log the migration
INSERT INTO migration_log (version, description, applied_at) 
VALUES ('001', 'Enable pgvector extension', NOW())
ON CONFLICT (version) DO NOTHING;
