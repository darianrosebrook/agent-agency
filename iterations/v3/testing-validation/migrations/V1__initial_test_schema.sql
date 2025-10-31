-- Initial migration for testing-validation
-- Creates basic test tables for E2E testing

CREATE TABLE IF NOT EXISTS test_research (
    id SERIAL PRIMARY KEY,
    topic TEXT NOT NULL,
    content TEXT,
    citations JSONB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS test_code_changes (
    id SERIAL PRIMARY KEY,
    file_path TEXT NOT NULL,
    old_content TEXT,
    new_content TEXT,
    change_type TEXT,
    applied_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE IF NOT EXISTS test_agent_runs (
    id SERIAL PRIMARY KEY,
    agent_type TEXT NOT NULL,
    task_description TEXT,
    start_time TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    end_time TIMESTAMP,
    status TEXT,
    result TEXT,
    metadata JSONB
);

CREATE INDEX idx_test_research_topic ON test_research(topic);
CREATE INDEX idx_test_code_changes_file_path ON test_code_changes(file_path);
CREATE INDEX idx_test_agent_runs_agent_type ON test_agent_runs(agent_type);
CREATE INDEX idx_test_agent_runs_status ON test_agent_runs(status);
