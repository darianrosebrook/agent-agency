# V3 Multimodal RAG System

> Comprehensive multimodal retrieval-augmented generation for Apple Silicon, supporting videos, slides, diagrams, and transcripts with semantic awareness for local development work.

**Status**: Foundation layer complete; placeholder implementations for Phase 2+ enrichers and integrations.

---

## Architecture Overview

```
Local macOS M-series Machine
  ↓
[File Monitor] → [Ingestors] → [Normalizers] → [Enrichers] → [Indexers] → [Embedding Service]
     ↓                                                              ↓
Watch for new/changed media                              Vector DB + Audit Logs
(video, slides, diagrams, captions)                              ↓
                                                        [Council/Claim Extraction]
```

---

## System Components

### 1. Ingestors (iterations/v3/ingestors/src/)

Modality-specific content extraction from local files.

#### VideoIngestor
- **Extension**: `.mp4`, `.mov`, `.avi`, `.mkv`
- **Bridge**: AVAssetReader (Swift) for frame extraction
- **Output**: Segments with keyframes, scene boundaries, audio
- **Config**: `FrameSamplerConfig`, `SceneDetectorConfig`
- **Status**: Placeholder - awaiting Swift bridge integration

```rust
let ingestor = VideoIngestor::new(None, None);
let result = ingestor.ingest(Path::new("demo.mp4"), Some("project-x")).await?;
```

#### SlidesIngestor
- **Extension**: `.pdf`, `.key`
- **Primary**: PDFKit vector text extraction
- **Fallback**: Vision OCR with circuit breaker
- **Output**: Slide pages with text blocks, layout, tables
- **Status**: Placeholder - awaiting PDFKit integration

```rust
let ingestor = SlidesIngestor::new();
let result = ingestor.ingest(Path::new("slides.pdf"), Some("project-x")).await?;
```

#### DiagramsIngestor
- **Extension**: `.svg`, `.graphml`
- **Parsing**: XML → nodes, edges, labels
- **Output**: Graph structure + PNG render for embeddings
- **Status**: Placeholder - awaiting SVG/GraphML parsers

```rust
let ingestor = DiagramsIngestor::new();
let result = ingestor.ingest(Path::new("architecture.svg"), Some("project-x")).await?;
```

#### CaptionsIngestor
- **Extension**: `.srt`, `.vtt`
- **Parsing**: Timestamp-aligned speech turns
- **Output**: SpeechTurn records with word timings
- **Status**: Implemented (SRT/VTT parsing working)

```rust
let ingestor = CaptionsIngestor::new();
let result = ingestor.ingest(Path::new("captions.srt"), Some("project-x")).await?;
```

### 2. File Watcher

Monitors project directory for new/changed media with debouncing and ingestor routing.

```rust
let mut watcher = FileWatcher::new(None);
watcher.watch(Path::new("/path/to/project"), |event| {
    println!("File event: {:?}", event);
}).await?;
```

**Features**:
- Debouncing (1s default)
- Size stability checks (2s default)
- Pattern-based ignore (`.tmp`, `.*`, `.git`)
- Automatic ingestor detection

### 3. Canonical Data Model

All content normalized to:

- **Documents**: Root media file with metadata
- **Segments**: Time/space slices (slide, scene, speech)
- **Blocks**: Semantic units (title, bullet, code, table, figure, caption)
- **SpeechTurns**: ASR/captions with speaker, timings
- **DiagramData**: Graph structure (entities, edges)
- **Provenance**: Fine-grained source tracking

**Database**: `iterations/v3/database/migrations/006_multimodal_rag_schema.sql`

### 4. Embedding Service (Extended)

#### New Types

```rust
pub enum ContentType {
    Text, Code, Documentation, // existing
    VideoFrame, SlideContent, DiagramNode, SpeechTranscript, VisualCaption, // NEW
}

pub struct EmbeddingModel {
    pub id: String,      // 'e5-small-v2', 'clip-vit-b32'
    pub modality: String, // 'text' | 'image' | 'audio'
    pub dim: usize,
    pub metric: String,  // 'cosine' | 'ip' | 'l2'
    pub active: bool,
}

pub struct MultimodalSearchResult {
    pub ref_id: String,
    pub kind: ContentType,
    pub snippet: String,
    pub citation: Option<String>,  // uri + (t0–t1 | bbox)
    pub feature: SearchResultFeature,  // per-index scores + fused
    pub project_scope: Option<String>,
}
```

#### Multimodal Indexer

Stores and searches per-model embeddings:

```rust
let mut indexer = MultimodalIndexer::new();
let mut embeddings = HashMap::new();
embeddings.insert("e5-small-v2".to_string(), vec![...1536 dims...]);
embeddings.insert("clip-vit-b32".to_string(), vec![...512 dims...]);

let indexed = indexer.index_block(
    block_id,
    "slide title text",
    "text",
    embeddings
).await?;
```

**Features**:
- Per-model vector storage (block_vectors table)
- Separate HNSW indices per model + metric
- Support for text, image, audio modalities

### 5. Multimodal Retriever

Late-fusion search with project scoping:

```rust
let retriever = MultimodalRetriever::new(None);
let query = MultimodalQuery {
    text: Some("architecture overview".to_string()),
    query_type: QueryType::Text,
    project_scope: Some("project-x".to_string()),
    max_results: 10,
};

let results = retriever.search(&query).await?;
```

**Features**:
- Route by query type (text, visual, time-anchored, hybrid)
- Reciprocal Rank Fusion (RRF) for score fusion
- Deduplication by content hash
- Project scope filtering (project-first, then global)
- Search audit logging

---

## Integration Points

### Council System

Multimodal context provider supplies bounded context:

```rust
pub struct MultimodalContextProvider {
    retriever: MultimodalRetriever,
}

impl ContextProvider for MultimodalContextProvider {
    async fn gather_context(
        &self,
        topic: &str,
        project_scope: Option<&str>,
    ) -> Result<Vec<ContextBlock>> {
        // Query multimodal RAG for relevant evidence
    }
}
```

### Claim Extraction

Multi-modal evidence collection for verification:

```rust
pub async fn collect_multimodal_evidence(
    claim: &Claim,
    project_scope: Option<&str>,
) -> Result<Vec<Evidence>> {
    // 1. Search for visual evidence (diagrams)
    // 2. Extract relevant speech turns with timestamps
    // 3. Return cross-modal evidence set
}
```

---

## Data Flow Examples

### Example 1: Ingesting a Tech Talk

```
1. File watcher detects: ~/projects/agent-agency/demo.mp4
2. VideoIngestor extracts:
   - Frames at 3 fps
   - Scene boundaries (SSIM+pHash)
   - Audio track → ASR (placeholder)
3. Normalizer creates Segments:
   - slide-1 (0-45s): Title slide
   - slide-2 (45-120s): Architecture diagram
   - ...
4. Enricher (Vision, ASR, entity extraction):
   - OCR title text "Agent Architecture"
   - Extract speaker diarization
   - Detect entities: "PostgreSQL", "Vector DB"
5. Indexer generates embeddings:
   - Text: e5-small-v2 (1536-dim)
   - Visual: clip-vit-b32 (512-dim) for diagram
6. Storage:
   - documents.id = UUID
   - segments[0..n] with blocks
   - block_vectors per model
   - speech_turns with confidence
7. Council queries:
   - "How is data stored?"
   - Retriever finds: diagram slide + speech turn explanation
   - Returns with citation: "demo.mp4#45-120s + bbox"
```

### Example 2: Project-Scoped Query

```
Query: "authentication flow" (project_scope: "project-x")

Retriever search order:
1. Text index (BM25 + e5-small-v2): project-x slides + global docs
2. Visual index (CLIP): project-x diagrams first
3. Graph index: diagram relationships
4. Fuse via RRF
5. Deduplicate by content_hash
6. Return: [{auth-diagram.svg, auth-talk.mp4#2:15, ...}]
```

---

## Database Schema Highlights

Key tables for multimodal RAG:

```sql
documents(id, uri, sha256, kind, project_scope, pipeline_version, toolchain, model_artifacts)
segments(id, doc_id, type, t0, t1, bbox, quality_score, stability_score)
blocks(id, segment_id, role, text, bbox, ocr_confidence)
embedding_models(id, modality, dim, metric, active)
block_vectors(id, block_id, model_id, modality, vec)
speech_turns(id, doc_id, speaker_id, provider, t0, t1, text, confidence)
speech_words(id, turn_id, t0, t1, token)
diagram_entities(id, segment_id, entity_type, normalized_name, embedding)
diagram_edges(id, segment_id, src, dst, label)
entities(id, segment_id, type, norm, pii, hash)
provenance(id, source_uri, sha256, t0, t1, spatial_ref, content_ref)
search_logs(id, query, created_at, results, features)
```

**Integrity**:
- Foreign key cascades (document → segments → blocks)
- Time inclusion trigger (blocks within segments)
- SHA256 deduplication (ignore re-ingests)

---

## Configuration

### Environment Variables

```bash
# Database
DATABASE_URL=postgresql://user:pass@localhost/v3_rag

# Embedding models
EMBEDDING_MODEL_TEXT=e5-small-v2
EMBEDDING_MODEL_VISUAL=clip-vit-b32
EMBEDDING_DIMS_TEXT=1536
EMBEDDING_DIMS_VISUAL=512

# File watcher
FILE_WATCHER_DEBOUNCE_MS=1000
FILE_WATCHER_SIZE_STABILITY_MS=2000

# Project scope (optional)
PROJECT_SCOPE=project-x
```

### Rust Config (Ingestors)

```rust
use ingestors::{
    VideoIngestor, FrameSamplerConfig, SceneDetectorConfig,
    FileWatcher, FileWatcherConfig,
};

let video_config = Some(FrameSamplerConfig {
    fps_target: 3.0,
    quality_threshold: 0.5,
    window_ms: 500,
});

let watcher_config = Some(FileWatcherConfig {
    debounce_ms: 1000,
    size_stability_check_ms: 2000,
    ignore_patterns: vec![...],
});
```

---

## Implementation Status & TODOs

### Completed (Phase 1)

- [x] Database schema with multimodal tables
- [x] Embedding service types (ContentType, EmbeddingModel, MultimodalSearchResult)
- [x] Ingestor framework with trait-based architecture
- [x] CaptionsIngestor (SRT/VTT parsing working)
- [x] File watcher foundation
- [x] Multimodal indexer structure
- [x] Multimodal retriever with RRF fusion
- [x] Integration hooks for council/claim extraction

### Pending (Phase 2-3)

- TODO: PLACEHOLDER - Integrate AVAssetReader (Swift bridge) for video ingestion
- TODO: PLACEHOLDER - Integrate PDFKit for PDF text extraction
- TODO: PLACEHOLDER - SVG/GraphML parsing and graph extraction
- TODO: PLACEHOLDER - Vision Framework bridge (RecognizeDocumentsRequest)
- TODO: PLACEHOLDER - WhisperX + pyannote for ASR/diarization
- TODO: PLACEHOLDER - Entity extraction (DataDetection + NER)
- TODO: PLACEHOLDER - Visual captioning (BLIP)
- TODO: PLACEHOLDER - BM25 full-text search indexing
- TODO: PLACEHOLDER - CLIP embedding generation
- TODO: PLACEHOLDER - Database persistence for block_vectors, search_logs
- TODO: PLACEHOLDER - Council/claim integration
- TODO: PLACEHOLDER - Project scope filtering enforcement
- TODO: PLACEHOLDER - Search audit logging
- TODO: PLACEHOLDER - Job scheduling with concurrency caps

### Success Metrics

- Ingest: < 2 min per video hour
- Search: P95 < 500 ms (warm cache)
- Coverage: All documented media types ingested
- Quality: Per-stage quality/confidence scores tracked
- Reliability: Circuit breakers, no memory leaks, idempotent ingest by SHA256

---

## Quick Start

```bash
cd iterations/v3

# Build ingestors
cd ingestors && cargo build && cd ..

# Run tests
cargo test --package ingestors

# CLI example (placeholder)
./target/debug/ingestors

# Database setup
cargo run --bin setup_db

# File watcher (placeholder)
# cargo run --bin file_watcher -- /path/to/project
```

---

## References

- **Database**: `iterations/v3/database/migrations/006_multimodal_rag_schema.sql`
- **Ingestors**: `iterations/v3/ingestors/src/`
- **Embedding Service**: `iterations/v3/embedding-service/src/multimodal_indexer.rs`
- **Retriever**: `iterations/v3/research/src/multimodal_retriever.rs`
- **Plan**: `/v.plan.md` (guardrails & implementation notes)

---

**Author**: @darianrosebrook  
**Date**: October 2025  
**Status**: Foundation implementation complete; Phase 2 enrichers pending Swift/Python bridges
