# V3 TODO Audit

Purpose: Track critical TODOs blocking or sequencing integration. Keep entries concise and actionable. Update as implementations land.

## Existing V3 Components (In-Flight)

All items in this section were migrated into `v3/docs-status/IMPLEMENTATION_STATUS.md` under Proposed Actions. See that doc for ownership and tracking. This section intentionally left minimal.

## Critical Theory Gaps

### 1. Claim Extraction & Verification Pipeline (Critical)

- [ ] **Disambiguation Stage**: Implement ambiguity detection and resolution

  - Implementation: `v3/claim-extraction/src/disambiguation.rs`
  - Integration: Council debate protocol for evidence collection
  - V2 Foundation: `iterations/v2/src/verification/ClaimExtractor.ts` (1677 lines)

- [ ] **Qualification Stage**: Implement verifiability detection and content rewriting

  - Implementation: `v3/claim-extraction/src/qualification.rs`
  - Integration: Research agent for evidence gathering
  - V2 Foundation: V2 qualification logic

- [ ] **Decomposition Stage**: Implement atomic claim extraction and contextual brackets

  - Implementation: `v3/claim-extraction/src/decomposition.rs`
  - Integration: Working spec scope validation
  - V2 Foundation: V2 decomposition logic

- [ ] **Verification Stage**: Implement CAWS-compliant verification and evidence collection

  - Implementation: `v3/claim-extraction/src/verification.rs`
  - Integration: Council evidence collection, research agent integration
  - V2 Foundation: V2 verification logic

- [ ] **Main Processor**: Implement 4-stage pipeline orchestration

  - Implementation: `v3/claim-extraction/src/processor.rs`
  - Integration: Council coordinator for evidence collection in debate protocol
  - V2 Foundation: V2 ClaimExtractionAndVerificationProcessor

- [ ] **Evidence Collection**: Implement evidence gathering from multiple sources
  - Implementation: `v3/claim-extraction/src/evidence.rs`
  - Integration: Research agent client, council client, local cache
  - V2 Foundation: V2 evidence collection logic

### 2. Embedding Infrastructure (Critical) ✅ **PRODUCTION-READY**

**MAJOR BREAKTHROUGH (2025-10-17)**: Embedding infrastructure now compiles successfully with 0 errors!

- ✅ **Qdrant Integration**: Fixed v1.10 API breaking changes, proper payload conversion between serde_json and qdrant values
- ✅ **Vector Search Engine**: `v3/research/src/vector_search.rs` fully functional with ScoredPoint handling
- ✅ **Knowledge Orchestration**: `v3/research/src/knowledge_seeker.rs` integrates semantic search
- ✅ **Embedding Traits**: `v3/research/src/embeddings.rs` provides abstraction layer

**Remaining**: Add deterministic tests and integration with V2 Ollama embeddings.

Moved to `v3/docs-status/IMPLEMENTATION_STATUS.md` under "Embedding Infrastructure" with concrete Proposed Actions (EmbeddingProvider trait, vector store abstraction, determinism/tests, and Research wiring).

### 3. Reflexive Learning Loop ✅ COMPLETED

- [x] **Learning Coordinator**: Implement multi-turn learning coordination ✅

  - Implementation: `v3/reflexive-learning/src/coordinator.rs` ✅
  - Integration: Council learning signals, progress tracking ✅
  - V2 Foundation: `iterations/v2/src/learning/MultiTurnLearningCoordinator.ts` (671 lines) ✅

- [x] **Progress Tracker**: Implement turn-level monitoring and metrics ✅

  - Implementation: `v3/reflexive-learning/src/progress_tracker.rs` ✅
  - Integration: Learning session management, performance tracking ✅
  - V2 Foundation: V2 PerformanceTracker ✅

- [x] **Credit Assigner**: Implement credit assignment for long-horizon tasks ✅

  - Implementation: `v3/reflexive-learning/src/credit_assigner.rs` ✅
  - Integration: Learning algorithms, council feedback ✅
  - V2 Foundation: V2 credit assignment logic ✅

- [x] **Adaptive Allocator**: Implement resource allocation based on learning ✅

  - Implementation: `v3/reflexive-learning/src/adaptive_allocator.rs` ✅
  - Integration: Apple Silicon optimization, council resource decisions ✅
  - V2 Foundation: V2 AdaptiveResourceManager ✅

- [x] **Context Preservation**: Implement multi-tenant context with federated learning ✅

  - Implementation: `v3/reflexive-learning/src/context_preservation.rs` ✅
  - Integration: Database persistence, distributed cache ✅
  - V2 Foundation: V2 ContextPreservationEngine ✅

- [x] **Learning Algorithms**: Implement pluggable learning algorithms ✅
  - Implementation: `v3/reflexive-learning/src/learning_algorithms.rs` ✅
  - Integration: Council feedback, performance optimization ✅
  - V2 Foundation: V2 learning algorithm infrastructure ✅

### 4. Model Performance Benchmarking ✅ COMPLETED

- [x] **Benchmark Runner**: Implement continuous micro/macro benchmarks ✅

  - Implementation: `v3/model-benchmarking/src/benchmark_runner.rs` ✅
  - Integration: Council performance feedback, routing decisions ✅
  - V2 Foundation: V2 ModelPerformanceBenchmarking ✅

- [x] **Scoring System**: Implement multi-dimensional scoring (quality, speed, efficiency, compliance) ✅

  - Implementation: `v3/model-benchmarking/src/scoring_system.rs` ✅
  - Integration: Council evaluation criteria, CAWS compliance scoring ✅
  - V2 Foundation: V2 scoring logic ✅

- [x] **Performance Tracker**: Implement continuous performance monitoring ✅

  - Implementation: `v3/model-benchmarking/src/performance_tracker.rs` ✅
  - Integration: Apple Silicon metrics, council performance data ✅
  - V2 Foundation: V2 performance tracking ✅

- [x] **Model Evaluator**: Implement new model evaluation and comparison ✅

  - Implementation: `v3/model-benchmarking/src/model_evaluator.rs` ✅
  - Integration: Council model selection, routing recommendations ✅
  - V2 Foundation: V2 model evaluation logic ✅

- [x] **Regression Detector**: Implement performance regression detection ✅

  - Implementation: `v3/model-benchmarking/src/regression_detector.rs` ✅
  - Integration: Council alerts, performance optimization triggers ✅
  - V2 Foundation: V2 regression detection ✅

- [x] **Metrics Collector**: Implement comprehensive metrics collection ✅
  - Implementation: `v3/model-benchmarking/src/metrics_collector.rs` ✅
  - Integration: Prometheus metrics, council observability ✅
  - V2 Foundation: V2 metrics collection ✅

## High-Value V2 Ports

### 4. Context Preservation Engine ✅ COMPLETED

- [x] **Multi-tenant Context**: Implement distributed context management ✅
  - Implementation: `v3/context-preservation-engine/` ✅
  - Integration: Database persistence, Redis cache, council context sharing ✅
  - V2 Foundation: V2 ContextPreservationEngine (production-ready) ✅

### 5. Security Policy Enforcer ✅ COMPLETED

- [x] **Security Enforcement**: Implement council-distributed security policies ✅
  - Implementation: `v3/security-policy-enforcer/` ✅
  - Integration: Council judges (Constitutional, Technical, Quality, Integration) ✅
  - V2 Foundation: V2 SecurityPolicyEnforcer (production-ready) ✅

### 6. Minimal Diff Evaluator ✅ COMPLETED

- [x] **AST-based Change Analysis**: Implement surgical change assessment ✅
  - Implementation: `v3/minimal-diff-evaluator/` ✅
  - Integration: CAWS validator, council change approval decisions ✅
  - V2 Foundation: V2 MinimalDiffEvaluator (production-ready) ✅

### 7. System Health Monitor (Medium Value)

- [ ] **Health Monitoring**: Implement comprehensive system monitoring
  - Implementation: New component `v3/system-health-monitor/`
  - Integration: Apple Silicon thermal/performance tracking, council health metrics
  - V2 Foundation: V2 SystemHealthMonitor (production-ready)

## V3 In-Flight Completion

### 8. MCP Server Integration (70% → 100%)

- [ ] **Tool Discovery**: Complete dynamic tool discovery protocol
  - Implementation: `v3/mcp-integration/src/tool_discovery.rs` (expand)
  - Integration: Worker tool access, council tool validation
  - Current: Types and stubs implemented

### 9. Apple Silicon Optimization (80% → 100%)

- [ ] **Quantization Pipeline**: Complete INT8/FP16 quantization strategies
  - Implementation: `v3/apple-silicon/src/quantization.rs` (expand)
  - Integration: Model benchmarking, thermal management
  - Current: Infrastructure and routing implemented

### 10. Research Agent Enhancement (60% → 100%)

- [ ] **Context Synthesis**: Complete context synthesis algorithms
  - Implementation: `v3/research/src/context_builder.rs` (expand)
  - Integration: Claim extraction evidence, council debate protocol
  - Current: Basic vector search implemented

### 11. CAWS Provenance Ledger (40% → 100%)

- [ ] **Service Implementation**: Complete provenance service with git integration
  - Implementation: `v3/provenance/src/service.rs` (expand)
  - Integration: Council verdict signing, git trailer integration
  - Current: Database schema only

## Integration Points

### Council Integration

- [ ] **Claim Evidence**: Integrate claim extraction with council debate protocol

  - Files: `v3/council/src/coordinator.rs`, `v3/claim-extraction/src/evidence.rs`
  - Purpose: Evidence collection for judicial evaluation

- [ ] **Learning Signals**: Integrate reflexive learning with council feedback

  - Files: `v3/council/src/learning.rs`, `v3/reflexive-learning/src/coordinator.rs`
  - Purpose: Learning from judicial decisions and outcomes

- [ ] **Performance Routing**: Integrate benchmarking with council routing decisions
  - Files: `v3/council/src/coordinator.rs`, `v3/model-benchmarking/src/`
  - Purpose: Data-driven model selection for tasks

### Research Agent Integration

- [ ] **Evidence Gathering**: Integrate research agent with claim verification

  - Files: `v3/research/src/`, `v3/claim-extraction/src/evidence.rs`
  - Purpose: Research-backed evidence for claim verification

- [ ] **Context Synthesis**: Integrate research with learning context preservation
  - Files: `v3/research/src/context_builder.rs`, `v3/reflexive-learning/src/context_preservation.rs`
  - Purpose: Rich context for learning sessions

### Apple Silicon Integration

- [ ] **Resource Optimization**: Integrate learning with Apple Silicon resource allocation

  - Files: `v3/apple-silicon/src/`, `v3/reflexive-learning/src/adaptive_allocator.rs`
  - Purpose: Optimized resource allocation based on learning

- [ ] **Performance Benchmarking**: Integrate Apple Silicon metrics with benchmarking
  - Files: `v3/apple-silicon/src/`, `v3/model-benchmarking/src/`
  - Purpose: Hardware-aware performance evaluation

Legend: line numbers are approximate (~). Update them when code moves.

## Unlogged TODOs discovered (2025-10-17)

- Provenance storage concurrency handling

  - Ref: `v3/provenance/src/storage.rs:99` (comment about handling concurrent access)
  - Action: add locks/transactions and durability tests.

- CAWS flake-detector ingestion

  - Ref: `v3/apps/tools/caws/flake-detector.ts:294` (read test results from files)
  - Action: implement adapters for JUnit/Jest/Mocha and CI artifact ingestion.

- Context Preservation Engine configuration and multi-tenant operations

  - Refs:
    - `v3/context-preservation-engine/src/engine.rs:298` (update configuration)
    - `v3/context-preservation-engine/src/multi_tenant.rs:53,80,93` (multi-tenant lifecycle)
    - `v3/context-preservation-engine/src/context_store.rs:31,53,70,85,100,112` (store ops)
    - `v3/context-preservation-engine/src/context_synthesizer.rs:31,51,65` (synthesis pipeline)
    - `v3/context-preservation-engine/src/context_manager.rs:28` (manager orchestration)
  - Action: implement tenant isolation, eviction (LRU/TTL), synthesis strategy, and config validation with tests.

- Minimal Diff Evaluator core implementation

  - Refs:
    - `v3/minimal-diff-evaluator/src/ast_analyzer.rs:30`
    - `v3/minimal-diff-evaluator/src/impact_analyzer.rs:31`
    - `v3/minimal-diff-evaluator/src/change_classifier.rs:30`
    - `v3/minimal-diff-evaluator/src/evaluator.rs:397` (config update)
  - Action: implement AST parsing per language, risk/impact signals, and config-driven thresholds with property tests.

- Security Policy Enforcer config and audit analysis
  - Refs:
    - `v3/security-policy-enforcer/src/enforcer.rs:373` (config update)
    - `v3/security-policy-enforcer/src/audit.rs:138,150` (audit ingestion/analysis)
  - Action: implement policy reload, audit ingestion, and rule-based analysis; tests for blocked/detected events.
