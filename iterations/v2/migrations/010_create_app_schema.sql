/**
 * Migration: Create App Schema for Tenant Context
 * 
 * @author @darianrosebrook
 * @migration 010
 * @component Database Infrastructure
 * 
 * Creates the app schema and sets up tenant context variables for RLS.
 */

-- ============================================================================
-- Create App Schema
-- ============================================================================

CREATE SCHEMA IF NOT EXISTS app;

-- ============================================================================
-- Create Tenant Context Variables
-- ============================================================================

-- Create a function to set tenant context
CREATE OR REPLACE FUNCTION app.set_tenant_context(tenant_id TEXT, user_id TEXT DEFAULT NULL, session_id TEXT DEFAULT NULL)
RETURNS VOID AS $$
BEGIN
    -- Set session variables for RLS policies
    PERFORM set_config('app.current_tenant', tenant_id, true);
    IF user_id IS NOT NULL THEN
        PERFORM set_config('app.current_user', user_id, true);
    END IF;
    IF session_id IS NOT NULL THEN
        PERFORM set_config('app.current_session', session_id, true);
    END IF;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Create Helper Functions
-- ============================================================================

-- Function to get current tenant
CREATE OR REPLACE FUNCTION app.current_tenant()
RETURNS TEXT AS $$
BEGIN
    RETURN current_setting('app.current_tenant', true);
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to get current user
CREATE OR REPLACE FUNCTION app.current_user()
RETURNS TEXT AS $$
BEGIN
    RETURN current_setting('app.current_user', true);
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- Function to get current session
CREATE OR REPLACE FUNCTION app.current_session()
RETURNS TEXT AS $$
BEGIN
    RETURN current_setting('app.current_session', true);
EXCEPTION
    WHEN OTHERS THEN
        RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- ============================================================================
-- Comments for Documentation
-- ============================================================================

COMMENT ON SCHEMA app IS 'Application schema for tenant context and RLS support';

COMMENT ON FUNCTION app.set_tenant_context IS 'Sets tenant context variables for RLS policies';

COMMENT ON FUNCTION app.current_tenant IS 'Gets the current tenant ID from session context';

COMMENT ON FUNCTION app.current_user IS 'Gets the current user ID from session context';

COMMENT ON FUNCTION app.current_session IS 'Gets the current session ID from session context';

-- ============================================================================
-- Migration Complete
-- ============================================================================
