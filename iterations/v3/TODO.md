# V3 TODO Audit

Purpose: Track critical TODOs blocking or sequencing integration. Keep entries concise and actionable. Update as implementations land.

## Existing V3 Components (In-Flight)

- [ ] Orchestration: pass real patches/language hints to validator

  - File: `v3/orchestration/src/orchestrate.rs:~36`
  - Note: Replace empty slices with data from worker artifacts (patch diffs + file extensions)

- [ ] Orchestration: map full TaskSpec fields

  - File: `v3/orchestration/src/orchestrate.rs:~10`
  - Note: Populate acceptance_criteria and description from WorkingSpec/TaskDescriptor once available

- [ ] Validator: replace NoopMde with real MDE

  - File: `v3/orchestration/src/caws_runtime.rs:~95`
  - Note: Implement MinimalDiffEvaluator and calibrate thresholds per tier/language

- [ ] Validator: expand MDE violation mapping

  - File: `v3/orchestration/src/caws_runtime.rs:~132`
  - Note: Map risky_patterns to specific violation codes (e.g., LargeRefactor, GeneratedCodeDump)

- [ ] Persistence: persist waivers alongside verdicts

  - File: `v3/orchestration/src/persistence_postgres.rs:~37`
  - Note: Call persist_waivers() after final decision; consider FK to verdict_id

- [ ] Persistence: add signing + hash chain per ADR-003

  - File: `v3/orchestration/src/persistence_postgres.rs:~20`
  - Note: Introduce signer trait, store signature/hash_chain into verdicts table

- [ ] Coordinator: accept validator + research context

  - File: `v3/council/src/coordinator.rs:~60`
  - Note: Extend evaluation context to include ValidationResult and research evidence bundle

- [ ] Coordinator: populate remediation and constitutional_refs from inputs

  - File: `v3/council/src/coordinator.rs:~130`
  - Note: Merge validator remediation and refs into FinalVerdict consistently

- [ ] Coordinator: compute verification_summary when claims present

  - File: `v3/council/src/coordinator.rs:~150`
  - Note: Use WorkerOutput.claims/evidence_refs to build coverage stats

- [ ] DB: tune pool and retries per environment

  - File: `v3/orchestration/src/db.rs:~8`
  - Note: Configurable timeouts, retries, and backoff

- [ ] Docs: link MDE implementation doc when created

  - File: `v3/docs/caws-runtime-validator.md:~28`
  - Note: Add link once MDE module exists

- [ ] Docs: signer and git trailer integration guide
  - File: `v3/docs/database/provenance.md:~20`
  - Note: Document signer trait usage and git trailer workflow

## Critical Theory Gaps (Phase 1 - Weeks 1-4)

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

### 2. Embedding Infrastructure (Critical)

- [ ] **Embedding Service**: Implement Ollama embeddinggemma integration

  - Implementation: `v3/embedding-service/src/lib.rs`
  - Integration: Council semantic context, research agent knowledge retrieval
  - V2 Foundation: `iterations/v2/src/embeddings/EmbeddingService.ts` (473 lines)

- [ ] **Semantic Context Generation**: Implement context-aware embedding generation

  - Implementation: `v3/embedding-service/src/context.rs`
  - Integration: Council coordinator for semantic task understanding
  - V2 Foundation: `iterations/v2/src/context/ContextManager.ts` (semantic context generation)

- [ ] **Vector Similarity Search**: Implement embedding-based similarity matching

  - Implementation: `v3/embedding-service/src/similarity.rs`
  - Integration: Research agent for evidence correlation
  - V2 Foundation: `iterations/v2/src/embeddings/EmbeddingService.ts` (similarity functions)

- [ ] **Council Semantic Integration**: Implement semantic evaluation in council judges

  - Implementation: `v3/council/src/semantic.rs`
  - Integration: All council judges for context-aware decision making
  - V2 Foundation: `iterations/v2/src/orchestrator/ArbiterOrchestrator.ts` (semantic agent selection)

- [ ] **Research Agent Embeddings**: Implement knowledge base embedding integration

  - Implementation: `v3/research/src/embeddings.rs`
  - Integration: Research agent for semantic knowledge retrieval
  - V2 Foundation: `iterations/v2/src/embeddings/EmbeddingService.ts` (knowledge base integration)

### 3. Reflexive Learning Loop (High Priority)

- [ ] **Learning Coordinator**: Implement multi-turn learning coordination

  - Implementation: `v3/reflexive-learning/src/coordinator.rs`
  - Integration: Council learning signals, progress tracking
  - V2 Foundation: `iterations/v2/src/learning/MultiTurnLearningCoordinator.ts` (671 lines)

- [ ] **Progress Tracker**: Implement turn-level monitoring and metrics

  - Implementation: `v3/reflexive-learning/src/progress_tracker.rs`
  - Integration: Learning session management, performance tracking
  - V2 Foundation: V2 PerformanceTracker

- [ ] **Credit Assigner**: Implement credit assignment for long-horizon tasks

  - Implementation: `v3/reflexive-learning/src/credit_assigner.rs`
  - Integration: Learning algorithms, council feedback
  - V2 Foundation: V2 credit assignment logic

- [ ] **Adaptive Allocator**: Implement resource allocation based on learning

  - Implementation: `v3/reflexive-learning/src/adaptive_allocator.rs`
  - Integration: Apple Silicon optimization, council resource decisions
  - V2 Foundation: V2 AdaptiveResourceManager

- [ ] **Context Preservation**: Implement multi-tenant context with federated learning

  - Implementation: `v3/reflexive-learning/src/context_preservation.rs`
  - Integration: Database persistence, distributed cache
  - V2 Foundation: V2 ContextPreservationEngine

- [ ] **Learning Algorithms**: Implement pluggable learning algorithms
  - Implementation: `v3/reflexive-learning/src/learning_algorithms.rs`
  - Integration: Council feedback, performance optimization
  - V2 Foundation: V2 learning algorithm infrastructure

### 3. Model Performance Benchmarking (High Priority)

- [ ] **Benchmark Runner**: Implement continuous micro/macro benchmarks

  - Implementation: `v3/model-benchmarking/src/benchmark_runner.rs`
  - Integration: Council performance feedback, routing decisions
  - V2 Foundation: V2 ModelPerformanceBenchmarking

- [ ] **Scoring System**: Implement multi-dimensional scoring (quality, speed, efficiency, compliance)

  - Implementation: `v3/model-benchmarking/src/scoring_system.rs`
  - Integration: Council evaluation criteria, CAWS compliance scoring
  - V2 Foundation: V2 scoring logic

- [ ] **Performance Tracker**: Implement continuous performance monitoring

  - Implementation: `v3/model-benchmarking/src/performance_tracker.rs`
  - Integration: Apple Silicon metrics, council performance data
  - V2 Foundation: V2 performance tracking

- [ ] **Model Evaluator**: Implement new model evaluation and comparison

  - Implementation: `v3/model-benchmarking/src/model_evaluator.rs`
  - Integration: Council model selection, routing recommendations
  - V2 Foundation: V2 model evaluation logic

- [ ] **Regression Detector**: Implement performance regression detection

  - Implementation: `v3/model-benchmarking/src/regression_detector.rs`
  - Integration: Council alerts, performance optimization triggers
  - V2 Foundation: V2 regression detection

- [ ] **Metrics Collector**: Implement comprehensive metrics collection
  - Implementation: `v3/model-benchmarking/src/metrics_collector.rs`
  - Integration: Prometheus metrics, council observability
  - V2 Foundation: V2 metrics collection

## High-Value V2 Ports (Phase 2 - Weeks 5-8)

### 4. Context Preservation Engine (High Value)

- [ ] **Multi-tenant Context**: Implement distributed context management
  - Implementation: `v3/reflexive-learning/src/context_preservation.rs` (expand)
  - Integration: Database persistence, Redis cache, council context sharing
  - V2 Foundation: V2 ContextPreservationEngine (production-ready)

### 5. Adaptive Resource Manager (Medium Value)

- [ ] **Resource Allocation**: Implement tier-based resource allocation
  - Implementation: `v3/reflexive-learning/src/adaptive_allocator.rs` (expand)
  - Integration: Apple Silicon ANE/GPU/CPU routing, council resource decisions
  - V2 Foundation: V2 AdaptiveResourceManager (production-ready)

### 6. Security Policy Enforcer (Medium Value)

- [ ] **Security Enforcement**: Implement council-distributed security policies
  - Implementation: New component `v3/security-policy-enforcer/`
  - Integration: Council judges (Constitutional, Technical, Quality, Integration)
  - V2 Foundation: V2 SecurityPolicyEnforcer (production-ready)

### 7. System Health Monitor (Medium Value)

- [ ] **Health Monitoring**: Implement comprehensive system monitoring
  - Implementation: New component `v3/system-health-monitor/`
  - Integration: Apple Silicon thermal/performance tracking, council health metrics
  - V2 Foundation: V2 SystemHealthMonitor (production-ready)

## V3 In-Flight Completion (Phase 3 - Weeks 9-12)

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
