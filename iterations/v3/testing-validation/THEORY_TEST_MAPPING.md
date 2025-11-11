# Theory Document → Test Coverage Mapping

**Generated**: 2025-11-10  
**Purpose**: Map existing tests to theory document sections (`docs/arbiter/theory.md`)  
**Status**: Partial coverage - critical gaps identified

---

## Mapping Methodology

This document maps each section of `docs/arbiter/theory.md` to:
- ✅ **Existing tests** that validate the theory
- ❌ **Missing tests** that need to be created
- ⚠️ **Partial coverage** where tests exist but don't fully validate the theory

---

## Theory Section → Test Mapping

### 1. Overview and Goals

**Theory Section**: Lines 1-20  
**Key Concepts**:
- CAWS-integrated arbiter stack
- Runtime governance system
- Constitutional authority
- Executable contract governance

**Test Coverage**:
- ✅ **`caws_governance.rs`** - Tests CAWS constitutional authority, working spec validation, budget enforcement
- ✅ **`integrated_playground_quality.rs`** - Tests CAWS compliance in agent execution
- ⚠️ **Missing**: No test specifically validates "CAWS as executable contract" concept

**Test References**:
```rust
// caws_governance.rs:34-47
test_working_spec_validation() // Validates working spec as executable contract
test_budget_enforcement() // Validates CAWS budget enforcement
test_scope_boundary_enforcement() // Validates scope boundaries
```

---

### 2. Hardware for Local Performance

**Theory Section**: Lines 22-32  
**Key Concepts**:
- Apple Silicon M-series optimization
- Core ML acceleration (2.8x speedup)
- Unified memory architecture
- ANE (Apple Neural Engine) utilization

**Test Coverage**:
- ❌ **Missing**: No tests validate hardware-specific optimizations
- ❌ **Missing**: No tests validate Core ML acceleration
- ❌ **Missing**: No tests validate ANE utilization
- ⚠️ **Partial**: `performance_scalability.rs` tests general performance but not hardware-specific

**Required Tests**:
- [ ] **Core ML Acceleration Test**: Verify 2.8x speedup vs CPU fallback
- [ ] **ANE Utilization Test**: Verify ANE is used for inference
- [ ] **Unified Memory Test**: Verify memory efficiency on Apple Silicon
- [ ] **Model Loading Test**: Verify models load efficiently on M-series hardware

---

### 3. Orchestration Model and Arbitration Mechanisms

**Theory Section**: Lines 34-46  
**Key Concepts**:
- Centralized coordinator agent
- LLM debate approach (AWS research)
- Built-in judging module (Mistral 7B)
- Self-consistency & voting
- Reflective iteration loops

**Test Coverage**:
- ✅ **`multi_agent_coordination.rs`** - Tests arbitration mechanisms, conflict resolution
- ✅ **`self_prompting_loops.rs`** - Tests iterative refinement (reflective loops)
- ✅ **`integrated_playground_quality.rs`** - Tests evaluation and refinement cycles
- ⚠️ **Partial**: No explicit LLM debate tests
- ⚠️ **Partial**: No explicit self-consistency voting tests

**Test References**:
```rust
// multi_agent_coordination.rs:56-69
test_arbitration_mechanism() // Tests arbitration logic

// self_prompting_loops.rs:43-57
test_satisficing_logic() // Tests iterative improvement

// integrated_playground_quality.rs:700-800
run_playground_test_with_feedback() // Tests reflective iteration
```

**Missing Tests**:
- [ ] **LLM Debate Test**: Test AWS-style debate between models with judge
- [ ] **Self-Consistency Test**: Test running model multiple times and voting
- [ ] **Judge Model Test**: Test Mistral 7B as judge for outputs
- [ ] **Reflective Iteration Test**: Test explicit reflection loops

---

### 4. Model-Agnostic Design and Hot-Swapping Capability

**Theory Section**: Lines 48-65  
**Key Concepts**:
- Pluggable model interfaces
- Performance tracking & preference
- Dynamic model selection
- Speculative execution with multiple models
- Hybrid routing (fast + accurate models)

**Test Coverage**:
- ✅ **`self_prompting_loops.rs`** - Tests model hot-swapping
- ✅ **`reflexive_learning.rs`** - Tests performance tracking
- ⚠️ **Partial**: No explicit pluggable interface tests
- ⚠️ **Partial**: No speculative execution tests
- ⚠️ **Partial**: No hybrid routing tests

**Test References**:
```rust
// self_prompting_loops.rs:92-100
test_model_hot_swap() // Tests model swapping during loops

// reflexive_learning.rs:40-53
test_performance_data_collection() // Tests performance tracking
```

**Missing Tests**:
- [ ] **Pluggable Interface Test**: Test swapping between different model backends
- [ ] **Performance Preference Test**: Test routing based on historical performance
- [ ] **Speculative Execution Test**: Test fast + accurate model race condition
- [ ] **Hybrid Routing Test**: Test intelligent model selection (fast vs accurate)
- [ ] **Model Registry Test**: Test model discovery and registration

---

### 5. Low-Level Implementation and Performance Considerations

**Theory Section**: Lines 67-90  
**Key Concepts**:
- Rust/C++ implementation (not Python)
- Graph-Flow style orchestration
- Async concurrency (`async/.await`)
- Core ML runtime integration
- ONNX Runtime / TensorRT support
- Direct hardware access (SIMD, GPU cores)

**Test Coverage**:
- ✅ **`performance_scalability.rs`** - Tests general performance
- ⚠️ **Partial**: Tests performance but not implementation language validation
- ❌ **Missing**: No tests validate Rust implementation efficiency
- ❌ **Missing**: No tests validate async concurrency performance
- ❌ **Missing**: No tests validate Core ML integration

**Test References**:
```rust
// performance_scalability.rs:1-100
test_resource_utilization() // Tests CPU/memory usage
test_concurrent_load() // Tests concurrent execution
```

**Missing Tests**:
- [ ] **Rust Implementation Test**: Verify Rust performance vs Python baseline
- [ ] **Async Concurrency Test**: Test parallel task orchestration efficiency
- [ ] **Core ML Integration Test**: Test Core ML runtime integration
- [ ] **Hardware Acceleration Test**: Verify GPU/ANE utilization
- [ ] **Memory Efficiency Test**: Verify low memory footprint

---

### 6. Ensuring Correctness and Traceability

**Theory Section**: Lines 92-110  
**Key Concepts**:
- Validation tests (automatic checks)
- Consistency and rule enforcement
- Arbiter as auditor (LLM critique)
- Comprehensive logging
- Audit trails
- Versioning (model/prompt versions)
- Real-time monitoring

**Test Coverage**:
- ✅ **`caws_governance.rs`** - Tests validation and rule enforcement
- ✅ **`integrated_playground_quality.rs`** - Tests validation (compilation checks)
- ✅ **`claim_verification.rs`** - Tests factual accuracy validation
- ⚠️ **Partial**: No explicit audit trail tests
- ⚠️ **Partial**: No explicit versioning tests
- ⚠️ **Partial**: No explicit monitoring tests

**Test References**:
```rust
// caws_governance.rs:34-47
test_working_spec_validation() // Tests validation logic

// integrated_playground_quality.rs:400-500
check_code_compiles() // Tests automatic validation

// claim_verification.rs:32-46
test_claim_extraction() // Tests factual validation
```

**Missing Tests**:
- [ ] **Audit Trail Test**: Verify complete decision logging
- [ ] **Versioning Test**: Test model/prompt version tracking
- [ ] **Monitoring Test**: Test real-time metrics and dashboards
- [ ] **Traceability Test**: Test "which model contributed what" queries
- [ ] **Arbiter Auditor Test**: Test LLM-based output critique

---

### 7. High-Quality Claim Extraction and Factual Verification

**Theory Section**: Lines 112-546  
**Key Concepts**:
- **Stage 1: Contextual Disambiguation** - Ambiguity detection and resolution
- **Stage 2: Verifiable Content Qualification** - Factual gatekeeping
- **Stage 3: Precise Claim Decomposition** - Atomic claim extraction
- **Stage 4: CAWS-Compliant Verification** - Evidence-based validation
- Multi-modal claim extraction
- Claim-based arbitration
- Research-based evaluation metrics

**Test Coverage**:
- ✅ **`claim_verification.rs`** - Tests claim extraction pipeline
- ⚠️ **Partial**: Tests extraction but not all 4 stages comprehensively
- ❌ **Missing**: No tests for Stage 1 (disambiguation) details
- ❌ **Missing**: No tests for Stage 2 (qualification) details
- ❌ **Missing**: No tests for Stage 3 (decomposition) details
- ❌ **Missing**: No tests for Stage 4 (CAWS verification) details
- ❌ **Missing**: No multi-modal claim extraction tests
- ❌ **Missing**: No claim-based arbitration tests

**Test References**:
```rust
// claim_verification.rs:32-46
test_claim_extraction() // Tests basic extraction

// claim_verification.rs:80-93
test_contextual_disambiguation() // Tests disambiguation (partial)
```

**Missing Tests**:
- [ ] **Stage 1: Disambiguation Test** - Test ambiguity detection and resolution
- [ ] **Stage 2: Qualification Test** - Test verifiable content detection
- [ ] **Stage 3: Decomposition Test** - Test atomic claim extraction
- [ ] **Stage 4: CAWS Verification Test** - Test evidence-based validation
- [ ] **Multi-Modal Extraction Test** - Test code/documentation/data claims
- [ ] **Claim-Based Arbitration Test** - Test using claims for decisions
- [ ] **Research Metrics Test** - Test coverage/decontextualization metrics

---

### 8. CAWS-Compliant Arbitration Protocol

**Theory Section**: Lines 548-560  
**Key Concepts**:
- CAWS Adjudication Cycle (Pleading → Examination → Deliberation → Verdict → Publication)
- JSON RPC to Arbiter
- Rust validator using CAWS schemas
- Local plug-ins (build, lint, coverage)
- Signed YAML verdict record
- Git integration with `CAWS-VERDICT-ID` trailer

**Test Coverage**:
- ✅ **`caws_governance.rs`** - Tests CAWS adjudication cycle
- ✅ **`integrated_playground_quality.rs`** - Tests CAWS compliance checks
- ⚠️ **Partial**: Tests governance but not full adjudication cycle
- ❌ **Missing**: No tests for JSON RPC protocol
- ❌ **Missing**: No tests for Git integration with verdict trailers

**Test References**:
```rust
// caws_governance.rs:34-100
test_working_spec_validation() // Tests examination stage
test_budget_enforcement() // Tests deliberation stage
test_waiver_workflow() // Tests verdict stage
```

**Missing Tests**:
- [ ] **Full Adjudication Cycle Test** - Test all 5 stages end-to-end
- [ ] **JSON RPC Protocol Test** - Test worker → arbiter communication
- [ ] **Rust Validator Test** - Test CAWS schema validation
- [ ] **Git Integration Test** - Test verdict commit with trailer
- [ ] **Signed Verdict Test** - Test YAML verdict signing

---

### 9. Arbiter Reasoning Engine

**Theory Section**: Lines 562-578  
**Key Concepts**:
- CAWS Debate protocol
- Evidence completeness scoring (E)
- Budget adherence scoring (B)
- Gate integrity scoring (G)
- Provenance clarity scoring (P)
- Composite score: `S = 0.4E + 0.3B + 0.2G + 0.1P`
- CoreML Mistral as judge

**Test Coverage**:
- ✅ **`multi_agent_coordination.rs`** - Tests arbitration mechanisms
- ✅ **`caws_governance.rs`** - Tests CAWS compliance scoring
- ⚠️ **Partial**: Tests arbitration but not explicit CAWS Debate protocol
- ❌ **Missing**: No tests for composite scoring formula
- ❌ **Missing**: No tests for CoreML Mistral as judge

**Test References**:
```rust
// multi_agent_coordination.rs:56-69
test_arbitration_mechanism() // Tests arbitration logic

// caws_governance.rs:49-65
test_budget_enforcement() // Tests budget adherence (B)
```

**Missing Tests**:
- [ ] **CAWS Debate Test** - Test debate protocol between workers
- [ ] **Composite Scoring Test** - Test `S = 0.4E + 0.3B + 0.2G + 0.1P` formula
- [ ] **Evidence Completeness Test** - Test E scoring
- [ ] **Gate Integrity Test** - Test G scoring
- [ ] **Provenance Clarity Test** - Test P scoring
- [ ] **CoreML Judge Test** - Test Mistral as judge model

---

### 10. Reflexive Learning & Memory Integration

**Theory Section**: Lines 580-715  
**Key Concepts**:
- Multi-tenant context offloading
- Federated learning engine
- Progress tracking & turn-level monitoring
- Trajectory analysis
- Rubric engineering framework
- Thinking budget management
- Curriculum learning system
- Failure mode detection & mitigation

**Test Coverage**:
- ✅ **`reflexive_learning.rs`** - Tests reflexive learning components
- ✅ **`self_prompting_loops.rs`** - Tests iterative improvement
- ⚠️ **Partial**: Tests learning but not all components
- ❌ **Missing**: No tests for federated learning
- ❌ **Missing**: No tests for context offloading
- ❌ **Missing**: No tests for curriculum learning

**Test References**:
```rust
// reflexive_learning.rs:40-99
test_performance_data_collection() // Tests progress tracking
test_learning_adaptation() // Tests learning from outcomes
test_curriculum_progression() // Tests curriculum (partial)
test_adaptive_resource_allocation() // Tests resource allocation
```

**Missing Tests**:
- [ ] **Federated Learning Test** - Test cross-tenant learning
- [ ] **Context Offloading Test** - Test multi-tenant memory management
- [ ] **Trajectory Analysis Test** - Test turn-level reward assignment
- [ ] **Rubric Engineering Test** - Test weighted reward calculation
- [ ] **Thinking Budget Test** - Test thinking resource optimization
- [ ] **Failure Mode Detection Test** - Test RL instability mitigation

---

### 11. Model Performance Benchmarking & Evaluation System

**Theory Section**: Lines 716-1068  
**Key Concepts**:
- Continuous micro-benchmarks (daily)
- Macro-benchmarks (weekly)
- New model evaluation pipeline (monthly)
- Multi-dimensional scoring framework
- Dynamic weighting by task surface
- "Good enough" performance criteria
- Model lifecycle management
- Reflexive model selection

**Test Coverage**:
- ✅ **`performance_scalability.rs`** - Tests performance metrics
- ✅ **`reflexive_learning.rs`** - Tests performance tracking
- ❌ **Missing**: No tests for benchmarking cadence
- ❌ **Missing**: No tests for model evaluation pipeline
- ❌ **Missing**: No tests for scoring framework
- ❌ **Missing**: No tests for model lifecycle management

**Test References**:
```rust
// performance_scalability.rs:1-100
test_resource_utilization() // Tests basic performance
test_concurrent_load() // Tests scalability
```

**Missing Tests**:
- [ ] **Micro-Benchmark Test** - Test daily active model health checks
- [ ] **Macro-Benchmark Test** - Test weekly comprehensive evaluation
- [ ] **New Model Pipeline Test** - Test monthly new model assessment
- [ ] **Scoring Framework Test** - Test multi-dimensional scoring
- [ ] **Task Surface Weighting Test** - Test dynamic weighting
- [ ] **Performance Thresholds Test** - Test "good enough" criteria
- [ ] **Model Lifecycle Test** - Test update/retirement strategies
- [ ] **Reflexive Selection Test** - Test performance-driven routing

---

### 12. Arbiter & Worker Runtime Optimization Strategy

**Theory Section**: Lines 1070-1332  
**Key Concepts**:
- Multi-stage decision pipeline (3-stage lock-free)
- Arbiter performance budgets (<50ms decision latency)
- Worker precision optimization (INT8 + FP16)
- Graph optimization (ORT format, static shapes)
- Execution provider selection (Core ML vs MPS)
- Streaming task execution
- Bayesian optimization framework
- Apple Silicon-specific optimizations

**Test Coverage**:
- ✅ **`performance_scalability.rs`** - Tests general performance
- ❌ **Missing**: No tests for multi-stage pipeline
- ❌ **Missing**: No tests for precision optimization
- ❌ **Missing**: No tests for graph optimization
- ❌ **Missing**: No tests for Bayesian optimization
- ❌ **Missing**: No tests for Apple Silicon optimizations

**Test References**:
```rust
// performance_scalability.rs:1-100
test_resource_utilization() // Tests basic performance
test_sla_compliance() // Tests response time (partial)
```

**Missing Tests**:
- [ ] **Multi-Stage Pipeline Test** - Test 3-stage decision pipeline
- [ ] **Decision Latency Test** - Test <50ms arbiter latency
- [ ] **Precision Optimization Test** - Test INT8/FP16 quantization
- [ ] **Graph Optimization Test** - Test ORT format and static shapes
- [ ] **Provider Selection Test** - Test Core ML vs MPS heuristics
- [ ] **Streaming Execution Test** - Test chunked task execution
- [ ] **Bayesian Optimization Test** - Test auto-tuning framework
- [ ] **Apple Silicon Test** - Test ANE/Core ML/MPS optimizations

---

### 13. CoreML-First Architecture Rationale

**Theory Section**: Lines 1334-1386  
**Key Concepts**:
- CoreML Mistral as primary model
- ANE acceleration (2.8x speedup)
- Single model stack simplification
- CoreML-optimized models (7.5 MB FastViT T8 F16)
- Hardware-specific compilation
- Ollama removal

**Test Coverage**:
- ❌ **Missing**: No tests validate CoreML-first architecture
- ❌ **Missing**: No tests validate ANE acceleration
- ❌ **Missing**: No tests validate CoreML Mistral integration
- ❌ **Missing**: No tests validate Ollama removal

**Required Tests**:
- [ ] **CoreML Integration Test** - Verify CoreML Mistral loads and runs
- [ ] **ANE Acceleration Test** - Verify 2.8x speedup vs CPU
- [ ] **Single Model Stack Test** - Verify CoreML-only execution
- [ ] **Model Optimization Test** - Verify optimized model formats
- [ ] **Ollama Removal Test** - Verify no Ollama dependencies

---

## Test → Theory Mapping (Reverse)

### `caws_governance.rs`
**Maps to Theory Sections**:
- ✅ Section 1: CAWS Constitutional Authority (Lines 1-20)
- ✅ Section 8: CAWS-Compliant Arbitration Protocol (Lines 548-560)
- ✅ Section 6: Correctness and Traceability (Lines 92-110)

**Coverage**: **Good** - Comprehensive CAWS governance testing

---

### `integrated_playground_quality.rs`
**Maps to Theory Sections**:
- ✅ Section 1: CAWS governance (partial)
- ✅ Section 3: Orchestration and arbitration (partial)
- ✅ Section 6: Correctness and traceability (validation tests)
- ⚠️ Section 9: Arbiter reasoning (partial - quality evaluation)

**Coverage**: **Good** - Tests agent execution with quality gates

---

### `claim_verification.rs`
**Maps to Theory Sections**:
- ✅ Section 7: Claim Extraction Pipeline (Lines 112-546) - **Partial**
- ✅ Section 6: Correctness and traceability (factual validation)

**Coverage**: **Partial** - Tests extraction but not all 4 stages comprehensively

---

### `self_prompting_loops.rs`
**Maps to Theory Sections**:
- ✅ Section 3: Orchestration and arbitration (reflective iteration)
- ✅ Section 4: Model-agnostic design (hot-swapping)
- ✅ Section 10: Reflexive learning (iterative improvement)

**Coverage**: **Good** - Tests self-prompting and model swapping

---

### `reflexive_learning.rs`
**Maps to Theory Sections**:
- ✅ Section 4: Model-agnostic design (performance tracking)
- ✅ Section 10: Reflexive learning (Lines 580-715) - **Partial**
- ✅ Section 11: Model benchmarking (performance tracking)

**Coverage**: **Partial** - Tests learning but not all components

---

### `multi_agent_coordination.rs`
**Maps to Theory Sections**:
- ✅ Section 3: Orchestration and arbitration (Lines 34-46)
- ✅ Section 9: Arbiter reasoning engine (Lines 562-578) - **Partial**

**Coverage**: **Partial** - Tests coordination but not full CAWS Debate protocol

---

### `performance_scalability.rs`
**Maps to Theory Sections**:
- ⚠️ Section 5: Low-level implementation (performance) - **Partial**
- ⚠️ Section 11: Model benchmarking (performance metrics) - **Partial**
- ⚠️ Section 12: Runtime optimization (general performance) - **Partial**

**Coverage**: **Partial** - Tests performance but not optimization details

---

### `scenario_2_research.rs`
**Maps to Theory Sections**:
- ✅ Section 7: Research capabilities (KnowledgeSeeker)
- ⚠️ Section 7: Claim extraction (not tested)

**Coverage**: **Partial** - Tests research but not claim extraction

---

## Critical Test Gaps (Priority Order)

### 🔴 **Tier 1: Core Theory Validation (Blocking)**

1. **CAWS Adjudication Cycle** (Section 8)
   - **Theory**: Lines 548-560
   - **Status**: ❌ Missing - No full cycle test
   - **Impact**: High - Core governance mechanism
   - **Test Needed**: Full 5-stage cycle (Pleading → Examination → Deliberation → Verdict → Publication)

2. **Arbiter Reasoning Engine** (Section 9)
   - **Theory**: Lines 562-578
   - **Status**: ⚠️ Partial - Tests arbitration but not CAWS Debate
   - **Impact**: High - Core decision-making logic
   - **Test Needed**: CAWS Debate protocol, composite scoring formula

3. **Claim Extraction Pipeline** (Section 7)
   - **Theory**: Lines 112-546
   - **Status**: ⚠️ Partial - Tests extraction but not all 4 stages
   - **Impact**: High - Factual accuracy foundation
   - **Test Needed**: All 4 stages (Disambiguation, Qualification, Decomposition, Verification)

4. **CoreML-First Architecture** (Section 13)
   - **Theory**: Lines 1334-1386
   - **Status**: ❌ Missing - No CoreML validation tests
   - **Impact**: High - Primary execution path
   - **Test Needed**: CoreML integration, ANE acceleration, model optimization

### 🟡 **Tier 2: Important Theory Validation**

5. **Model-Agnostic Design** (Section 4)
   - **Theory**: Lines 48-65
   - **Status**: ⚠️ Partial - Tests hot-swap but not all features
   - **Impact**: Medium - System flexibility
   - **Test Needed**: Pluggable interfaces, speculative execution, hybrid routing

6. **Reflexive Learning** (Section 10)
   - **Theory**: Lines 580-715
   - **Status**: ⚠️ Partial - Tests learning but not all components
   - **Impact**: Medium - Continuous improvement
   - **Test Needed**: Federated learning, context offloading, curriculum learning

7. **Model Benchmarking** (Section 11)
   - **Theory**: Lines 716-1068
   - **Status**: ❌ Missing - No benchmarking tests
   - **Impact**: Medium - Model selection optimization
   - **Test Needed**: Benchmark cadence, scoring framework, lifecycle management

8. **Runtime Optimization** (Section 12)
   - **Theory**: Lines 1070-1332
   - **Status**: ❌ Missing - No optimization tests
   - **Impact**: Medium - Performance optimization
   - **Test Needed**: Multi-stage pipeline, precision optimization, Bayesian tuning

### 🟢 **Tier 3: Enhancement Validation**

9. **Hardware Optimization** (Section 2)
   - **Theory**: Lines 22-32
   - **Status**: ❌ Missing - No hardware-specific tests
   - **Impact**: Low - Performance enhancement
   - **Test Needed**: ANE utilization, Core ML acceleration, unified memory

10. **Low-Level Implementation** (Section 5)
    - **Theory**: Lines 67-90
    - **Status**: ⚠️ Partial - Tests performance but not implementation details
    - **Impact**: Low - Implementation validation
    - **Test Needed**: Rust efficiency, async concurrency, Core ML integration

---

## Recommended Test Implementation Plan

### Phase 1: Core Theory Validation (3-4 weeks)

**Week 1: CAWS Adjudication Cycle**
- [ ] Implement full 5-stage cycle test
- [ ] Test JSON RPC protocol
- [ ] Test Git integration with verdict trailers
- [ ] Test signed YAML verdicts

**Week 2: Arbiter Reasoning Engine**
- [ ] Implement CAWS Debate protocol test
- [ ] Test composite scoring formula (`S = 0.4E + 0.3B + 0.2G + 0.1P`)
- [ ] Test CoreML Mistral as judge
- [ ] Test evidence/budget/gate/provenance scoring

**Week 3: Claim Extraction Pipeline**
- [ ] Test Stage 1: Contextual Disambiguation
- [ ] Test Stage 2: Verifiable Content Qualification
- [ ] Test Stage 3: Precise Claim Decomposition
- [ ] Test Stage 4: CAWS-Compliant Verification
- [ ] Test end-to-end pipeline

**Week 4: CoreML-First Architecture**
- [ ] Test CoreML Mistral integration
- [ ] Test ANE acceleration (2.8x speedup)
- [ ] Test single model stack
- [ ] Test optimized model formats

### Phase 2: Important Theory Validation (2-3 weeks)

**Week 5: Model-Agnostic Design**
- [ ] Test pluggable model interfaces
- [ ] Test speculative execution
- [ ] Test hybrid routing (fast + accurate)
- [ ] Test performance-based preference

**Week 6: Reflexive Learning**
- [ ] Test federated learning
- [ ] Test context offloading
- [ ] Test curriculum learning
- [ ] Test failure mode detection

**Week 7: Model Benchmarking**
- [ ] Test micro-benchmark cadence
- [ ] Test macro-benchmark evaluation
- [ ] Test new model pipeline
- [ ] Test scoring framework

### Phase 3: Enhancement Validation (1-2 weeks)

**Week 8: Runtime Optimization**
- [ ] Test multi-stage pipeline
- [ ] Test precision optimization
- [ ] Test Bayesian auto-tuning
- [ ] Test Apple Silicon optimizations

**Week 9: Hardware & Implementation**
- [ ] Test ANE utilization
- [ ] Test Core ML acceleration
- [ ] Test Rust efficiency
- [ ] Test async concurrency

---

## Test Documentation Standards

### Test-to-Theory References

Each test should include comments referencing theory sections:

```rust
//! CAWS Adjudication Cycle Test
//!
//! Tests theory section: docs/arbiter/theory.md Lines 548-560
//! Validates:
//! - Pleading stage (worker submits diff + rationale)
//! - Examination stage (CAWS budget checks)
//! - Deliberation stage (gate metrics collection)
//! - Verdict stage (PASS/FAIL/WAIVER_REQUIRED)
//! - Publication stage (Git commit with CAWS-VERDICT-ID trailer)
```

### Theory Section Headers

Each theory section should have a corresponding test file or test section:

```rust
// Theory: docs/arbiter/theory.md Section 8 (Lines 548-560)
// CAWS-Compliant Arbitration Protocol
mod caws_adjudication_cycle {
    // Test implementations here
}
```

---

## Success Criteria

### Coverage Goals

- **Theory Coverage**: 100% of critical sections (Tier 1) tested
- **Test-to-Theory Mapping**: Every test references theory section
- **Theory-to-Test Mapping**: Every theory section has test coverage

### Quality Metrics

- **Test Completeness**: All theory concepts validated
- **Documentation**: All tests reference theory sections
- **Maintainability**: Theory changes trigger test updates

---

## Conclusion

**Current State**: Tests exist for many theory concepts, but **critical gaps** remain in:
- CAWS Adjudication Cycle (full 5-stage test)
- Arbiter Reasoning Engine (CAWS Debate protocol)
- Claim Extraction Pipeline (all 4 stages)
- CoreML-First Architecture (integration validation)

**Recommendation**: Implement Tier 1 tests (core theory validation) before considering the system production-ready. These tests validate the fundamental governance and decision-making mechanisms described in the theory document.

---

## Quick Reference: Theory → Test Mapping

| Theory Section | Lines | Test File | Coverage | Priority |
|---------------|-------|-----------|----------|----------|
| Overview & Goals | 1-20 | `caws_governance.rs` | ✅ Good | High |
| Hardware Performance | 22-32 | ❌ None | ❌ Missing | Low |
| Orchestration Model | 34-46 | `multi_agent_coordination.rs` | ⚠️ Partial | High |
| Model-Agnostic Design | 48-65 | `self_prompting_loops.rs` | ⚠️ Partial | Medium |
| Low-Level Implementation | 67-90 | `performance_scalability.rs` | ⚠️ Partial | Low |
| Correctness & Traceability | 92-110 | `caws_governance.rs` | ⚠️ Partial | High |
| Claim Extraction | 112-546 | `claim_verification.rs` | ⚠️ Partial | High |
| CAWS Arbitration Protocol | 548-560 | `caws_governance.rs` | ⚠️ Partial | High |
| Arbiter Reasoning Engine | 562-578 | `multi_agent_coordination.rs` | ⚠️ Partial | High |
| Reflexive Learning | 580-715 | `reflexive_learning.rs` | ⚠️ Partial | Medium |
| Model Benchmarking | 716-1068 | ❌ None | ❌ Missing | Medium |
| Runtime Optimization | 1070-1332 | ❌ None | ❌ Missing | Medium |
| CoreML Architecture | 1334-1386 | ❌ None | ❌ Missing | High |



