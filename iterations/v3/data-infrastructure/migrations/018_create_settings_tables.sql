-- Migration 018: Create Settings Management Tables
-- Creates database tables for user settings, app settings, integrations, and API keys

-- ===========================================
-- USER SETTINGS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS user_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    setting_key VARCHAR(255) NOT NULL,
    setting_value JSONB NOT NULL DEFAULT '{}'::jsonb,
    setting_type VARCHAR(50) NOT NULL DEFAULT 'preference',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    UNIQUE(user_id, setting_key)
);

-- Indexes for user_settings
CREATE INDEX IF NOT EXISTS idx_user_settings_user_id ON user_settings(user_id);
CREATE INDEX IF NOT EXISTS idx_user_settings_setting_key ON user_settings(setting_key);
CREATE INDEX IF NOT EXISTS idx_user_settings_setting_type ON user_settings(setting_type);
CREATE INDEX IF NOT EXISTS idx_user_settings_updated_at ON user_settings(updated_at DESC);

-- ===========================================
-- APP SETTINGS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS app_settings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    setting_key VARCHAR(255) NOT NULL UNIQUE,
    setting_value JSONB NOT NULL DEFAULT '{}'::jsonb,
    setting_type VARCHAR(50) NOT NULL DEFAULT 'configuration',
    description TEXT,
    is_public BOOLEAN NOT NULL DEFAULT FALSE,
    created_by VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_by VARCHAR(255)
);

-- Indexes for app_settings
CREATE INDEX IF NOT EXISTS idx_app_settings_setting_key ON app_settings(setting_key);
CREATE INDEX IF NOT EXISTS idx_app_settings_setting_type ON app_settings(setting_type);
CREATE INDEX IF NOT EXISTS idx_app_settings_is_public ON app_settings(is_public);
CREATE INDEX IF NOT EXISTS idx_app_settings_updated_at ON app_settings(updated_at DESC);

-- ===========================================
-- INTEGRATIONS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS integrations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    integration_type VARCHAR(100) NOT NULL,
    provider VARCHAR(100) NOT NULL,
    configuration JSONB NOT NULL DEFAULT '{}'::jsonb,
    credentials JSONB NOT NULL DEFAULT '{}'::jsonb,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_enabled BOOLEAN NOT NULL DEFAULT FALSE,
    last_sync_at TIMESTAMP WITH TIME ZONE,
    sync_status VARCHAR(50) DEFAULT 'pending',
    sync_error TEXT,
    created_by VARCHAR(255) NOT NULL DEFAULT 'system',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_by VARCHAR(255)
);

-- Indexes for integrations
CREATE INDEX IF NOT EXISTS idx_integrations_name ON integrations(name);
CREATE INDEX IF NOT EXISTS idx_integrations_integration_type ON integrations(integration_type);
CREATE INDEX IF NOT EXISTS idx_integrations_provider ON integrations(provider);
CREATE INDEX IF NOT EXISTS idx_integrations_is_active ON integrations(is_active);
CREATE INDEX IF NOT EXISTS idx_integrations_is_enabled ON integrations(is_enabled);
CREATE INDEX IF NOT EXISTS idx_integrations_sync_status ON integrations(sync_status);
CREATE INDEX IF NOT EXISTS idx_integrations_updated_at ON integrations(updated_at DESC);

-- ===========================================
-- API KEYS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS api_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    key_name VARCHAR(255) NOT NULL,
    key_hash VARCHAR(255) NOT NULL UNIQUE,
    key_prefix VARCHAR(20) NOT NULL,
    scopes TEXT[] NOT NULL DEFAULT ARRAY[]::TEXT[],
    rate_limit_per_minute INTEGER DEFAULT 60,
    rate_limit_per_hour INTEGER DEFAULT 1000,
    rate_limit_per_day INTEGER DEFAULT 10000,
    last_used_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    is_revoked BOOLEAN NOT NULL DEFAULT FALSE,
    revoked_at TIMESTAMP WITH TIME ZONE,
    revoked_reason TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_by VARCHAR(255) NOT NULL DEFAULT 'system'
);

-- Indexes for api_keys
CREATE INDEX IF NOT EXISTS idx_api_keys_user_id ON api_keys(user_id);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_hash ON api_keys(key_hash);
CREATE INDEX IF NOT EXISTS idx_api_keys_key_prefix ON api_keys(key_prefix);
CREATE INDEX IF NOT EXISTS idx_api_keys_is_active ON api_keys(is_active);
CREATE INDEX IF NOT EXISTS idx_api_keys_is_revoked ON api_keys(is_revoked);
CREATE INDEX IF NOT EXISTS idx_api_keys_expires_at ON api_keys(expires_at);
CREATE INDEX IF NOT EXISTS idx_api_keys_last_used_at ON api_keys(last_used_at DESC);
CREATE INDEX IF NOT EXISTS idx_api_keys_created_at ON api_keys(created_at DESC);


-- ===========================================
-- COMMENTS
-- ===========================================

COMMENT ON TABLE user_settings IS 'User-specific preferences and settings (theme, notifications, etc.)';
COMMENT ON TABLE app_settings IS 'System-wide application configuration settings';
COMMENT ON TABLE integrations IS 'External service integrations (GitHub, Slack, etc.)';
COMMENT ON TABLE api_keys IS 'API keys for programmatic access with rate limiting and scopes';
COMMENT ON COLUMN app_settings.setting_type IS 'Type of setting: configuration, feature_flag, etc.';
COMMENT ON COLUMN integrations.integration_type IS 'Type of integration: webhook, oauth, api, etc.';
COMMENT ON COLUMN integrations.provider IS 'Service provider: github, slack, discord, etc.';
COMMENT ON COLUMN integrations.credentials IS 'Encrypted credentials for the integration';
COMMENT ON COLUMN api_keys.key_hash IS 'SHA-256 hash of the API key (never store plain keys)';
COMMENT ON COLUMN api_keys.key_prefix IS 'First 8 characters of the key for identification';
COMMENT ON COLUMN api_keys.scopes IS 'Array of permission scopes for this API key';


