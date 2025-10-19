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
DECLARE
  segment_record RECORD;
BEGIN
  -- Get the parent segment
  SELECT * INTO segment_record FROM segments WHERE id = NEW.segment_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'Segment % does not exist', NEW.segment_id;
  END IF;

  -- Validate time bounds if segment has temporal data
  IF segment_record.t0 IS NOT NULL AND segment_record.t1 IS NOT NULL THEN
    -- Block timestamps must be within segment bounds
    IF NEW.t0 IS NOT NULL AND (NEW.t0 < segment_record.t0 OR NEW.t0 > segment_record.t1) THEN
      RAISE EXCEPTION 'Block start time % is outside segment time bounds [%s, %s]',
        NEW.t0, segment_record.t0, segment_record.t1;
    END IF;

    IF NEW.t1 IS NOT NULL AND (NEW.t1 < segment_record.t0 OR NEW.t1 > segment_record.t1) THEN
      RAISE EXCEPTION 'Block end time % is outside segment time bounds [%s, %s]',
        NEW.t1, segment_record.t0, segment_record.t1;
    END IF;

    -- Ensure block time ordering
    IF NEW.t0 IS NOT NULL AND NEW.t1 IS NOT NULL AND NEW.t0 >= NEW.t1 THEN
      RAISE EXCEPTION 'Block start time % must be before end time %', NEW.t0, NEW.t1;
    END IF;
  END IF;

  -- Validate bbox consistency if both segment and block have bbox
  IF segment_record.bbox IS NOT NULL AND NEW.bbox IS NOT NULL THEN
    -- TODO: Implement comprehensive spatial relationship validation for multimodal content
    -- - [ ] Support different geometric containment types (fully contained, overlapping, adjacent)
    -- - [ ] Implement multi-dimensional bbox validation (2D, 3D, temporal)
    -- - [ ] Add spatial relationship types (contains, intersects, touches, within)
    -- - [ ] Support different coordinate systems and transformations
    -- - [ ] Implement spatial indexing and query optimization
    -- - [ ] Add spatial relationship consistency checking across modalities
    -- - [ ] Support spatial constraint validation and error reporting
    IF NOT validate_bbox_containment(segment_record.bbox, NEW.bbox) THEN
      RAISE WARNING 'Block bbox may extend outside segment bbox';
    END IF;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- TODO: Implement comprehensive spatial geometry validation functions
-- - [ ] Support complex geometric shapes beyond rectangles (polygons, circles, irregular shapes)
-- - [ ] Implement proper spatial reference systems and coordinate transformations
-- - [ ] Add geometric operations (intersection, union, difference, buffer)
-- - [ ] Support different dimensionality (2D, 3D, 4D with time)
-- - [ ] Implement spatial indexing and query optimization
-- - [ ] Add geometric validation and topological consistency checks
-- - [ ] Support different geometric data formats (WKT, GeoJSON, PostGIS)
CREATE OR REPLACE FUNCTION validate_bbox_containment(parent_bbox JSONB, child_bbox JSONB)
RETURNS BOOLEAN AS $$
DECLARE
  parent_x1 FLOAT;
  parent_y1 FLOAT;
  parent_x2 FLOAT;
  parent_y2 FLOAT;
  child_x1 FLOAT;
  child_y1 FLOAT;
  child_x2 FLOAT;
  child_y2 FLOAT;
BEGIN
  -- Extract bbox coordinates (assuming [x1,y1,x2,y2] format)
  parent_x1 := (parent_bbox->>0)::FLOAT;
  parent_y1 := (parent_bbox->>1)::FLOAT;
  parent_x2 := (parent_bbox->>2)::FLOAT;
  parent_y2 := (parent_bbox->>3)::FLOAT;

  child_x1 := (child_bbox->>0)::FLOAT;
  child_y1 := (child_bbox->>1)::FLOAT;
  child_x2 := (child_bbox->>2)::FLOAT;
  child_y2 := (child_bbox->>3)::FLOAT;

  -- Check if child bbox is contained within parent bbox
  RETURN child_x1 >= parent_x1 AND child_y1 >= parent_y1 AND
         child_x2 <= parent_x2 AND child_y2 <= parent_y2;
EXCEPTION
  WHEN OTHERS THEN
    -- Return true on parsing errors to avoid blocking inserts
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Validate speech turn temporal ordering
CREATE OR REPLACE FUNCTION validate_speech_turn_timing() RETURNS TRIGGER AS $$
BEGIN
  -- Ensure speech turn has valid time ordering
  IF NEW.t0 >= NEW.t1 THEN
    RAISE EXCEPTION 'Speech turn start time % must be before end time %', NEW.t0, NEW.t1;
  END IF;

  -- Ensure reasonable duration (not too long for a single turn)
  IF NEW.t1 - NEW.t0 > 300.0 THEN -- 5 minutes max
    RAISE WARNING 'Speech turn duration %.1f seconds seems unusually long', NEW.t1 - NEW.t0;
  END IF;

  -- Ensure non-negative timing
  IF NEW.t0 < 0 OR NEW.t1 < 0 THEN
    RAISE EXCEPTION 'Speech turn timing cannot be negative: [%, %]', NEW.t0, NEW.t1;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate vector dimensions match embedding model
CREATE OR REPLACE FUNCTION validate_vector_dimensions() RETURNS TRIGGER AS $$
DECLARE
  expected_dim INTEGER;
BEGIN
  -- Get expected dimension from embedding model
  SELECT dim INTO expected_dim
  FROM embedding_models
  WHERE id = NEW.model_id;

  IF NOT FOUND THEN
    RAISE EXCEPTION 'Embedding model % does not exist', NEW.model_id;
  END IF;

  -- Validate vector dimension matches model expectation
  IF array_length(NEW.vec, 1) != expected_dim THEN
    RAISE EXCEPTION 'Vector dimension % does not match embedding model % dimension %',
      array_length(NEW.vec, 1), NEW.model_id, expected_dim;
  END IF;

  -- Validate modality consistency
  IF NEW.modality NOT IN ('text', 'image', 'audio') THEN
    RAISE EXCEPTION 'Invalid modality %', NEW.modality;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate diagram entity relationships
CREATE OR REPLACE FUNCTION validate_diagram_entity_relationships() RETURNS TRIGGER AS $$
DECLARE
  segment_kind VARCHAR(50);
BEGIN
  -- Ensure diagram entities belong to diagram segments
  SELECT s.type INTO segment_kind
  FROM segments s
  WHERE s.id = NEW.segment_id;

  IF segment_kind != 'diagram' THEN
    RAISE EXCEPTION 'Diagram entities can only belong to diagram segments, not %', segment_kind;
  END IF;

  -- Validate entity type
  IF NEW.entity_type NOT IN ('node', 'edge', 'label') THEN
    RAISE EXCEPTION 'Invalid diagram entity type %', NEW.entity_type;
  END IF;

  -- Validate embedding consistency if provided
  IF NEW.embedding IS NOT NULL AND NEW.embedding_model_id IS NOT NULL THEN
    -- Check that embedding dimension matches model
    DECLARE
      expected_dim INTEGER;
    BEGIN
      SELECT dim INTO expected_dim
      FROM embedding_models
      WHERE id = NEW.embedding_model_id;

      IF array_length(NEW.embedding, 1) != expected_dim THEN
        RAISE EXCEPTION 'Diagram entity embedding dimension % does not match model % dimension %',
          array_length(NEW.embedding, 1), NEW.embedding_model_id, expected_dim;
      END IF;
    END;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Validate document uniqueness constraints
CREATE OR REPLACE FUNCTION validate_document_uniqueness() RETURNS TRIGGER AS $$
DECLARE
  existing_count INTEGER;
BEGIN
  -- Check for duplicate URI + SHA256 combinations
  SELECT COUNT(*) INTO existing_count
  FROM documents
  WHERE uri = NEW.uri AND sha256 = NEW.sha256 AND id != COALESCE(NEW.id, '00000000-0000-0000-0000-000000000000'::UUID);

  IF existing_count > 0 THEN
    RAISE EXCEPTION 'Document with URI % and SHA256 % already exists', NEW.uri, NEW.sha256;
  END IF;

  -- Validate SHA256 format (64 hex characters)
  IF NEW.sha256 !~ '^[a-f0-9]{64}$' THEN
    RAISE EXCEPTION 'Invalid SHA256 format: %', NEW.sha256;
  END IF;

  -- Validate document kind
  IF NEW.kind NOT IN ('video', 'slides', 'diagram', 'transcript') THEN
    RAISE EXCEPTION 'Invalid document kind %', NEW.kind;
  END IF;

  RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_validate_segment_time
  BEFORE INSERT OR UPDATE ON blocks
  FOR EACH ROW EXECUTE FUNCTION validate_segment_time_inclusion();

CREATE TRIGGER trg_validate_speech_turn_timing
  BEFORE INSERT OR UPDATE ON speech_turns
  FOR EACH ROW EXECUTE FUNCTION validate_speech_turn_timing();

CREATE TRIGGER trg_validate_vector_dimensions
  BEFORE INSERT OR UPDATE ON block_vectors
  FOR EACH ROW EXECUTE FUNCTION validate_vector_dimensions();

CREATE TRIGGER trg_validate_diagram_entity_relationships
  BEFORE INSERT OR UPDATE ON diagram_entities
  FOR EACH ROW EXECUTE FUNCTION validate_diagram_entity_relationships();

CREATE TRIGGER trg_validate_document_uniqueness
  BEFORE INSERT OR UPDATE ON documents
  FOR EACH ROW EXECUTE FUNCTION validate_document_uniqueness();

-- Bootstrap default embedding models (can be extended via config)
INSERT INTO embedding_models (id, modality, dim, metric, active)
VALUES 
  ('e5-small-v2', 'text', 1536, 'cosine', TRUE),
  ('clip-vit-b32', 'image', 512, 'cosine', TRUE)
ON CONFLICT (id) DO NOTHING;
