-- V3 verdicts and waivers tables

CREATE TABLE IF NOT EXISTS verdicts (
  id UUID PRIMARY KEY,
  task_id TEXT NOT NULL,
  decision TEXT NOT NULL CHECK (decision IN ('accept','reject','modify')),
  votes JSONB NOT NULL,
  dissent TEXT NOT NULL,
  remediation JSONB NOT NULL DEFAULT '[]'::jsonb,
  constitutional_refs TEXT[] NOT NULL DEFAULT '{}',
  signature BYTEA,
  hash_chain BYTEA,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_verdicts_created ON verdicts (created_at DESC);
CREATE INDEX IF NOT EXISTS idx_verdicts_task ON verdicts (task_id);
CREATE INDEX IF NOT EXISTS idx_verdicts_constitutional ON verdicts USING GIN (constitutional_refs);

CREATE TABLE IF NOT EXISTS waivers (
  id TEXT PRIMARY KEY,
  reason TEXT NOT NULL,
  scope TEXT,
  task_id TEXT NOT NULL,
  verdict_id UUID REFERENCES verdicts(id) ON DELETE SET NULL,
  created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_waivers_task ON waivers (task_id);

