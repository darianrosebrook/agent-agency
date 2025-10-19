# Remaining TODOs Implementation Guide

**Author:** @darianrosebrook  
**Last Updated:** October 19, 2025

This document provides detailed guidance for implementing the 230 remaining TODOs across the codebase, organized by priority and architectural domain.

---

## 1. Critical Security & Infrastructure (5 items)

### 1.1 ANE Device Initialization (`apple-silicon/src/ane.rs`)
**Status:** ❌ Blocked by Objective-C bindings  
**Complexity:** VERY HIGH  
**Impact:** Core performance optimization for Apple Silicon

#### Requirements:
1. **Objective-C Class Binding** (lines 449-470)
   - Fix `StrongPtr<objc::runtime::Object>` type issues
   - Use `objc2` crate or update `objc` bindings
   - Implement proper Message trait for ANE device
   
2. **Thread Safety** (lines 462-494)
   - Wrap `*mut objc::runtime::Object` in thread-safe container
   - Implement Send + Sync traits
   - Use `parking_lot::Mutex` for shared state
   
3. **Dispatch Queue** (lines 497-528)
   - Replace commented `dispatch::Queue` usage
   - Use `dispatch-rs` crate for async operations
   - Implement proper QoS (Quality of Service) settings
   
4. **Memory Management** (lines 531-539)
   - Configure ANE memory regions
   - Implement command queue initialization
   - Add cleanup handlers

#### Suggested Implementation Path:
```rust
// Use objc2 crate for safer bindings
use objc2::class;
use objc2::{msg_send, msg_send_id};

// Wrap in Arc<Mutex> for thread safety
let ane_device = Arc::new(Mutex::new(unsafe {
    let cls = class!(ANEDevice);
    msg_send_id![cls, new]
}));
```

---

### 1.2 Council Debate Tie-Breaking (`council/src/coordinator.rs`)
**Status:** ❌ Requires debate resolution logic  
**Complexity:** MEDIUM  
**Impact:** Arbitration system accuracy

#### Requirements (lines 395-442):
1. **Debate Resolution (4 TODOs)**
   - Implement CAWS rule-based scoring
   - Apply override policies for special cases
   - Generate resolution rationale
   - Performance optimization with caching

2. **Transcript Compilation (3 TODOs)**
   - Collect all debate contributions
   - Sign transcript for provenance
   - Analyze contribution patterns
   - Store for audit trail

#### Implementation Strategy:
```rust
// Apply tier-specific rules
fn apply_caws_rules(contributions: &[Contribution], tier: u32) -> Score {
    let base_score = calculate_base_score(contributions);
    
    match tier {
        1 => base_score * 1.2,  // Tier 1: strict enforcement
        2 => base_score,        // Tier 2: standard
        _ => base_score * 0.8,  // Tier 3: relaxed
    }
}
```

---

### 1.3 Worker Health Checking (`workers/src/manager.rs`)
**Status:** ❌ Using mock data  
**Complexity:** MEDIUM  
**Impact:** System observability and resilience

#### Requirements (4 TODOs in manager, 1 in executor):
1. **Actual Health Metrics** (not simulated)
   - Measure actual response times (not 150ms placeholder)
   - Query real worker state
   - Aggregate health signals
   
2. **Health Check Framework**
   - Define health check protocol
   - Implement health aggregation
   - Create remediation workflows

#### Quick Fix:
Replace mock data with real measurements:
```rust
// BEFORE: let response_time_ms: u64 = 150;
// AFTER:
let start = Instant::now();
let response = worker.health_check().await?;
let response_time_ms = start.elapsed().as_millis() as u64;
```

---

## 2. Data Processing Pipeline (33 items)

### 2.1 Multi-Modal Enrichers (15 TODOs)

#### 2.1.1 Vision Processing
- **visual_caption_enricher.rs** (2 TODOs)
  - Integrate BLIP/SigLIP via Python subprocess
  - Extract tags using NLP
  
- **vision_enricher.rs** (2 TODOs)  
  - Implement Swift bridge for Vision framework
  - Add ML model inference

#### 2.1.2 Audio Processing
- **asr_enricher.rs** (2 TODOs)
  - Python subprocess for speech recognition
  - Swift bridge to SFSpeechRecognizer
  
#### 2.1.3 Entity Enrichment
- **entity_enricher.rs** (3 TODOs)
  - Apple DataDetection for emails/URLs/dates
  - Optional NER (spaCy, NerDL)
  - BERTopic or KeyBERT for topics

### 2.2 Ingestors (12 TODOs)

#### 2.2.1 Caption Processing
- **captions_ingestor.rs** (3 TODOs)
  - SRT subtitle format parsing
  - VTT (WebVTT) format parsing
  - Segment boundary extraction

#### 2.2.2 Diagram Processing
- **diagrams_ingestor.rs** (3 TODOs)
  - SVG parsing for vector extraction
  - GraphML parsing for graph structures
  - Node/edge/label extraction

#### 2.2.3 Slide Processing
- **slides_ingestor.rs** (3 TODOs)
  - PDFKit integration for vector text
  - Text and layout extraction
  - Keynote document handling

#### 2.2.4 Video Processing
- **video_ingestor.rs** (3 TODOs)
  - AVAssetReader frame extraction via Swift
  - SSIM/pHash comparison for key frames
  - Highest quality frame selection

---

### 2.3 Indexing Infrastructure (10 TODOs)

#### 2.3.1 HNSW Vector Search (`indexers/src/hnsw_indexer.rs`)
- 3 TODOs for initialization, insertion, search
- Dependency: `hnsw_rs` or `hnswlib`
- Implement hierarchical navigation

#### 2.3.2 BM25 Full-Text Search (`indexers/src/bm25_indexer.rs`)
- 5 TODOs for Tantivy integration
- Setup, indexing, searching, commit logic

#### 2.3.3 Database Indexing (`indexers/src/database.rs`)
- 2 TODOs for vector database queries
- Implement retrieval from block_vectors table

---

## 3. Context & Learning Systems (15 items)

### 3.1 Context Preservation Engine (10 TODOs)
- **engine.rs** (3 TODOs): Background monitoring, metrics storage, trend calculation
- **multi_tenant.rs** (7 TODOs): Cache retrieval, statistics, context queries, updates

### 3.2 Claim Extraction & Verification (5 TODOs)
- **verification.rs** (2 TODOs): External API calls, semantic similarity
- **disambiguation.rs** (1 TODO): Knowledge base integration
- **multi_modal_verification.rs** (2 TODOs): Database integration

---

## 4. Testing & Infrastructure (190+ items)

### 4.1 Integration Tests
- Database availability checks
- Council system initialization
- Performance benchmarking

### 4.2 Documentation
- Core ML integration status
- Risk assessment
- TODO analyzer integration

---

## Implementation Priority Matrix

```
┌─────────────────────────────────────────────────────────┐
│          PRIORITY MATRIX (Business Impact vs Effort)    │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  HIGH IMPACT    │ ANE (HIGH)    │ Indexing (MEDIUM)    │
│  LOW EFFORT     │ Workers (MED) │ Dashboard (DONE)     │
│                 │               │                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  HIGH IMPACT    │ Council       │ Enrichers (15)       │
│  HIGH EFFORT    │ Tie-breaking  │ Ingestors (12)       │
│                 │               │                      │
├─────────────────────────────────────────────────────────┤
│                                                          │
│  LOW IMPACT     │ Tests         │ Context Engine       │
│  VARIABLE       │ Documentation │ Claim Extraction     │
│  EFFORT         │               │                      │
│                 │               │                      │
└─────────────────────────────────────────────────────────┘
```

---

## Testing Strategy for Each Implementation

### Pre-Implementation Checklist
- [ ] Write acceptance tests before implementation
- [ ] Document expected behavior
- [ ] Create mock/stub versions if needed
- [ ] Set up performance benchmarks

### Post-Implementation Checklist
- [ ] Run full test suite
- [ ] Check linting (`cargo check`, TypeScript compiler)
- [ ] Verify coverage metrics
- [ ] Document actual behavior
- [ ] Update integration tests
- [ ] Performance regression testing

---

## Integration Order Recommendation

1. **Session 2 (2-3 hours):**
   - Worker health checking (quick win, removes mock data)
   - Council tie-breaking (enables full debate resolution)
   - ANE initialization (critical path blocker)

2. **Session 3 (3-4 hours):**
   - Indexing infrastructure (HNSW + BM25)
   - Core enrichers (vision + audio)
   - Basic ingestors (captions, diagrams)

3. **Session 4+ (Ongoing):**
   - Advanced ingestors (slides, video)
   - Context preservation
   - Claim extraction
   - Comprehensive testing

---

## Resource References

### Crate Dependencies to Add/Update
```toml
# For ANE/Objective-C
objc2 = "0.5"
dispatch-rs = "0.2"

# For indexing
hnsw_rs = "0.2"
tantivy = "0.21"

# For utilities
parking_lot = "0.12"
serde = { version = "1.0", features = ["derive"] }
```

### Key Integration Points
- AudioToolbox (macOS audio)
- Vision (ML for images)
- CoreML (model inference)
- Foundation (basic utilities)
- Dispatch (async operations)

---

## Success Metrics

Each completed TODO should achieve:
- ✅ Zero compiler warnings/errors
- ✅ Documentation (JSDoc/rustdoc)
- ✅ Unit tests (80%+ coverage)
- ✅ Integration tests
- ✅ No breaking API changes
- ✅ Backward compatibility maintained

---

**Questions or Blockers?** Refer to architecture documentation in `/docs/` and Core ML implementation plan (`coreml-impl.plan.md`) for reference implementations of similar systems.
