# V3 Integration Points for V2 Components

## Executive Summary

V3 has established a solid architectural foundation with clear integration points for pulling V2's mature components. This document identifies **key integration hooks** where V2 components should be ported to complete V3's functionality.

**Key Finding**: V3 is architecturally ready for V2 integration - the stubs and interfaces are already in place, but the implementations are missing.

---

## 🔗 Primary Integration Points

### 1. Claim Extraction Pipeline (`claim-extraction/`)

**V3 Status**: Framework exists, implementation is stub-only
**V2 Component**: `ClaimExtractor.ts` (1677 lines of proven claim processing)
**Integration Point**: `src/processor.rs` (currently just returns empty results)

**Files to Integrate**:

```rust
// V3 claim-extraction/src/processor.rs - CURRENTLY STUB
pub async fn run(&self, _input: &str, _ctx: &ProcessingContext) -> Result<ClaimExtractionResult, ClaimExtractionError> {
    // Stub: return empty extraction result
    Ok(ClaimExtractionResult { /* empty */ })
}

// SHOULD BECOME: Full 4-stage pipeline
pub async fn run(&self, input: &str, ctx: &ProcessingContext) -> Result<ClaimExtractionResult, ClaimExtractionError> {
    // 1. Disambiguation (pronoun resolution, ambiguity detection)
    // 2. Qualification (subjective vs objective claims)
    // 3. Decomposition (compound sentence splitting)
    // 4. Verification (evidence collection)
}
```

**V2 Dependencies to Port**:

- `ClaimExtractor.ts` → `processor.rs`
- `VerificationEngine.ts` → `verification.rs`
- `FactChecker.ts` → Integration with research agent
- `CredibilityScorer.ts` → `evidence.rs`

### 2. Council Judge Enhancement (`council/`)

**V3 Status**: Judges can render verdicts but lack evidence processing
**V2 Component**: Verification pipeline for evidence-backed judgments
**Integration Point**: `coordinator.rs` evidence enrichment hooks

**Current Hook**:

```rust
// coordinator.rs - Evidence collection integration point
pub async fn build_consensus(&self, task_id: TaskId) -> Result<ConsensusResult> {
    // AFTER: Judge evaluations complete
    // BEFORE: Consensus calculation

    // INTEGRATION POINT: Enrich judge verdicts with V2 evidence
    // Call claim-extraction processor here
    // Add evidence to judge evaluations
}
```

**Integration Strategy**:

1. **Pre-Judge Processing**: Run claim extraction on task input
2. **Evidence Enrichment**: Add extracted claims to judge prompts
3. **Post-Judge Validation**: Verify judge verdicts against extracted evidence
4. **Debate Integration**: Use extracted claims in debate protocol

### 3. Research Agent Knowledge Integration (`research/`)

**V3 Status**: Vector search framework exists, but no knowledge sources
**V2 Component**: `KnowledgeSeeker.ts` and knowledge graph
**Integration Point**: `knowledge_seeker.rs` and `vector_search.rs`

**Current Structure**:

```rust
// research/src/knowledge_seeker.rs
pub struct KnowledgeSeeker {
    vector_engine: VectorSearchEngine,
    web_scraper: WebScraper,
    // MISSING: V2 KnowledgeDatabaseClient integration
}
```

**V2 Components to Integrate**:

- `KnowledgeDatabaseClient.ts` → Database query interface
- `HybridSearchView` → Graph-vector search logic
- `ConfidenceManager.ts` → Knowledge scoring
- `EmbeddingService.ts` → Vector generation

### 4. MCP Server Tool Integration (`mcp-integration/`)

**V3 Status**: MCP server framework exists for tool discovery
**V2 Component**: Verification tools and CAWS validation
**Integration Point**: `caws_integration.rs` and tool registry

**Current Integration Points**:

```rust
// mcp-integration/src/caws_integration.rs
pub struct CawsIntegration {
    // INTEGRATION POINT: Add V2 CAWS validator
    // INTEGRATION POINT: Add V2 verification tools
    // INTEGRATION POINT: Add V2 provenance tracking
}
```

---

## 🏗️ Secondary Integration Points

### 5. Worker Pool CAWS Integration (`workers/`)

**Integration Point**: Pre/post execution hooks for CAWS compliance
**V2 Component**: `caws-runtime-validator` and self-check utilities

### 6. Orchestration Layer (`orchestration/`)

**Integration Point**: Task routing with semantic context
**V2 Component**: `ArbiterOrchestrator` semantic agent selection

### 7. Database Layer Extensions (`database/`)

**Integration Point**: Additional tables for V2 schemas
**V2 Component**: Knowledge graph tables, provenance tracking

### 8. Apple Silicon Optimization (`apple-silicon/`)

**Integration Point**: Model optimization for V2 components
**V2 Component**: Performance-critical embedding and verification services

---

## 🔄 Data Flow Integration Architecture

```
Task Input
    ↓
V3 Council Coordinator
    ↓ [INTEGRATION POINT 1]
V2 Claim Extraction Pipeline (4-stage processing)
    ↓ [INTEGRATION POINT 2]
Judge Evaluations (evidence-enriched)
    ↓ [INTEGRATION POINT 3]
V2 Knowledge Graph Search (for judge research)
    ↓
Consensus Building
    ↓ [INTEGRATION POINT 4]
V2 Provenance Tracking (CAWS compliance)
    ↓
Final Verdict
```

---

## 📋 Implementation Priority Matrix

### Phase 1: Core Evidence Pipeline (Week 1-2)

1. **Port Claim Extraction Processor** - Highest priority, enables judge evidence processing
2. **Integrate Council Evidence Hooks** - Connect claim processing to judge evaluation
3. **Add Basic Knowledge Graph Client** - Enable semantic context for judges

### Phase 2: Enhanced Verification (Week 3-4)

1. **Port Verification Engine** - Add multi-source fact-checking to research agent
2. **Implement Credibility Scoring** - Evidence weighting and validation
3. **Add Confidence Management** - Knowledge scoring and decay

### Phase 3: Production Resilience (Week 5-6)

1. **Port Resilience Infrastructure** - Circuit breakers, retry logic, health checks
2. **Integrate Structured Logging** - Observability and monitoring
3. **Add CAWS Provenance** - Audit trail compliance

### Phase 4: Advanced Features (Week 7-8)

1. **Port Embedding Service** - Semantic search capabilities
2. **Integrate Hybrid Search** - Graph-vector knowledge retrieval
3. **Add Apple Silicon Optimization** - Performance tuning for new components

---

## 🔧 Technical Integration Details

### Claim Extraction Integration

**V3 Interface** (Already Exists):

```rust
pub trait ClaimProcessor {
    async fn process_sentence(
        &self,
        sentence: &str,
        context: &ProcessingContext,
    ) -> Result<ClaimExtractionResult, ClaimExtractionError>;
}
```

**V2 Implementation** (To Port):

```typescript
export class ClaimExtractor {
  async extractAtomicClaims(
    sentence: string,
    context: ConversationContext
  ): Promise<AtomicClaim[]> {
    // 4-stage pipeline implementation
  }
}
```

**Integration Mapping**:

- V2 `ConversationContext` → V3 `ProcessingContext`
- V2 `AtomicClaim[]` → V3 `Vec<AtomicClaim>`
- V2 `EvidenceManifest` → V3 `Vec<Evidence>`

### Council Evidence Enrichment

**Integration Hook** (Add to coordinator.rs):

```rust
impl ConsensusCoordinator {
    async fn enrich_judge_verdicts_with_evidence(
        &self,
        task_spec: &TaskSpec,
        evaluations: &mut HashMap<JudgeId, JudgeEvaluation>,
    ) -> Result<()> {
        // INTEGRATION POINT: Call claim extraction
        let claims = self.claim_processor
            .process_sentence(&task_spec.description, &task_spec.into())
            .await?;

        // INTEGRATION POINT: Add claims to judge prompts
        for evaluation in evaluations.values_mut() {
            evaluation.enriched_evidence = claims.verification_evidence;
        }

        Ok(())
    }
}
```

### Knowledge Graph Integration

**Integration Point** (Extend research/src/knowledge_seeker.rs):

```rust
pub struct KnowledgeSeeker {
    vector_engine: VectorSearchEngine,
    web_scraper: WebScraper,
    // ADD: V2 Knowledge Graph Integration
    knowledge_client: KnowledgeDatabaseClient, // From V2
    confidence_manager: ConfidenceManager,      // From V2
}

impl KnowledgeSeeker {
    pub async fn perform_semantic_search(
        &self,
        query: &KnowledgeQuery,
    ) -> Result<Vec<KnowledgeResult>> {
        // INTEGRATION POINT: Use V2 hybrid search
        let graph_results = self.knowledge_client.search_entity_graph(&query.entity_id).await?;
        let vector_results = self.vector_engine.search(&query.text).await?;

        // INTEGRATION POINT: Apply V2 confidence scoring
        let scored_results = self.confidence_manager.score_results(
            graph_results,
            vector_results
        ).await?;

        Ok(scored_results)
    }
}
```

---

## 🧪 Testing Integration Strategy

### Unit Tests

- Test V2 component ports in isolation
- Verify interface compatibility with V3 types
- Performance regression tests against V2 benchmarks

### Integration Tests

- End-to-end claim processing through council
- Judge evidence enrichment verification
- Knowledge graph search integration

### Council Integration Tests

```rust
#[tokio::test]
async fn test_council_with_claim_processing() {
    // GIVEN: Task with complex claims
    let task_spec = create_task_with_complex_claims();

    // WHEN: Council processes with V2 integration
    let result = coordinator.evaluate_task(task_spec).await?;

    // THEN: Verdicts include extracted claims and evidence
    assert!(result.individual_verdicts.values().all(|v| !v.evidence.is_empty()));
}
```

---

## 📊 Success Metrics

### Functional Completeness

- [ ] Council judges can process complex linguistic patterns
- [ ] Evidence-backed verdicts with verifiable claims
- [ ] Semantic knowledge search for judge research
- [ ] CAWS compliance automatically verified

### Performance Targets

- [ ] Claim processing <500ms (V2 standard)
- [ ] Judge evaluation <1s with evidence enrichment
- [ ] Knowledge search <100ms with confidence scoring
- [ ] Council consensus <2s end-to-end

### Quality Gates

- [ ] 95%+ evidence coverage for judge verdicts
- [ ] Zero claim extraction failures on test suite
- [ ] CAWS compliance rate >98%
- [ ] Complete audit trails for all decisions

---

## 🚀 Implementation Roadmap

### Week 1: Foundation

- Set up V2 component extraction scripts
- Create Rust interface adapters for V2 types
- Establish integration test framework

### Week 2: Core Claim Processing

- Port ClaimExtractor.ts to processor.rs
- Integrate with council coordinator
- Add evidence enrichment to judge evaluations

### Week 3: Knowledge Integration

- Port KnowledgeDatabaseClient to research crate
- Add confidence scoring to knowledge results
- Integrate semantic search with council debates

### Week 4: Verification Pipeline

- Port VerificationEngine.ts to verification.rs
- Add multi-source fact-checking
- Implement credibility scoring

### Week 5: Production Hardening

- Port resilience infrastructure (circuit breakers, retries)
- Add structured logging and monitoring
- Implement health checks

### Week 6: CAWS Integration

- Port provenance tracking system
- Add CAWS compliance validation
- Implement audit trail verification

---

## 🎯 Critical Success Factors

1. **Incremental Integration**: Port V2 components one-by-one with full testing
2. **Interface Compatibility**: Ensure V2/V3 type compatibility through adapters
3. **Performance Preservation**: Maintain V2 performance standards in Rust ports
4. **Evidence Quality**: Judges must have access to high-quality, verified evidence
5. **CAWS Compliance**: All integrations must maintain constitutional requirements

---

## 📋 Next Steps

1. **Immediate**: Create detailed porting plan for ClaimExtractor.ts
2. **Week 1**: Begin claim processing integration with council
3. **Week 2**: Add evidence enrichment to judge evaluations
4. **Week 3**: Integrate knowledge graph search for research
5. **Ongoing**: Performance benchmarking and quality assurance

**V3 has the architecture. V2 has the implementations. Integration will create the ultimate enterprise AI system.**
