-- Migration 031: Create Milestone Completions Table
-- Creates database tables for curriculum learning milestone tracking:
-- milestone_completions, curriculum_profiles, curriculum_paths
-- @author @darianrosebrook

-- ===========================================
-- CURRICULUM PROFILES TABLE
-- ===========================================
-- Stores agent skill profiles for curriculum learning

CREATE TABLE IF NOT EXISTS curriculum_profiles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL UNIQUE,
    overall_level VARCHAR(50) NOT NULL DEFAULT 'beginner',
    total_tasks_completed INTEGER NOT NULL DEFAULT 0,
    total_tasks_succeeded INTEGER NOT NULL DEFAULT 0,
    skills JSONB NOT NULL DEFAULT '{}',
    last_updated TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for curriculum_profiles
CREATE INDEX IF NOT EXISTS idx_curriculum_profiles_agent_id
    ON curriculum_profiles(agent_id);
CREATE INDEX IF NOT EXISTS idx_curriculum_profiles_overall_level
    ON curriculum_profiles(overall_level);
CREATE INDEX IF NOT EXISTS idx_curriculum_profiles_last_updated
    ON curriculum_profiles(last_updated DESC);

-- ===========================================
-- CURRICULUM PATHS TABLE
-- ===========================================
-- Stores predefined curriculum paths with milestones

CREATE TABLE IF NOT EXISTS curriculum_paths (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    path_id VARCHAR(255) NOT NULL UNIQUE,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    domains JSONB NOT NULL DEFAULT '[]',
    milestones JSONB NOT NULL DEFAULT '[]',
    difficulty_progression JSONB NOT NULL DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for curriculum_paths
CREATE INDEX IF NOT EXISTS idx_curriculum_paths_path_id
    ON curriculum_paths(path_id);
CREATE INDEX IF NOT EXISTS idx_curriculum_paths_active
    ON curriculum_paths(is_active);

-- ===========================================
-- MILESTONE COMPLETIONS TABLE
-- ===========================================
-- Tracks completion of individual curriculum milestones by agents

CREATE TABLE IF NOT EXISTS milestone_completions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    milestone_id VARCHAR(255) NOT NULL,
    domain VARCHAR(100) NOT NULL,
    required_level VARCHAR(50) NOT NULL,
    target_level VARCHAR(50) NOT NULL,
    complexity VARCHAR(50) NOT NULL,
    success BOOLEAN NOT NULL,
    quality_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    completion_time_ms INTEGER,
    attempts INTEGER NOT NULL DEFAULT 1,
    prerequisites_met BOOLEAN NOT NULL DEFAULT TRUE,
    metadata JSONB DEFAULT '{}',
    completed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),

    -- Foreign key constraint (optional, curriculum_profiles may not exist for all agents)
    CONSTRAINT fk_milestone_completions_agent
        FOREIGN KEY (agent_id) REFERENCES curriculum_profiles(agent_id)
        ON DELETE CASCADE,

    -- Ensure unique completion per agent-milestone pair
    CONSTRAINT unique_agent_milestone
        UNIQUE (agent_id, milestone_id)
);

-- Indexes for milestone_completions
CREATE INDEX IF NOT EXISTS idx_milestone_completions_agent_id
    ON milestone_completions(agent_id);
CREATE INDEX IF NOT EXISTS idx_milestone_completions_milestone_id
    ON milestone_completions(milestone_id);
CREATE INDEX IF NOT EXISTS idx_milestone_completions_domain
    ON milestone_completions(domain);
CREATE INDEX IF NOT EXISTS idx_milestone_completions_success
    ON milestone_completions(success);
CREATE INDEX IF NOT EXISTS idx_milestone_completions_completed_at
    ON milestone_completions(completed_at DESC);
CREATE INDEX IF NOT EXISTS idx_milestone_completions_agent_domain
    ON milestone_completions(agent_id, domain);
CREATE INDEX IF NOT EXISTS idx_milestone_completions_agent_success
    ON milestone_completions(agent_id, success);

-- ===========================================
-- LEARNING HISTORY TABLE
-- ===========================================
-- Tracks detailed learning outcomes for curriculum analysis

CREATE TABLE IF NOT EXISTS learning_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    agent_id UUID NOT NULL,
    task_id UUID REFERENCES tasks(id) ON DELETE SET NULL,
    domain VARCHAR(100) NOT NULL,
    complexity VARCHAR(50) NOT NULL,
    adjusted_complexity VARCHAR(50),
    skill_level_before VARCHAR(50) NOT NULL,
    skill_level_after VARCHAR(50),
    success BOOLEAN NOT NULL,
    quality_score DOUBLE PRECISION NOT NULL DEFAULT 0.0,
    execution_metadata JSONB DEFAULT '{}',
    recorded_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for learning_history
CREATE INDEX IF NOT EXISTS idx_learning_history_agent_id
    ON learning_history(agent_id);
CREATE INDEX IF NOT EXISTS idx_learning_history_task_id
    ON learning_history(task_id);
CREATE INDEX IF NOT EXISTS idx_learning_history_domain
    ON learning_history(domain);
CREATE INDEX IF NOT EXISTS idx_learning_history_success
    ON learning_history(success);
CREATE INDEX IF NOT EXISTS idx_learning_history_recorded_at
    ON learning_history(recorded_at DESC);
CREATE INDEX IF NOT EXISTS idx_learning_history_agent_domain_success
    ON learning_history(agent_id, domain, success);

-- ===========================================
-- FUNCTIONS: Curriculum Management
-- ===========================================

-- Function to get or create curriculum profile
CREATE OR REPLACE FUNCTION get_or_create_curriculum_profile(p_agent_id UUID)
RETURNS UUID AS $$
DECLARE
    profile_id UUID;
BEGIN
    -- Try to find existing profile
    SELECT id INTO profile_id
    FROM curriculum_profiles
    WHERE agent_id = p_agent_id;

    -- If not found, create new profile
    IF profile_id IS NULL THEN
        INSERT INTO curriculum_profiles (agent_id)
        VALUES (p_agent_id)
        RETURNING id INTO profile_id;
    END IF;

    RETURN profile_id;
END;
$$ LANGUAGE plpgsql;

-- Function to record milestone completion
CREATE OR REPLACE FUNCTION record_milestone_completion(
    p_agent_id UUID,
    p_milestone_id VARCHAR(255),
    p_domain VARCHAR(100),
    p_required_level VARCHAR(50),
    p_target_level VARCHAR(50),
    p_complexity VARCHAR(50),
    p_success BOOLEAN,
    p_quality_score DOUBLE PRECISION,
    p_completion_time_ms INTEGER,
    p_attempts INTEGER,
    p_prerequisites_met BOOLEAN,
    p_metadata JSONB
)
RETURNS UUID AS $$
DECLARE
    completion_id UUID;
BEGIN
    -- Ensure profile exists
    PERFORM get_or_create_curriculum_profile(p_agent_id);

    -- Insert or update milestone completion
    INSERT INTO milestone_completions (
        agent_id, milestone_id, domain, required_level, target_level,
        complexity, success, quality_score, completion_time_ms,
        attempts, prerequisites_met, metadata
    )
    VALUES (
        p_agent_id, p_milestone_id, p_domain, p_required_level, p_target_level,
        p_complexity, p_success, p_quality_score, p_completion_time_ms,
        p_attempts, p_prerequisites_met, p_metadata
    )
    ON CONFLICT (agent_id, milestone_id) DO UPDATE SET
        success = EXCLUDED.success,
        quality_score = GREATEST(milestone_completions.quality_score, EXCLUDED.quality_score),
        completion_time_ms = EXCLUDED.completion_time_ms,
        attempts = milestone_completions.attempts + 1,
        completed_at = NOW()
    RETURNING id INTO completion_id;

    RETURN completion_id;
END;
$$ LANGUAGE plpgsql;

-- Function to check if milestone prerequisites are met
CREATE OR REPLACE FUNCTION check_milestone_prerequisites(
    p_agent_id UUID,
    p_prerequisite_ids JSONB
)
RETURNS BOOLEAN AS $$
DECLARE
    prerequisite_id TEXT;
    completed_count INTEGER := 0;
    total_count INTEGER := 0;
BEGIN
    -- If no prerequisites, return true
    IF p_prerequisite_ids IS NULL OR jsonb_array_length(p_prerequisite_ids) = 0 THEN
        RETURN TRUE;
    END IF;

    -- Count total prerequisites
    total_count := jsonb_array_length(p_prerequisite_ids);

    -- Count completed prerequisites
    FOR prerequisite_id IN SELECT jsonb_array_elements_text(p_prerequisite_ids)
    LOOP
        IF EXISTS (
            SELECT 1 FROM milestone_completions
            WHERE agent_id = p_agent_id
            AND milestone_id = prerequisite_id
            AND success = TRUE
        ) THEN
            completed_count := completed_count + 1;
        END IF;
    END LOOP;

    -- All prerequisites must be met
    RETURN completed_count = total_count;
END;
$$ LANGUAGE plpgsql;

-- Function to get agent skill level for domain
CREATE OR REPLACE FUNCTION get_agent_skill_level(
    p_agent_id UUID,
    p_domain VARCHAR(100)
)
RETURNS VARCHAR(50) AS $$
DECLARE
    skill_level VARCHAR(50);
BEGIN
    -- Get from curriculum profile
    SELECT skills->>p_domain
    INTO skill_level
    FROM curriculum_profiles
    WHERE agent_id = p_agent_id;

    -- Return beginner if not found
    IF skill_level IS NULL THEN
        RETURN 'beginner';
    END IF;

    RETURN skill_level;
END;
$$ LANGUAGE plpgsql;

-- ===========================================
-- VIEWS: Curriculum Analytics
-- ===========================================

-- Agent progress overview
CREATE OR REPLACE VIEW curriculum_agent_progress AS
SELECT
    cp.agent_id,
    cp.overall_level,
    cp.total_tasks_completed,
    cp.total_tasks_succeeded,
    CASE
        WHEN cp.total_tasks_completed > 0
        THEN (cp.total_tasks_succeeded::DOUBLE PRECISION / cp.total_tasks_completed)
        ELSE 0.0
    END as success_rate,
    jsonb_object_keys(cp.skills) as domains,
    cp.last_updated
FROM curriculum_profiles cp;

-- Milestone completion statistics
CREATE OR REPLACE VIEW milestone_completion_stats AS
SELECT
    domain,
    milestone_id,
    COUNT(*) as total_attempts,
    COUNT(*) FILTER (WHERE success = TRUE) as successful_completions,
    AVG(quality_score) FILTER (WHERE success = TRUE) as avg_quality_score,
    AVG(completion_time_ms) FILTER (WHERE success = TRUE) as avg_completion_time_ms,
    MIN(completion_time_ms) FILTER (WHERE success = TRUE) as min_completion_time_ms,
    MAX(completion_time_ms) FILTER (WHERE success = TRUE) as max_completion_time_ms
FROM milestone_completions
GROUP BY domain, milestone_id
ORDER BY domain, successful_completions DESC;

-- Learning progress trends (last 30 days)
CREATE OR REPLACE VIEW learning_progress_trends_30d AS
SELECT
    agent_id,
    domain,
    DATE(recorded_at) as date,
    COUNT(*) as total_tasks,
    COUNT(*) FILTER (WHERE success = TRUE) as successful_tasks,
    AVG(quality_score) as avg_quality_score,
    COUNT(DISTINCT skill_level_after) FILTER (WHERE skill_level_after IS NOT NULL) as level_advancements
FROM learning_history
WHERE recorded_at >= NOW() - INTERVAL '30 days'
GROUP BY agent_id, domain, DATE(recorded_at)
ORDER BY agent_id, domain, date DESC;

-- ===========================================
-- LOG MIGRATION
-- ===========================================

INSERT INTO migration_log (version, description, applied_at)
VALUES ('031', 'Create milestone completions and curriculum learning tables (curriculum_profiles, curriculum_paths, milestone_completions, learning_history)', NOW())
ON CONFLICT (version) DO NOTHING;
