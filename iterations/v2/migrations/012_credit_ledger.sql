-- Credit ledger and adaptive policy tracking
-- Migration 012: Credit Ledger and Task Memory
-- Author: @darianrosebrook

-- Credit ledger (worker performance)
CREATE TABLE IF NOT EXISTS credit_ledger (
    id SERIAL PRIMARY KEY,
    agent_id TEXT NOT NULL,
    credits REAL NOT NULL DEFAULT 0,
    debits REAL NOT NULL DEFAULT 0,
    reason TEXT NOT NULL,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Task memory (long-horizon context)
CREATE TABLE IF NOT EXISTS task_memory (
    task_id TEXT NOT NULL,
    memory_key TEXT NOT NULL,
    memory_value JSONB NOT NULL DEFAULT '{}',
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (task_id, memory_key)
);

-- Adaptive policy configuration
CREATE TABLE IF NOT EXISTS adaptive_policy_config (
    id SERIAL PRIMARY KEY,
    policy_name TEXT NOT NULL UNIQUE,
    policy_config JSONB NOT NULL DEFAULT '{}',
    enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

-- Worker performance summary view
CREATE OR REPLACE VIEW worker_performance_summary AS
SELECT
    agent_id,
    SUM(credits) as total_credits,
    SUM(debits) as total_debits,
    (SUM(credits) - SUM(debits)) as net_balance,
    COUNT(*) as transaction_count,
    MIN(created_at) as first_transaction,
    MAX(created_at) as last_transaction
FROM credit_ledger
GROUP BY
    agent_id;

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_credit_ledger_agent_id ON credit_ledger (agent_id);

CREATE INDEX IF NOT EXISTS idx_credit_ledger_created_at ON credit_ledger (created_at);

CREATE INDEX IF NOT EXISTS idx_task_memory_task_id ON task_memory (task_id);

CREATE INDEX IF NOT EXISTS idx_task_memory_updated_at ON task_memory (updated_at);

CREATE INDEX IF NOT EXISTS idx_adaptive_policy_enabled ON adaptive_policy_config (enabled);

-- Trigger for adaptive policy config updated_at
CREATE TRIGGER update_adaptive_policy_config_updated_at
  BEFORE UPDATE ON adaptive_policy_config
  FOR EACH ROW
  EXECUTE FUNCTION update_updated_at_column();
