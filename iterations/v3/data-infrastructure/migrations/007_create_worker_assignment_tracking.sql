-- Create worker assignment tracking schema
-- Migration: 007_create_worker_assignment_tracking.sql

-- ===========================================
-- WORKER ASSIGNMENTS
-- ===========================================

-- Track worker assignments to milestones
CREATE TABLE IF NOT EXISTS worker_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id UUID NOT NULL,
    milestone_id VARCHAR(255) NOT NULL,
    plan_id UUID,
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    status VARCHAR(50) NOT NULL DEFAULT 'Assigned' CHECK (status IN ('Assigned', 'Active', 'Completed', 'Failed', 'Cancelled', 'Reassigned')),
    priority VARCHAR(50) NOT NULL DEFAULT 'Normal' CHECK (priority IN ('Low', 'Normal', 'High', 'Critical')),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    failed_at TIMESTAMPTZ,
    failure_reason TEXT,
    cpu_cores INTEGER,
    memory_mb INTEGER,
    disk_mb INTEGER,
    network_mbps FLOAT,
    time_limit_ms BIGINT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for efficient querying
CREATE INDEX IF NOT EXISTS idx_worker_assignments_worker_id ON worker_assignments(worker_id);
CREATE INDEX IF NOT EXISTS idx_worker_assignments_milestone_id ON worker_assignments(milestone_id);
CREATE INDEX IF NOT EXISTS idx_worker_assignments_plan_id ON worker_assignments(plan_id);
CREATE INDEX IF NOT EXISTS idx_worker_assignments_status ON worker_assignments(status);
CREATE INDEX IF NOT EXISTS idx_worker_assignments_assigned_at ON worker_assignments(assigned_at DESC);
CREATE INDEX IF NOT EXISTS idx_worker_assignments_completed_at ON worker_assignments(completed_at DESC);

-- Composite index for active assignments query
CREATE INDEX IF NOT EXISTS idx_worker_assignments_worker_status ON worker_assignments(worker_id, status);

-- ===========================================
-- WORKER PERFORMANCE METRICS
-- ===========================================

-- Store worker performance metrics over time
CREATE TABLE IF NOT EXISTS worker_performance_metrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    worker_id UUID NOT NULL,
    measurement_time TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    tasks_completed BIGINT NOT NULL DEFAULT 0,
    tasks_failed BIGINT NOT NULL DEFAULT 0,
    avg_execution_time_ms FLOAT NOT NULL DEFAULT 0.0,
    success_rate FLOAT NOT NULL DEFAULT 0.0 CHECK (success_rate >= 0.0 AND success_rate <= 1.0),
    performance_score FLOAT NOT NULL DEFAULT 0.0 CHECK (performance_score >= 0.0 AND performance_score <= 1.0),
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for performance queries
CREATE INDEX IF NOT EXISTS idx_worker_performance_worker_id ON worker_performance_metrics(worker_id);
CREATE INDEX IF NOT EXISTS idx_worker_performance_measurement_time ON worker_performance_metrics(measurement_time DESC);
CREATE INDEX IF NOT EXISTS idx_worker_performance_score ON worker_performance_metrics(performance_score DESC);

-- Composite index for worker performance history
CREATE INDEX IF NOT EXISTS idx_worker_performance_worker_time ON worker_performance_metrics(worker_id, measurement_time DESC);

-- ===========================================
-- ASSIGNMENT HISTORY
-- ===========================================

-- Track assignment history and status changes
CREATE TABLE IF NOT EXISTS assignment_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES worker_assignments(id) ON DELETE CASCADE,
    worker_id UUID NOT NULL,
    milestone_id VARCHAR(255) NOT NULL,
    event_type VARCHAR(50) NOT NULL CHECK (event_type IN ('assigned', 'started', 'completed', 'failed', 'cancelled', 'reassigned', 'status_changed')),
    old_status VARCHAR(50),
    new_status VARCHAR(50),
    event_description TEXT,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for history queries
CREATE INDEX IF NOT EXISTS idx_assignment_history_assignment_id ON assignment_history(assignment_id);
CREATE INDEX IF NOT EXISTS idx_assignment_history_worker_id ON assignment_history(worker_id);
CREATE INDEX IF NOT EXISTS idx_assignment_history_milestone_id ON assignment_history(milestone_id);
CREATE INDEX IF NOT EXISTS idx_assignment_history_event_type ON assignment_history(event_type);
CREATE INDEX IF NOT EXISTS idx_assignment_history_created_at ON assignment_history(created_at DESC);

-- ===========================================
-- ASSIGNMENT STATISTICS VIEW
-- ===========================================

-- View for current assignment statistics
CREATE OR REPLACE VIEW assignment_statistics AS
SELECT
    w.id as worker_id,
    w.name as worker_name,
    COUNT(DISTINCT CASE WHEN wa.status IN ('Assigned', 'Active') THEN wa.id END) as active_assignments,
    COUNT(DISTINCT CASE WHEN wa.status = 'Completed' THEN wa.id END) as completed_assignments,
    COUNT(DISTINCT CASE WHEN wa.status = 'Failed' THEN wa.id END) as failed_assignments,
    COALESCE(AVG(wpm.performance_score), 0.0) as avg_performance_score,
    COALESCE(MAX(wpm.measurement_time), w.created_at) as last_performance_update
FROM workers w
LEFT JOIN worker_assignments wa ON w.id = wa.worker_id
LEFT JOIN worker_performance_metrics wpm ON w.id = wpm.worker_id
GROUP BY w.id, w.name, w.created_at;

-- ===========================================
-- HELPER FUNCTIONS
-- ===========================================

-- Function to update assignment status and create history entry
CREATE OR REPLACE FUNCTION update_assignment_status(
    p_assignment_id UUID,
    p_new_status VARCHAR(50),
    p_event_description TEXT DEFAULT NULL
)
RETURNS VOID AS $$
DECLARE
    v_old_status VARCHAR(50);
    v_worker_id UUID;
    v_milestone_id VARCHAR(255);
BEGIN
    -- Get current status
    SELECT status, worker_id, milestone_id INTO v_old_status, v_worker_id, v_milestone_id
    FROM worker_assignments
    WHERE id = p_assignment_id;

    IF v_old_status IS NULL THEN
        RAISE EXCEPTION 'Assignment not found: %', p_assignment_id;
    END IF;

    -- Update assignment
    UPDATE worker_assignments
    SET 
        status = p_new_status,
        updated_at = NOW(),
        started_at = CASE WHEN p_new_status = 'Active' AND started_at IS NULL THEN NOW() ELSE started_at END,
        completed_at = CASE WHEN p_new_status IN ('Completed', 'Failed') AND completed_at IS NULL THEN NOW() ELSE completed_at END,
        failed_at = CASE WHEN p_new_status = 'Failed' AND failed_at IS NULL THEN NOW() ELSE failed_at END
    WHERE id = p_assignment_id;

    -- Create history entry
    INSERT INTO assignment_history (
        assignment_id,
        worker_id,
        milestone_id,
        event_type,
        old_status,
        new_status,
        event_description
    ) VALUES (
        p_assignment_id,
        v_worker_id,
        v_milestone_id,
        CASE 
            WHEN p_new_status = 'Active' THEN 'started'
            WHEN p_new_status = 'Completed' THEN 'completed'
            WHEN p_new_status = 'Failed' THEN 'failed'
            WHEN p_new_status = 'Cancelled' THEN 'cancelled'
            WHEN p_new_status = 'Reassigned' THEN 'reassigned'
            ELSE 'status_changed'
        END,
        v_old_status,
        p_new_status,
        COALESCE(p_event_description, 'Status changed from ' || v_old_status || ' to ' || p_new_status)
    );
END;
$$ LANGUAGE plpgsql;

-- Function to get latest performance metrics for a worker
CREATE OR REPLACE FUNCTION get_latest_worker_performance(p_worker_id UUID)
RETURNS TABLE (
    worker_id UUID,
    tasks_completed BIGINT,
    tasks_failed BIGINT,
    avg_execution_time_ms FLOAT,
    success_rate FLOAT,
    performance_score FLOAT,
    measurement_time TIMESTAMPTZ
) AS $$
BEGIN
    RETURN QUERY
    SELECT 
        wpm.worker_id,
        wpm.tasks_completed,
        wpm.tasks_failed,
        wpm.avg_execution_time_ms,
        wpm.success_rate,
        wpm.performance_score,
        wpm.measurement_time
    FROM worker_performance_metrics wpm
    WHERE wpm.worker_id = p_worker_id
    ORDER BY wpm.measurement_time DESC
    LIMIT 1;
END;
$$ LANGUAGE plpgsql;


