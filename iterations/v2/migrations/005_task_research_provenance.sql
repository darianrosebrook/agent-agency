-- Migration: Task Research Provenance Table
-- Description: Stores audit trail of research performed for tasks
-- Created: 2025-10-12
-- Part of: ARBITER-006 Phase 4 (Task-Driven Research)

-- Create task_research_provenance table
CREATE TABLE IF NOT EXISTS task_research_provenance (
  id SERIAL PRIMARY KEY,
  task_id VARCHAR(255) NOT NULL,
  queries JSONB NOT NULL,
  findings_count INTEGER NOT NULL DEFAULT 0,
  confidence DECIMAL(3, 2) NOT NULL DEFAULT 0,
  performed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
  duration_ms INTEGER,
  successful BOOLEAN NOT NULL DEFAULT TRUE,
  error TEXT,
  created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

-- Indexes for performance
CONSTRAINT task_research_provenance_confidence_check CHECK (confidence >= 0 AND confidence <= 1)
);

-- Create indexes
CREATE INDEX IF NOT EXISTS idx_task_research_provenance_task_id ON task_research_provenance (task_id);

CREATE INDEX IF NOT EXISTS idx_task_research_provenance_performed_at ON task_research_provenance (performed_at DESC);

CREATE INDEX IF NOT EXISTS idx_task_research_provenance_successful ON task_research_provenance (successful);

-- Create GIN index for queries JSONB
CREATE INDEX IF NOT EXISTS idx_task_research_provenance_queries ON task_research_provenance USING GIN (queries);

-- Add comment
COMMENT ON
TABLE task_research_provenance IS 'Audit trail of research operations performed for tasks';

COMMENT ON COLUMN task_research_provenance.task_id IS 'ID of task that research was performed for';

COMMENT ON COLUMN task_research_provenance.queries IS 'JSON array of queries executed';

COMMENT ON COLUMN task_research_provenance.findings_count IS 'Number of findings returned';

COMMENT ON COLUMN task_research_provenance.confidence IS 'Overall confidence score (0-1)';

COMMENT ON COLUMN task_research_provenance.performed_at IS 'When research was performed';

COMMENT ON COLUMN task_research_provenance.duration_ms IS 'How long research took in milliseconds';

COMMENT ON COLUMN task_research_provenance.successful IS 'Whether research completed successfully';

COMMENT ON COLUMN task_research_provenance.error IS 'Error message if research failed';