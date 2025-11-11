-- Migration 025: Create Task Comments Table
-- Creates database table for task comments that can be read as context by agents
-- Author: @darianrosebrook

BEGIN;

-- ===========================================
-- TASK COMMENTS TABLE
-- ===========================================

CREATE TABLE IF NOT EXISTS task_comments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    task_id UUID NOT NULL REFERENCES tasks(id) ON DELETE CASCADE,
    content TEXT NOT NULL,
    created_by VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    
    -- Constraints
    CONSTRAINT task_comments_content_not_empty CHECK (length(trim(content)) > 0)
);

-- Indexes for task_comments
CREATE INDEX IF NOT EXISTS idx_task_comments_task_id ON task_comments(task_id);
CREATE INDEX IF NOT EXISTS idx_task_comments_created_at ON task_comments(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_task_comments_created_by ON task_comments(created_by);

-- Composite index for efficient task comment queries
CREATE INDEX IF NOT EXISTS idx_task_comments_task_created ON task_comments(task_id, created_at DESC);

-- ===========================================
-- TRIGGERS
-- ===========================================

-- Update updated_at timestamp on comment update
CREATE OR REPLACE FUNCTION update_task_comment_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_update_task_comment_updated_at
    BEFORE UPDATE ON task_comments
    FOR EACH ROW
    EXECUTE FUNCTION update_task_comment_updated_at();

-- ===========================================
-- COMMENTS
-- ===========================================

COMMENT ON TABLE task_comments IS 'Comments on tasks that can be read as context by agents when viewing tasks';
COMMENT ON COLUMN task_comments.task_id IS 'Foreign key to tasks. Comments are automatically deleted when task is deleted via CASCADE.';
COMMENT ON COLUMN task_comments.content IS 'Comment content text';
COMMENT ON COLUMN task_comments.created_by IS 'User or agent identifier who created the comment';

-- Log migration
INSERT INTO migration_log (version, description, applied_at)
VALUES ('025', 'create_task_comments', NOW())
ON CONFLICT (version) DO NOTHING;

COMMIT;

