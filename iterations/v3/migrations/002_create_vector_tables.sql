-- Create vector storage tables for multimodal RAG
-- Migration: 002_create_vector_tables.sql

-- Create block_vectors table for storing vector embeddings
CREATE TABLE IF NOT EXISTS block_vectors (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    block_id UUID NOT NULL,
    content TEXT NOT NULL,
    modality VARCHAR(50) NOT NULL, -- 'text', 'image', 'audio', 'video', 'diagram'
    embedding_model_id VARCHAR(100) NOT NULL,
    embedding VECTOR(384), -- Default dimension for e5-small-v2
    metadata JSONB DEFAULT '{}',
    project_scope VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Indexes for performance
    CONSTRAINT valid_modality CHECK (modality IN ('text', 'image', 'audio', 'video', 'diagram'))
);

-- Create HNSW index for vector similarity search
CREATE INDEX IF NOT EXISTS idx_block_vectors_embedding_hnsw 
ON block_vectors 
USING hnsw (embedding vector_cosine_ops)
WITH (m = 16, ef_construction = 64);

-- Create additional indexes for filtering
CREATE INDEX IF NOT EXISTS idx_block_vectors_modality ON block_vectors (modality);
CREATE INDEX IF NOT EXISTS idx_block_vectors_model_id ON block_vectors (embedding_model_id);
CREATE INDEX IF NOT EXISTS idx_block_vectors_project_scope ON block_vectors (project_scope);
CREATE INDEX IF NOT EXISTS idx_block_vectors_created_at ON block_vectors (created_at);

-- Create search_audit_log table for tracking search operations
CREATE TABLE IF NOT EXISTS search_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    query TEXT NOT NULL,
    results JSONB NOT NULL DEFAULT '[]',
    features JSONB DEFAULT '{}',
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    user_id UUID,
    session_id VARCHAR(100),
    response_time_ms INTEGER,
    result_count INTEGER DEFAULT 0
);

-- Create index for audit log queries
CREATE INDEX IF NOT EXISTS idx_search_audit_timestamp ON search_audit_log (timestamp);
CREATE INDEX IF NOT EXISTS idx_search_audit_user_id ON search_audit_log (user_id);

-- Create vector_store_stats table for monitoring
CREATE TABLE IF NOT EXISTS vector_store_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    total_vectors INTEGER DEFAULT 0,
    vectors_by_modality JSONB DEFAULT '{}',
    vectors_by_model JSONB DEFAULT '{}',
    avg_embedding_dimension INTEGER DEFAULT 384,
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Create function to update vector store statistics
CREATE OR REPLACE FUNCTION update_vector_store_stats()
RETURNS VOID AS $$
BEGIN
    INSERT INTO vector_store_stats (
        total_vectors,
        vectors_by_modality,
        vectors_by_model,
        avg_embedding_dimension,
        last_updated
    )
    SELECT 
        COUNT(*) as total_vectors,
        jsonb_object_agg(modality, count) as vectors_by_modality,
        jsonb_object_agg(embedding_model_id, count) as vectors_by_model,
        384 as avg_embedding_dimension, -- Default for e5-small-v2
        NOW() as last_updated
    FROM (
        SELECT 
            modality,
            embedding_model_id,
            COUNT(*) as count
        FROM block_vectors
        GROUP BY modality, embedding_model_id
    ) stats
    ON CONFLICT (id) DO UPDATE SET
        total_vectors = EXCLUDED.total_vectors,
        vectors_by_modality = EXCLUDED.vectors_by_modality,
        vectors_by_model = EXCLUDED.vectors_by_model,
        last_updated = EXCLUDED.last_updated;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to update stats when vectors are inserted/updated/deleted
CREATE OR REPLACE FUNCTION trigger_update_vector_stats()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM update_vector_store_stats();
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_update_vector_stats
    AFTER INSERT OR UPDATE OR DELETE ON block_vectors
    FOR EACH STATEMENT
    EXECUTE FUNCTION trigger_update_vector_stats();

-- Log the migration
INSERT INTO migration_log (version, description, applied_at) 
VALUES ('002', 'Create vector storage tables', NOW())
ON CONFLICT (version) DO NOTHING;
