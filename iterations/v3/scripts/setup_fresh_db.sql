-- Setup script for fresh agent_agency database
-- Run this as a superuser (darianrosebrook or postgres)

-- Create agent_agency user if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_user WHERE usename = 'agent_agency') THEN
        CREATE USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
        RAISE NOTICE 'User agent_agency created';
    ELSE
        -- Update password if user exists
        ALTER USER agent_agency WITH PASSWORD 'agent_agency_dev' SUPERUSER;
        RAISE NOTICE 'User agent_agency password updated';
    END IF;
END $$;

-- Grant all privileges on database to agent_agency user
GRANT ALL PRIVILEGES ON DATABASE agent_agency TO agent_agency;

-- Connect to agent_agency database and enable pgvector
\c agent_agency

-- Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- Verify pgvector is enabled
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_extension WHERE extname = 'vector') THEN
        RAISE NOTICE 'pgvector extension enabled successfully';
    ELSE
        RAISE EXCEPTION 'Failed to enable pgvector extension';
    END IF;
END $$;





