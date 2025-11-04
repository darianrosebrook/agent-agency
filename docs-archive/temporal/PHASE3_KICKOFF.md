# Phase 3 Kickoff: V3 Multimodal RAG Bridge Implementations

**Status**: Ready to Implement  
**Date**: October 18, 2025  
**Estimated Duration**: 5 weeks (Oct 25 - Nov 8, 2025)  
**Owner**: Assigned Phase 3 Developer

---

## Overview

Phase 3 focuses on **bridge implementations** - connecting the production-ready foundation (Phases 1-2) to external systems:

1. **PostgreSQL pgvector** - Vector database queries
2. **Swift Bridges** - Apple-native framework integration
3. **Python Bridges** - External ML model integration
4. **System Integration** - Council and Claim Extraction wiring

All components have placeholder TODOs clearly marked. This document provides concrete starting points.

---

## Phase 3 Week-by-Week Breakdown

### **Week 1: PostgreSQL pgvector Layer**

**Goal**: Enable vector similarity search through database

**Files to Modify**:
- `iterations/v3/indexers/src/database.rs` (3 methods)
- `iterations/v3/database/migrations/006_multimodal_rag_schema.sql` (indices)

**Tasks**:

1. **Enable pgvector Extension**
   ```sql
   CREATE EXTENSION IF NOT EXISTS vector;
   ```
   - Run on target PostgreSQL database
   - Verify: `SELECT * FROM pg_extension WHERE extname = 'vector';`

2. **Implement VectorStore::store_vector()**
   ```rust
   // File: indexers/src/database.rs
   async fn store_vector(&self, record: BlockVectorRecord) -> Result<()> {
       sqlx::query(
           r#"
           INSERT INTO block_vectors (block_id, model_id, modality, vec)
           VALUES ($1, $2, $3, $4::vector)
           ON CONFLICT (block_id, model_id) DO UPDATE
           SET vec = EXCLUDED.vec
           "#
       )
       .bind(record.block_id)
       .bind(&record.model_id)
       .bind(&record.modality)
       .bind(&record.vector)
       .execute(&self.pool)
       .await?;
       Ok(())
   }
   ```

3. **Implement VectorStore::search_similar()**
   ```rust
   // Choose operator based on metric:
   // - Cosine similarity: <=>
   // - Inner product: <#>
   // - L2 distance: <->
   
   // Create HNSW indices in migration:
   CREATE INDEX idx_block_vectors_e5_hnsw
     ON block_vectors USING hnsw (vec vector_cosine_ops)
     WHERE model_id = 'e5-small-v2';
   ```

4. **Implement VectorStore::log_search()**
   ```rust
   // INSERT into search_logs with query, results, features, timestamp
   ```

**Tests to Run**:
```bash
cd iterations/v3
cargo test --lib indexers::database --features postgres
```

**Acceptance Criteria**:
- Can INSERT vector without error
- Can search_similar and get ranked results
- Audit log contains correct entries
- All 3 methods have passing tests

---

### **Week 2: Swift Bridges (Vision Framework & Apple Speech)**

**Goal**: OCR and speech recognition via native frameworks

**New Files to Create**:
- `iterations/v3/apple-silicon/src/vision_bridge.rs`
- `iterations/v3/apple-silicon/src/speech_bridge.rs`

**Task 2.1: Vision Framework Bridge**

Reference: `/MULTIMODAL_RAG_PHASE3_PLAN.md` section "Priority 2: Swift Bridges"

```rust
// File: apple-silicon/src/vision_bridge.rs

use anyhow::Result;

#[link(name = "Foundation", kind = "framework")]
#[link(name = "Vision", kind = "framework")]
extern "C" {
    fn analyze_document_request(
        image_bytes: *const u8,
        image_len: usize,
        timeout_ms: u64,
    ) -> *mut c_char;
}

pub struct VisionBridge;

impl VisionBridge {
    pub async fn analyze_document(
        image_data: &[u8],
        timeout_ms: u64,
    ) -> Result<VisionAnalysisResult> {
        unsafe {
            let result_ptr = analyze_document_request(
                image_data.as_ptr(),
                image_data.len(),
                timeout_ms,
            );
            
            if result_ptr.is_null() {
                return Err(anyhow::anyhow!("Vision analysis failed"));
            }
            
            let result_str = CStr::from_ptr(result_ptr)
                .to_string_lossy()
                .to_string();
            
            libc::free(result_ptr as *mut libc::c_void);
            
            let analysis: VisionAnalysisResult = serde_json::from_str(&result_str)?;
            Ok(analysis)
        }
    }
}
```

**Task 2.2: Apple Speech Framework Bridge**

Similar pattern to Vision bridge. See Phase 3 Plan for code template.

**Integration Points**:
- `VisionEnricher::analyze_document()` calls `VisionBridge::analyze_document()`
- `AsrEnricher::transcribe_apple()` calls `SpeechBridge::transcribe()`

**Tests**:
```bash
# Xcode unit tests for Swift side
# Rust integration tests for FFI correctness
```

**Acceptance Criteria**:
- Vision bridge returns OCR results
- Speech bridge returns transcription with timings
- Memory properly managed (@autoreleasepool)
- Circuit breaker protects against timeout

---

### **Week 3: Python Bridges (WhisperX, BLIP, pyannote)**

**Goal**: ML model inference via Python subprocesses

**New File to Create**:
- `iterations/v3/enrichers/src/python_bridge.rs`

**Task 3.1: WhisperX Integration**

```rust
// File: enrichers/src/python_bridge.rs

pub async fn transcribe_with_whisperx(
    audio_data: &[u8],
    language: Option<&str>,
) -> Result<AsrResult> {
    // 1. Write audio to temp file
    let temp_path = std::env::temp_dir()
        .join(format!("audio_{}.wav", uuid::Uuid::new_v4()));
    std::fs::write(&temp_path, audio_data)?;
    
    // 2. Call WhisperX subprocess
    let lang_arg = language.unwrap_or("en");
    let output = Command::new("whisperx")
        .arg(temp_path.to_str().unwrap())
        .arg("--language").arg(lang_arg)
        .arg("--diarize_model").arg("pyannote")
        .arg("--output_format").arg("json")
        .output()?;
    
    if !output.status.success() {
        return Err(anyhow::anyhow!("WhisperX failed"));
    }
    
    // 3. Parse JSON output
    let json: WhisperXOutput = serde_json::from_slice(&output.stdout)?;
    
    // 4. Convert to AsrResult with word timings
    Ok(convert_whisperx_to_asr_result(json))
}
```

**Task 3.2: BLIP Visual Captioning**

Similar pattern - subprocess call to Python script with image input.

**Task 3.3: Error Handling**

- Circuit breaker for subprocess timeouts
- Job scheduler backpressure
- Graceful fallback if model unavailable

**Tests**:
```bash
# Integration tests with real subprocess calls
cargo test --test '*enricher*' --features python-bridges
```

**Acceptance Criteria**:
- WhisperX subprocess runs successfully
- Output parsed and converted to AsrResult
- Word timings extracted correctly
- BLIP captions generated with tags
- Circuit breaker prevents cascade failures

---

### **Weeks 4-5: System Integration & End-to-End Testing**

**Goal**: Wire everything together and validate full pipeline

**Task 4.1: Council Integration**

File to Create: `iterations/v3/council/src/multimodal_provider.rs`

```rust
pub struct MultimodalContextProvider {
    retriever: Arc<MultimodalRetriever>,
    scheduler: Arc<JobScheduler>,
}

#[async_trait]
impl ContextProvider for MultimodalContextProvider {
    async fn gather_context(
        &self,
        topic: &str,
        budget: ContextBudget,
    ) -> Result<Vec<ContextBlock>> {
        // 1. Check job scheduler
        if !self.scheduler.try_acquire(JobType::Embedding)? {
            return Err(anyhow::anyhow!("Embedding queue full"));
        }
        
        // 2. Search across modalities
        let query = MultimodalQuery {
            text: Some(topic.to_string()),
            query_type: QueryType::Hybrid,
            project_scope: None,
            max_results: budget.k,
        };
        
        let results = self.retriever.search(&query).await?;
        
        // 3. Deduplicate and respect token budget
        let blocks: Vec<ContextBlock> = results
            .into_iter()
            .take_while(|_| { /* token budgeting */ true })
            .map(|r| ContextBlock {
                text: r.snippet,
                confidence: r.feature.fused_score,
                citation: r.citation,
                modality: r.kind,
            })
            .collect();
        
        self.scheduler.release(JobType::Embedding, true);
        Ok(blocks)
    }
}
```

**Task 4.2: Claim Extraction Integration**

File to Create: `iterations/v3/claim-extraction/src/multimodal_evidence.rs`

```rust
pub struct MultimodalEvidenceCollector {
    retriever: Arc<MultimodalRetriever>,
}

impl MultimodalEvidenceCollector {
    pub async fn collect_evidence_for_claim(
        &self,
        claim: &Claim,
        project_scope: Option<&str>,
    ) -> Result<Vec<Evidence>> {
        // Search for visual evidence (diagrams, figures)
        // Search for speech evidence (with timestamps)
        // Search for text evidence (documents)
        // Return deduplicated, ranked evidence
    }
}
```

**Task 4.3: End-to-End Test**

File to Create: `iterations/v3/integration-tests/tests/multimodal_rag_e2e.rs`

```rust
#[tokio::test]
async fn test_multimodal_rag_full_pipeline() -> Result<()> {
    // 1. Setup
    let db = DatabasePool::new("postgresql://...", 5).await?;
    let scheduler = JobScheduler::new(50);
    let retriever = MultimodalRetriever::new(Some(Arc::new(db.clone())));
    
    // 2. Ingest test media
    let test_video = std::fs::read("tests/fixtures/sample_talk.mp4")?;
    let ingest_result = VideoIngestor::new(None, None)
        .ingest(Path::new("tests/fixtures/sample_talk.mp4"), Some("test-project"))
        .await?;
    
    // 3. Enrich
    let vision_enricher = VisionEnricher::new(EnricherConfig::default());
    let ocr_result = vision_enricher
        .analyze_document(&ingest_result.segments[0].blocks[0].raw_bytes.unwrap(), None)
        .await?;
    
    // 4. Index
    let mut indexer = MultimodalIndexer::new();
    for segment in &ingest_result.segments {
        for block in &segment.blocks {
            let embeddings = generate_embeddings(&block.text).await?;
            indexer.index_block(block.id, &block.text, "text", embeddings).await?;
        }
    }
    
    // 5. Retrieve
    let query = MultimodalQuery {
        text: Some("What was discussed about machine learning?".to_string()),
        query_type: QueryType::Hybrid,
        project_scope: Some("test-project".to_string()),
        max_results: 5,
    };
    
    let results = retriever.search(&query).await?;
    assert!(!results.is_empty());
    assert!(results[0].feature.fused_score > 0.5);
    
    // 6. Verify Council integration
    let provider = MultimodalContextProvider::new(retriever, scheduler);
    let context = provider.gather_context("machine learning", ContextBudget::default()).await?;
    assert!(!context.is_empty());
    
    Ok(())
}
```

**Task 4.4: Performance Tuning**

- Measure P99 latency for each component
- Optimize query plans
- Tune concurrency caps based on load testing
- Profile memory usage

**Acceptance Criteria**:
- Full end-to-end pipeline works
- Council integration wired
- Claim extraction collects evidence
- P99 retrieval latency < 500ms
- All integration tests pass
- No memory leaks under sustained load

---

## Implementation Best Practices

### Database Layer (Week 1)
- Start with `pgvector` extension installation
- Test each method independently before integration
- Use transactions for data consistency
- Monitor query performance with EXPLAIN ANALYZE

### Swift Bridges (Week 2)
- Use @autoreleasepool for memory management
- Proper error handling with NSError conversion
- Test with real framework calls, not mocks
- Handle timeout gracefully

### Python Bridges (Week 3)
- Use virtualenv for isolated Python environment
- Pin all ML model versions in requirements.txt
- Handle subprocess output encoding properly
- Use circuit breaker for model unavailability

### System Integration (Weeks 4-5)
- Test Council integration with mock queries first
- Validate claim evidence with known claims
- Use fixture data for reproducible tests
- Run load tests before declaring ready

---

## Troubleshooting Guide

### Issue: pgvector queries timing out
**Fix**: Ensure HNSW indices created with correct parameters, check PostgreSQL configuration

### Issue: Swift FFI crashes
**Fix**: Verify memory management (@autoreleasepool), check C-ABI signatures

### Issue: Python subprocess hangs
**Fix**: Verify Python environment, check subprocess timeout configuration

### Issue: Council integration not wired
**Fix**: Verify provider interface matches ContextProvider trait, check injection

---

## Definition of Done

Phase 3 is complete when:

- PostgreSQL pgvector queries tested and working
- Swift Vision and Speech bridges functional
- Python WhisperX and BLIP bridges functional
- Council integration wired and tested
- Claim extraction evidence collection working
- End-to-end test passing (file → retrieve → council)
- Performance meets P99 < 500ms target
- All code reviewed and documented
- No breaking changes to Phases 1-2

---

## Commit Strategy

Week-by-week commits with clear messages:

```
Week 1: feat: implement PostgreSQL pgvector vector storage
Week 2: feat: add Swift Vision and Speech framework bridges
Week 3: feat: integrate WhisperX and BLIP Python models
Week 4: feat: wire multimodal context provider to council
Week 5: feat: extend claim extraction with multimodal evidence
```

No `--no-verify` allowed. All commits must pass linting and tests.

---

## Success Metrics

| Metric | Target | Acceptance |
|--------|--------|-----------|
| P99 Retrieval Latency | <500ms | Required |
| Test Coverage | 80%+ | Required |
| Circuit Breaker Trip Rate | <1%/24h | Required |
| Memory Leak Test | 0 leaks | Required |
| Documentation | 100% | Required |

---

## Next Action

**Immediate**: Review this document and `/MULTIMODAL_RAG_PHASE3_PLAN.md`  
**This Week**: Set up PostgreSQL pgvector and begin database layer implementation  
**Timeline**: 5 weeks (Oct 25 - Nov 8, 2025)

---

**Phase 3 Owner**: [Assigned Developer]  
**Kickoff Date**: October 25, 2025  
**Target Completion**: November 8, 2025  
**Status**: Ready to Implement
