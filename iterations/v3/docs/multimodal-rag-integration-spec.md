# Multimodal RAG Integration Specification
## V3 System Integration Plan

**Author:** @darianrosebrook  
**Status:** Integration Planning  
**Date:** October 19, 2025

---

## 1. Integration Overview

The multimodal RAG system consists of 6 independent crates that must be integrated into the existing V3 module ecosystem:

- **ingestors** - Ingestion layer (video, slides, diagrams, captions)
- **enrichers** - Enrichment layer (Vision, Speech, ASR, captioning)
- **indexers** - Indexing layer (BM25, HNSW, job scheduling)
- **embedding-service** - Vector management and embedding
- **research** - Retrieval and context provision
- **apple-silicon** - Native framework bridges (Vision, Speech)

---

## 2. V3 Module Integration Points

### 2.1 Council Module Integration

**Current State:** Council makes decisions based on evidence manifests
**Integration Point:** `council/src/lib.rs`

**Changes Required:**
```rust
// Import multimodal providers
use research::MultimodalContextProvider;
use council::MultimodalEvidenceEnricher;

// In Council decision flow:
// 1. When making a decision, query MultimodalContextProvider
let mm_context = multimodal_provider
    .get_decision_context("decision_point", project_scope)
    .await?;

// 2. Enrich claims with multimodal evidence
for claim in claims {
    let enriched = evidence_enricher
        .enrich_claim_with_multimodal_evidence(
            &claim.id,
            &claim.statement,
            None,
        )
        .await?;
    
    // 3. Use enriched evidence in verdict generation
    verdict.multimodal_evidence = enriched.multimodal_evidence;
}
```

**Priority:** HIGH - Direct impact on Council reasoning

### 2.2 Research Module Integration

**Current State:** Research handles knowledge seeking
**Integration Point:** `research/src/lib.rs`

**Changes Required:**
```rust
// Export multimodal components
pub use research::multimodal_context_provider::MultimodalContextProvider;
pub use research::multimodal_retriever::MultimodalRetriever;

// Integrate into KnowledgeSeeker
impl KnowledgeSeeker {
    pub async fn seek_multimodal_knowledge(
        &self,
        query: &str,
        context: &SearchContext,
    ) -> Result<MultimodalContext> {
        let retriever = MultimodalRetriever::new(self.db_pool.clone());
        retriever.search_multimodal(query, 10, context.project_scope).await
    }
}
```

**Priority:** HIGH - Core retrieval functionality

### 2.3 Orchestration Module Integration

**Current State:** Orchestration coordinates agent workflows
**Integration Point:** `orchestration/src/lib.rs`

**Changes Required:**
```rust
// Import ingestors and enrichers
use ingestors::{FileWatcher, VideoIngestor, SlidesIngestor};
use enrichers::{VisionEnricher, AsrEnricher, CircuitBreaker};

// In TaskOrchestrator:
pub async fn orchestrate_document_processing(
    &self,
    file_path: &Path,
) -> Result<ProcessingResult> {
    // 1. File watching for changes
    let watcher = FileWatcher::new();
    watcher.watch(file_path).await?;
    
    // 2. Ingest based on file type
    let blocks = match file_type {
        FileType::Video => self.ingest_video(file_path).await?,
        FileType::Slides => self.ingest_slides(file_path).await?,
        _ => vec![],
    };
    
    // 3. Enrich blocks
    let enriched = self.enrich_blocks(&blocks).await?;
    
    // 4. Index enriched content
    self.index_blocks(&enriched).await?;
}
```

**Priority:** MEDIUM - Workflow coordination

### 2.4 Database Module Integration

**Current State:** Database handles schema and persistence
**Integration Point:** `database/src/lib.rs`

**Changes Required:**
```rust
// Import indexers for persistence
use indexers::database::PostgresVectorStore;
use indexers::VectorStore;

// In DatabasePool:
pub async fn store_vector(&self, record: BlockVectorRecord) -> Result<()> {
    let store = PostgresVectorStore::new(self.pool.clone());
    store.store_vector(record).await
}

pub async fn search_vectors(
    &self,
    query: &[f32],
    model_id: &str,
    k: usize,
) -> Result<Vec<(Uuid, f32)>> {
    let store = PostgresVectorStore::new(self.pool.clone());
    store.search_similar(query, model_id, k, None).await
}

// Add vector storage tables (if not already present)
// See: database/migrations/007_pgvector_setup.sql
```

**Priority:** MEDIUM - Data persistence

### 2.5 Observer Module Integration

**Current State:** Observer provides observability and metrics
**Integration Point:** `observability/src/lib.rs`

**Changes Required:**
```rust
// Import performance metrics
use integration_tests::PerformanceMetrics;

// In MetricsCollector:
pub async fn collect_multimodal_metrics(
    &self,
    test_name: &str,
) -> Result<PerformanceMetrics> {
    // Collect ingestion metrics
    let ingest_time = self.get_metric("mm_ingest_time_ms")?;
    let enrich_time = self.get_metric("mm_enrich_time_ms")?;
    let index_time = self.get_metric("mm_index_time_ms")?;
    let retrieve_time = self.get_metric("mm_retrieve_time_ms")?;
    
    // Report via observability system
    self.report_performance(PerformanceMetrics {
        test_name: test_name.to_string(),
        ingest_time_ms: ingest_time,
        enrich_time_ms: enrich_time,
        index_time_ms: index_time,
        retrieve_time_ms: retrieve_time,
        // ... other metrics
    }).await
}
```

**Priority:** LOW - Observability enhancement

### 2.6 Workers Module Integration

**Current State:** Workers execute background tasks
**Integration Point:** `workers/src/lib.rs`

**Changes Required:**
```rust
// Import job scheduler
use indexers::JobScheduler;

// In WorkerPool:
pub async fn schedule_multimodal_work(
    &self,
    work_item: MultimodalWorkItem,
) -> Result<()> {
    let scheduler = JobScheduler::new();
    
    match work_item {
        MultimodalWorkItem::IndexBlock(block) => {
            scheduler.schedule_embedding_job(block).await?
        }
        MultimodalWorkItem::ProcessAudio(audio) => {
            scheduler.schedule_asr_job(audio).await?
        }
        MultimodalWorkItem::ProcessImage(image) => {
            scheduler.schedule_vision_job(image).await?
        }
    }
}
```

**Priority:** MEDIUM - Background processing

---

## 3. Implementation Sequence

### Phase 1: Foundation (Week 1)
1. **Database Integration**
   - Add multimodal tables (already done in migration 006/007)
   - Connect indexers to DatabasePool
   - Verify vector storage works

2. **Research Integration**
   - Wire MultimodalRetriever into KnowledgeSeeker
   - Export MultimodalContextProvider
   - Test retrieval pipeline

### Phase 2: Core Workflow (Week 2)
1. **Orchestration Integration**
   - Add ingestor coordination
   - Add enricher pipeline
   - Add indexer coordination

2. **Council Integration**
   - Wire MultimodalContextProvider to Council judges
   - Integrate MultimodalEvidenceEnricher with ClaimExtractor
   - Test decision-making with multimodal context

### Phase 3: Operations (Week 3)
1. **Worker Integration**
   - Schedule multimodal jobs
   - Implement backpressure handling
   - Test concurrent processing

2. **Observer Integration**
   - Collect multimodal metrics
   - Report to observability system
   - Monitor SLA compliance

### Phase 4: Validation (Week 4)
1. **End-to-end testing**
2. **Performance verification**
3. **Integration testing**

---

## 4. Configuration Requirements

### 4.1 Connection Strings

```toml
# Cargo.toml dependencies needed in each module

[dependencies]
ingestors = { path = "../ingestors" }
enrichers = { path = "../enrichers" }
indexers = { path = "../indexers" }
research = { path = "../research" }
apple-silicon = { path = "../apple-silicon" }
```

### 4.2 Environment Variables

```bash
# Database configuration
DATABASE_URL=postgresql://user:pass@localhost:5432/agent_agency_v3

# Vector configuration
VECTOR_STORE_BACKEND=pgvector
EMBEDDING_MODEL=e5-small-v2

# Enrichment configuration
VISION_FRAMEWORK_ENABLED=true
SPEECH_FRAMEWORK_ENABLED=true
WHISPERX_ENABLED=true
BLIP_ENABLED=true

# Performance configuration
MAX_CONCURRENT_INGESTIONS=2
MAX_CONCURRENT_ENRICHMENTS=4
MAX_CONCURRENT_INDEXING=2
```

### 4.3 Feature Flags

```toml
# In Cargo.toml of modules that use multimodal RAG

[features]
default = ["multimodal"]
multimodal = ["ingestors", "enrichers", "indexers", "research"]
```

---

## 5. Integration Test Plan

### 5.1 Unit Integration Tests

```rust
#[tokio::test]
async fn test_council_with_multimodal_context() {
    // 1. Setup Council with MultimodalContextProvider
    // 2. Make a decision requiring context
    // 3. Verify multimodal evidence was retrieved and used
    // 4. Assert decision quality improved with multimodal data
}

#[tokio::test]
async fn test_orchestration_full_pipeline() {
    // 1. Create test document (video/slides)
    // 2. Run through ingest → enrich → index
    // 3. Query for retrieval
    // 4. Use in Council decision
}
```

### 5.2 Performance Verification

```rust
#[tokio::test]
async fn test_sla_compliance() {
    let e2e_test = MultimodalRagE2eTests::new("integration_test");
    let metrics = e2e_test.test_complete_workflow().await?;
    
    metrics.verify_slas()?; // Must pass SLA verification
}
```

---

## 6. Risk Mitigation

### 6.1 Database Migration

**Risk:** Migration 007 might conflict with existing schema
**Mitigation:** 
- Run migration in test environment first
- Backup database before migration
- Provide rollback script

### 6.2 Performance Impact

**Risk:** Multimodal context retrieval might slow down Council decisions
**Mitigation:**
- Implement context caching
- Use async/await for non-blocking retrieval
- Budget enforcement prevents excessive context

### 6.3 Dependency Conflicts

**Risk:** New crate dependencies might conflict
**Mitigation:**
- Use feature flags for optional dependencies
- Run dependency resolution checks
- Use workspace features to manage versions

---

## 7. Success Criteria

- [  ] All modules compile without errors
- [  ] Integration tests pass
- [  ] Performance SLAs met
- [  ] Database migration successful
- [  ] Council decisions improved with multimodal context
- [  ] Full end-to-end workflow functional
- [  ] Documentation updated
- [  ] Team trained on new system

---

## 8. Integration Checklist

### Pre-Integration
- [ ] All multimodal RAG modules production-ready
- [ ] Database migrations prepared
- [ ] Environment variables documented
- [ ] Dependencies resolved

### Integration Phase
- [ ] Database integration done
- [ ] Research module integration done
- [ ] Council integration done
- [ ] Orchestration integration done
- [ ] Worker integration done
- [ ] Observer integration done

### Post-Integration
- [ ] All tests passing
- [ ] SLAs verified
- [ ] Performance benchmarked
- [ ] Documentation complete
- [ ] Team briefing completed

---

## 9. Timeline

| Phase | Weeks | Status |
|-------|-------|--------|
| Planning & Setup | 1 | Starting Now |
| Database & Research | 1-2 | Pending |
| Council & Orchestration | 2-3 | Pending |
| Workers & Observer | 3-4 | Pending |
| Validation & Deployment | 4-5 | Pending |

---

## 10. Next Steps

1. Review this specification with team
2. Prepare database migration environment
3. Start Phase 1 implementation (Database + Research)
4. Run integration tests after each phase
5. Verify SLA compliance throughout

---

**End of Integration Specification**
