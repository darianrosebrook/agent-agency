-- Migration: MCP Memory Permissions & User Setup
-- Description: Create MCP user with limited permissions for memory operations
-- Version: 018
-- Date: 2025-01-25

-- ===========================================
-- MCP USER CREATION & PERMISSIONS
-- ===========================================

-- Create MCP user with limited permissions
DO $$
BEGIN
    -- Create user if it doesn't exist
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'mcp_user') THEN
        CREATE USER mcp_user WITH PASSWORD 'secure_mcp_password_change_me';
    END IF;

    -- Grant connection to database
    GRANT CONNECT ON DATABASE agent_agency TO mcp_user;

    -- Grant usage on schema
    GRANT USAGE ON SCHEMA public TO mcp_user;
END
$$;

-- ===========================================
-- MCP READ PERMISSIONS (ALL TABLES)
-- ===========================================

-- Read access to all existing tables for search/discovery
GRANT SELECT ON ALL TABLES IN SCHEMA public TO mcp_user;

-- Read access to future tables
ALTER DEFAULT PRIVILEGES IN SCHEMA public GRANT SELECT ON TABLES TO mcp_user;

-- ===========================================
-- MCP MEMORY WRITE PERMISSIONS
-- ===========================================

-- Write permissions for memory operations only
GRANT INSERT, UPDATE ON agent_experiences TO mcp_user;
GRANT INSERT, UPDATE ON memory_embeddings TO mcp_user;
GRANT INSERT, UPDATE ON knowledge_graph_entities TO mcp_user;
GRANT INSERT, UPDATE ON knowledge_graph_relationships TO mcp_user;
GRANT INSERT, UPDATE ON temporal_analysis_results TO mcp_user;
GRANT INSERT, UPDATE ON memory_provenance TO mcp_user;
GRANT INSERT, UPDATE ON offloaded_contexts TO mcp_user;
GRANT INSERT, UPDATE ON memory_system_metrics TO mcp_user;

-- ===========================================
-- MCP FUNCTION PERMISSIONS
-- ===========================================

-- Grant execute permissions on memory-related functions
GRANT EXECUTE ON FUNCTION find_similar_memories TO mcp_user;
GRANT EXECUTE ON FUNCTION get_memory_system_health TO mcp_user;
GRANT EXECUTE ON FUNCTION calculate_memory_relevance TO mcp_user;
GRANT EXECUTE ON FUNCTION update_memory_access TO mcp_user;
GRANT EXECUTE ON FUNCTION cleanup_expired_contexts TO mcp_user;
GRANT EXECUTE ON FUNCTION apply_memory_decay_batch TO mcp_user;
GRANT EXECUTE ON FUNCTION migrate_experiences_to_embeddings TO mcp_user;

-- ===========================================
-- MCP SEQUENCE PERMISSIONS (for auto-increment)
-- ===========================================

-- Grant usage on sequences for ID generation
GRANT USAGE ON ALL SEQUENCES IN SCHEMA public TO mcp_user;

-- ===========================================
-- MCP TYPE USAGE PERMISSIONS
-- ===========================================

-- Grant usage on custom types
GRANT USAGE ON TYPE entity_type TO mcp_user;
GRANT USAGE ON TYPE relationship_type TO mcp_user;
GRANT USAGE ON TYPE memory_operation TO mcp_user;

-- ===========================================
-- SECURITY NOTES
-- ===========================================

/*
SECURITY MODEL:
- MCP user can READ all data for search/discovery
- MCP user can only WRITE to memory tables (no admin operations)
- MCP user cannot DROP, ALTER, or DELETE tables
- MCP user cannot access admin/system configuration
- All memory operations are audited via provenance tracking

ADMIN USER retains:
- Full database access
- Schema modifications
- User management
- System configuration
- Backup/restore operations
*/

-- ===========================================
-- VERIFICATION QUERIES
-- ===========================================

-- Verify MCP user permissions (run as admin)
/*
-- Check user exists
SELECT rolname FROM pg_roles WHERE rolname = 'mcp_user';

-- Check table permissions
SELECT grantee, table_name, privilege_type
FROM information_schema.role_table_grants
WHERE grantee = 'mcp_user'
ORDER BY table_name, privilege_type;

-- Check function permissions
SELECT proname, proacl
FROM pg_proc
WHERE proname IN ('find_similar_memories', 'get_memory_system_health')
AND proacl IS NOT NULL;
*/
