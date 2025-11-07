<!-- 4240a9b0-fd47-4210-9076-eb66d92f5b7e f496f8e5-39f0-4a99-9941-cb03eadeef1f -->
# Theory Alignment Completion Plan

## Overview

This plan addresses the critical gaps identified in the theory.md alignment assessment (78/100) to achieve functional completeness. The plan prioritizes high-impact integrations and automation that transform infrastructure into a fully operational system.

## Current State Assessment

**Strengths:**

- CAWS governance architecture implemented
- CoreML integration with ANE acceleration
- Comprehensive provenance system
- UnifiedOrchestrator coordinating components
- Rust implementation throughout

**Critical Gaps:**

1. Claim extraction not integrated into CAWS adjudication cycle
2. Continuous benchmarking not automated
3. Reflexive learning not connected to arbiter routing
4. MCP tools not fully discoverable by arbiter
5. Runtime optimization strategies not implemented

## Implementation Plan

### Phase 1: Claim Extraction Integration (Priority: High)

**Goal:** Integrate claim extraction pipeline into CAWS adjudication cycle for factual verification

**Files to Modify:**

- `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`
- `iterations/v3/agent-research/src/processor.rs`
- `iterations/v3/agent-orchestration/src/planning/caws_debate_scorer.rs`

**Implementation Steps:**

1. **Enable Verification Stage in ClaimExtractionProcessor**

- Remove TODOs in `agent-research/src/processor.rs` (lines 72-89)
- Integrate verification module from `agent-research`
- Implement CAWS-compliant verification stage
- Add unit tests for verification integration

2. **Integrate Claim Extraction into Examination Stage**

- Modify `stage_examination()` in `caws_adjudication_cycle.rs`
- Extract claims from worker artifacts using `ClaimExtractionProcessor`
- Verify claims against CAWS evidence requirements
- Pass verification results to deliberation stage

3. **Enhance Deliberation Stage with Claim-Based Scoring**

- Update `CawsDebateScorer` to incorporate claim verification scores
- Modify scoring formula: `S = 0.4E + 0.3B + 0.2G + 0.1P` where E includes claim verification
- Add claim-based evidence completeness scoring
- Integrate ambiguity resolution results into scoring

4. **Update Verdict Stage with Claim Verification Summary**

- Include claim extraction results in `FinalVerdictContract`
- Populate `verification_summary` field with claim metrics
- Add claim verification status to verdict reasoning

**Acceptance Criteria:**

- Claim extraction runs automatically during CAWS adjudication
- Verification results influence debate scoring
- Verdict includes claim verification summary
- All tests pass with claim extraction enabled

---

### Phase 2: Continuous Benchmarking Automation (Priority: High)

**Goal:** Implement automated continuous benchmarking system for model performance evaluation

**Files to Create:**

- `iterations/v3/agent-research/src/benchmarking/continuous_benchmarker.rs`
- `iterations/v3/agent-research/src/benchmarking/benchmark_scheduler.rs`
- `iterations/v3/agent-research/src/benchmarking/dataset_manager.rs`

**Files to Modify:**

- `iterations/v3/agent-research/src/lib.rs`
- `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`

**Implementation Steps:**

1. **Create Benchmark Scheduler**

- Implement `BenchmarkScheduler` with cadence management (daily/weekly/monthly)
- Add cron-like scheduling for micro/macro benchmarks
- Create benchmark task queue system
- Integrate with existing benchmark infrastructure

2. **Implement Dataset Manager**

- Create `BenchmarkDataset` manager for task surface datasets
- Implement dataset versioning and archival
- Add dataset validation and quality checks
- Support rolling 12-month dataset retention

3. **Build Continuous Benchmarker**

- Implement `ContinuousBenchmarker` orchestrating benchmark execution
- Add performance score calculation using multi-dimensional framework
- Integrate with `ModelPerformanceTracker` in data-infrastructure
- Store benchmark results in database with versioning

4. **Integrate Benchmark Results into Worker Assignment**

- Modify `WorkerAssignmentStrategy` to query benchmark scores
- Update `evaluate_candidates()` to use benchmark performance data
- Add performance-driven model selection based on benchmark results
- Implement exploration-exploitation balance for model selection

5. **Add Benchmark Observability**

- Create benchmark dashboard integration points
- Add alerting for performance degradation
- Implement benchmark report generation
- Add CI/CD integration for automated benchmarking

**Acceptance Criteria:**

- Daily micro-benchmarks run automatically
- Weekly macro-benchmarks execute on schedule
- Benchmark results influence worker assignment decisions
- Performance degradation alerts trigger automatically
- Benchmark reports generated and stored

---

### Phase 3: Reflexive Learning Integration (Priority: High)

**Goal:** Connect reflexive learning infrastructure to arbiter routing decisions

**Files to Modify:**

- `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`
- `iterations/v3/agent-workers/src/learning/adaptive_selector.rs`
- `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`

**Files to Create:**

- `iterations/v3/agent-orchestration/src/planning/reflexive_learner.rs`

**Implementation Steps:**

1. **Create ReflexiveLearner Component**

- Implement `ReflexiveLearner` struct with learning loop
- Add `learnFromOutcomes()` method processing completed tasks
- Implement `routeWithLearning()` using historical performance
- Add `adjustTaskDifficulty()` for curriculum-based progression

2. **Integrate Learning into Worker Assignment**

- Modify `WorkerAssignmentStrategy` to use `ReflexiveLearner`
- Update `evaluate_candidates()` to incorporate learning signals
- Add performance-based routing adjustments
- Implement credit assignment for multi-turn tasks

3. **Connect Learning to UnifiedOrchestrator**

- Add reflexive learning hooks to `execute_plan()` method
- Call `learnFromOutcomes()` after each milestone completion
- Update routing decisions based on learning updates
- Persist learning state across orchestrator restarts

4. **Implement Progress Tracking Integration**

- Connect turn-level progress tracking to learning system
- Add trajectory analysis for long-horizon tasks
- Implement rubric engineering for performance evaluation
- Add failure mode detection and mitigation

5. **Add Learning Observability**

- Track learning update frequency and impact
- Monitor routing decision quality improvements
- Add learning metrics to provenance system
- Create learning dashboard integration points

**Acceptance Criteria:**

- Learning updates occur after each task completion
- Routing decisions improve over time based on learning
- Task difficulty adjusts based on worker performance
- Learning state persists across orchestrator restarts
- Learning metrics visible in observability dashboards

---

### Phase 4: MCP-CAWS Tool Discovery Integration (Priority: Medium)

**Goal:** Complete MCP server integration with CAWS governance for dynamic tool discovery

**Files to Modify:**

- `iterations/v3/agent-mcp/src/server.rs`
- `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`
- `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`

**Files to Create:**

- `iterations/v3/agent-mcp/src/caws_tool_registry.rs`

**Implementation Steps:**

1. **Create CAWS Tool Registry**

- Implement `CawsToolRegistry` managing CAWS tool discovery
- Add tool registration and discovery APIs
- Integrate with existing MCP tool system
- Add tool capability metadata and CAWS compliance tags

2. **Enhance MCP Server with CAWS Tools**

- Register CAWS governance tools (validation, auditing, waiver management)
- Expose CAWS artifacts as MCP resources (working specs, provenance logs)
- Add tool discovery protocol for runtime tool querying
- Implement tool execution with CAWS compliance checking

3. **Integrate MCP Tools into Adjudication Cycle**

- Modify `CawsAdjudicationCycle` to discover tools via MCP
- Use MCP tools for CAWS validation during examination stage
- Invoke MCP tools for waiver management during deliberation
- Add tool execution results to provenance trail

4. **Connect MCP Tools to UnifiedOrchestrator**

- Add MCP tool discovery to orchestrator initialization
- Enable workers to discover and invoke CAWS tools via MCP
- Add tool usage tracking and analytics
- Integrate tool results into decision-making

5. **Add Tool Observability**

- Track tool discovery and usage patterns
- Monitor tool execution performance
- Add tool compliance metrics
- Create tool usage dashboard

**Acceptance Criteria:**

- CAWS tools discoverable via MCP protocol
- Tools invoked automatically during adjudication cycle
- Tool execution results influence CAWS decisions
- Tool usage tracked in provenance system
- Dynamic tool discovery works at runtime

---

### Phase 5: Runtime Optimization Strategies (Priority: Medium)

**Goal:** Implement advanced runtime optimization strategies from theory.md

**Files to Create:**

- `iterations/v3/agent-orchestration/src/optimization/multi_stage_pipeline.rs`
- `iterations/v3/agent-orchestration/src/optimization/auto_tuner.rs`
- `iterations/v3/agent-orchestration/src/optimization/streaming_executor.rs`

**Files to Modify:**

- `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`
- `iterations/v3/engine-coreml/src/lib.rs`

**Implementation Steps:**

1. **Implement Multi-Stage Decision Pipeline**

- Create `MultiStagePipeline` with fast-path classification (<50ms)
- Add worker selection optimization stage
- Implement dual-execution orchestration (primary + pre-compute)
- Add backpressure-aware concurrency control

2. **Build Auto-Tuner Framework**

- Implement `AutoTuner` with Bayesian optimization
- Define parameter space (chunk size, concurrency, memory)
- Add multi-objective optimization (latency, throughput, CAWS compliance)
- Create continuous optimization loop

3. **Add Streaming Task Execution**

- Implement `StreamingTaskExecutor` with chunked execution
- Add pre-computation phase for task preparation
- Create continuous execution state maintenance
- Add resumable task execution support

4. **Enhance CoreML Optimization**

- Add precision optimization profiles (INT8, FP16)
- Implement graph optimization (static shapes, operator fusion)
- Add provider selection heuristics (ANE vs MPS vs CPU)
- Create quality-preserving fallback mechanisms

5. **Add Optimization Observability**

- Track optimization parameter changes
- Monitor performance improvements
- Add optimization impact metrics
- Create optimization dashboard

**Acceptance Criteria:**

- Decision pipeline latency <50ms
- Auto-tuning improves performance over time
- Streaming execution reduces end-to-end latency
- CoreML optimizations maintain quality thresholds
- Optimization metrics visible in dashboards

---

## Success Metrics

**Phase 1 (Claim Extraction):**

- Claim extraction runs in 100% of adjudication cycles
- Verification results influence scoring in 100% of deliberations
- Verdict includes claim verification summary

**Phase 2 (Benchmarking):**

- Daily micro-benchmarks execute automatically
- Weekly macro-benchmarks complete successfully
- Benchmark results influence 80%+ of worker assignments

**Phase 3 (Reflexive Learning):**

- Learning updates occur after each task completion
- Routing accuracy improves 10%+ over baseline
- Task difficulty adjusts based on performance

**Phase 4 (MCP Integration):**

- CAWS tools discoverable via MCP protocol
- Tools invoked in 100% of adjudication cycles
- Tool usage tracked in provenance

**Phase 5 (Optimization):**

- Decision latency <50ms (from ~100ms)
- Throughput improves 2-4x while maintaining CAWS compliance
- End-to-end task completion 40% faster

## Dependencies

- Claim extraction verification module completion
- Benchmark dataset creation and validation
- Learning infrastructure components operational
- MCP server tool registration system
- CoreML optimization profiles defined

## Risk Mitigation

- **Claim Extraction:** Verification module may need additional work - create fallback to basic verification
- **Benchmarking:** Dataset quality critical - implement validation gates before benchmark execution
- **Reflexive Learning:** Learning may degrade performance initially - add performance monitoring and rollback
- **MCP Integration:** Tool discovery overhead - optimize discovery protocol for low latency
- **Optimization:** Quality preservation critical - implement quality gates before applying optimizations

## Timeline Estimate

- **Phase 1:** 1-2 weeks
- **Phase 2:** 2-3 weeks
- **Phase 3:** 2-3 weeks
- **Phase 4:** 1-2 weeks
- **Phase 5:** 2-3 weeks

**Total:** 8-13 weeks for complete implementation

### To-dos

- [ ] Integrate claim extraction pipeline into CAWS adjudication cycle examination and deliberation stages
- [ ] Enable verification stage in ClaimExtractionProcessor and integrate with CAWS compliance checking
- [ ] Create BenchmarkScheduler with daily/weekly/monthly cadence management and task queue system
- [ ] Implement BenchmarkDataset manager with versioning, validation, and rolling retention
- [ ] Integrate benchmark results into WorkerAssignmentStrategy for performance-driven model selection
- [ ] Create ReflexiveLearner component with learning loop, outcome processing, and routing adjustments
- [ ] Connect ReflexiveLearner to UnifiedOrchestrator for automatic learning updates after task completion
- [ ] Create CawsToolRegistry for dynamic CAWS tool discovery and registration via MCP protocol
- [ ] Integrate MCP tool discovery into CawsAdjudicationCycle for automatic tool invocation during adjudication
- [ ] Implement MultiStagePipeline with fast-path classification, worker selection optimization, and dual-execution
- [ ] Build AutoTuner with Bayesian optimization for continuous parameter tuning
- [ ] Implement StreamingTaskExecutor with chunked execution and resumable task support