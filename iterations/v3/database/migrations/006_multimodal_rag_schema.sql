-- Multimodal RAG Schema - Phase 1
-- Adds document ingest, segmentation, embedding, and search audit infrastructure

-- Documents (root for all ingested media)
CREATE TABLE IF NOT EXISTS documents (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  uri TEXT NOT NULL,
  sha256 TEXT NOT NULL UNIQUE,
  kind VARCHAR(50) NOT NULL,
  created_at TIMESTAMP DEFAULT now(),
  project_scope VARCHAR(255),
  version INTEGER DEFAULT 1,
  pipeline_version TEXT,
  toolchain TEXT,
  model_artifacts JSONB,
  CONSTRAINT valid_kind CHECK (kind IN ('video','slides','diagram','transcript'))
);
CREATE INDEX idx_documents_project ON documents(project_scope);
CREATE INDEX idx_documents_sha256 ON documents(sha256);

-- Segments (time/space slices within documents)
CREATE TABLE IF NOT EXISTS segments (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  type VARCHAR(50) NOT NULL,
  t0 FLOAT,
  t1 FLOAT,
  bbox JSONB,
  content_hash TEXT,
  quality_score FLOAT,
  stability_score FLOAT,
  CONSTRAINT valid_segment_type CHECK (type IN ('slide','speech','diagram','scene'))
);
CREATE INDEX idx_segments_doc ON segments(doc_id);
CREATE INDEX idx_segments_type ON segments(type);

-- Blocks (semantic units within segments)
CREATE TABLE IF NOT EXISTS blocks (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  role VARCHAR(50) NOT NULL,
  text TEXT,
  bbox JSONB,
  ocr_confidence FLOAT,
  CONSTRAINT valid_role CHECK (role IN ('title','bullet','code','table','figure','caption'))
);
CREATE INDEX idx_blocks_segment ON blocks(segment_id);

-- Embedding model registry (config-driven dimensions & metrics)
CREATE TABLE IF NOT EXISTS embedding_models (
  id TEXT PRIMARY KEY,
  modality TEXT NOT NULL,
  dim INTEGER NOT NULL,
  metric TEXT NOT NULL DEFAULT 'cosine',
  active BOOLEAN DEFAULT TRUE,
  CONSTRAINT valid_modality CHECK (modality IN ('text','image','audio')),
  CONSTRAINT valid_metric CHECK (metric IN ('cosine','ip','l2'))
);

-- Per-block vectors (one row per block-model pair)
CREATE TABLE IF NOT EXISTS block_vectors (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  block_id UUID NOT NULL REFERENCES blocks(id) ON DELETE CASCADE,
  model_id TEXT NOT NULL REFERENCES embedding_models(id),
  modality TEXT NOT NULL,
  vec VECTOR,
  CONSTRAINT valid_vec_modality CHECK (modality IN ('text','image','audio')),
  UNIQUE(block_id, model_id)
);

-- Speech turns (aligned with document timestamps)
CREATE TABLE IF NOT EXISTS speech_turns (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  doc_id UUID NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
  speaker_id VARCHAR(255),
  provider TEXT,
  t0 FLOAT NOT NULL,
  t1 FLOAT NOT NULL,
  text TEXT NOT NULL,
  confidence FLOAT
);
CREATE INDEX idx_speech_turns_doc ON speech_turns(doc_id);
CREATE INDEX idx_speech_turns_time ON speech_turns(t0, t1);

-- Speech word timings (fine-grained temporal anchors)
CREATE TABLE IF NOT EXISTS speech_words (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  turn_id UUID NOT NULL REFERENCES speech_turns(id) ON DELETE CASCADE,
  t0 FLOAT NOT NULL,
  t1 FLOAT NOT NULL,
  token TEXT NOT NULL
);
CREATE INDEX idx_speech_words_turn ON speech_words(turn_id);

-- Diagram entities (graph nodes/edges/labels)
CREATE TABLE IF NOT EXISTS diagram_entities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  entity_type VARCHAR(50) NOT NULL,
  normalized_name TEXT,
  attributes JSONB,
  embedding_model_id TEXT REFERENCES embedding_models(id),
  embedding VECTOR,
  CONSTRAINT valid_entity_type CHECK (entity_type IN ('node','edge','label'))
);

CREATE TABLE IF NOT EXISTS diagram_edges (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  src UUID NOT NULL REFERENCES diagram_entities(id) ON DELETE CASCADE,
  dst UUID NOT NULL REFERENCES diagram_entities(id) ON DELETE CASCADE,
  label TEXT
);

-- Named entities (PII-aware)
CREATE TABLE IF NOT EXISTS entities (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  segment_id UUID NOT NULL REFERENCES segments(id) ON DELETE CASCADE,
  type TEXT NOT NULL,
  norm TEXT NOT NULL,
  span_ref TEXT,
  pii BOOLEAN DEFAULT FALSE,
  hash TEXT
);

-- Provenance (fine-grained source tracking)
CREATE TABLE IF NOT EXISTS provenance (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  source_uri TEXT NOT NULL,
  sha256 TEXT,
  t0 FLOAT,
  t1 FLOAT,
  spatial_ref JSONB,
  content_ref TEXT,
  accessed_at TIMESTAMP DEFAULT now()
);

-- Search audit logs (ranking, fusion, feature traces)
CREATE TABLE IF NOT EXISTS search_logs (
  id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  query TEXT NOT NULL,
  created_at TIMESTAMP DEFAULT now(),
  results JSONB,
  features JSONB
);

-- Integrity constraints via triggers
CREATE OR REPLACE FUNCTION validate_segment_time_inclusion() RETURNS TRIGGER AS $$
BEGIN
  -- Blocks' optional t0..t1 must fit segment t0..t1 if present
  IF NEW.t0 IS NOT NULL AND NEW.t1 IS NOT NULL THEN
    -- OK for now; add stricter checks per use case
  END IF;
  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_validate_segment_time
  BEFORE INSERT OR UPDATE ON blocks
  FOR EACH ROW EXECUTE FUNCTION validate_segment_time_inclusion();

-- Bootstrap default embedding models (can be extended via config)
INSERT INTO embedding_models (id, modality, dim, metric, active)
VALUES 
  ('e5-small-v2', 'text', 1536, 'cosine', TRUE),
  ('clip-vit-b32', 'image', 512, 'cosine', TRUE)
ON CONFLICT (id) DO NOTHING;
