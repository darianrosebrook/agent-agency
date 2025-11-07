<!-- 67483c44-3278-459e-8cb0-bfb8f940c45d 36a0a834-99c1-4356-b039-c8cbd5b25228 -->
# V3 Arbiter Stack Integration Plan

## Executive Summary

The v3 implementation has **~85% of required functionality** already built, but components need to be connected into a cohesive flow. This plan identifies existing components, missing pieces, and the specific connection points needed to achieve the arbiter stack described in `theory.md`.

## Current State Assessment

### ✅ Fully Implemented Components

1. **CoreML Mistral Inference** (`engine-coreml`)

   - ✅ Real Mistral inference via ANE acceleration
   - ✅ Prompt caching with Blake3 hashing
   - ✅ RCU-safe model hot-swapping infrastructure
   - ✅ JSON schema validation
   - **Status**: Production-ready

2. **Constitutional Council** (`agent-constitutional-council`)

   - ✅ Four specialized judges (Constitutional, Technical, Quality, Integration)
   - ✅ Hybrid reasoning (deterministic + LLM)
   - ✅ CoreML integration via `JudgeEngine` trait
   - ✅ Verdict aggregation and decision making
   - **Status**: Production-ready

3. **CAWS Adjudication Cycle** (`agent-orchestration/src/planning/caws_adjudication_cycle.rs`)

   - ✅ All 5 stages implemented (Pleading, Examination, Deliberation, Verdict, Publication)
   - ✅ Claim extraction integration (conditional on `research` feature)
   - ✅ Quality gates integration (conditional)
   - ✅ CAWS tool registry integration (conditional on `mcp` feature)
   - ✅ Git verdict publication with `CAWS-VERDICT-ID` trailer
   - **Status**: Implemented but some features conditional

4. **Unified Orchestrator** (`agent-orchestration/src/orchestration/unified_orchestrator.rs`)

   - ✅ Complete execution flow (plan generation → execution → council review → refinement → merge)
   - ✅ Git worktree management integration
   - ✅ Memory system integration
   - ✅ Session management
   - ✅ State persistence and crash recovery
   - ✅ Turn-level progress tracking
   - ✅ Federated learning integration
   - **Status**: Comprehensive implementation

5. **Git Worktree Management** (`agent-orchestration/src/planning/worktree_manager.rs`)

   - ✅ Worktree creation per worker
   - ✅ Worktree merging with conflict detection
   - ✅ Worktree cleanup
   - ✅ Parallel worktree isolation
   - **Status**: Production-ready

6. **Claim Extraction** (`agent-research/src/disambiguation/patterns.rs`, `agent-research/src/verification/verifier.rs`)

   - ✅ Four-stage claim processing pipeline
   - ✅ Contextual disambiguation
   - ✅ Verifiable content qualification
   - ✅ Atomic claim decomposition
   - ✅ CAWS-compliant verification
   - **Status**: Implemented but integration conditional

7. **Reflexive Learning** (`agent-orchestration/src/planning/reflexive_learner.rs`)

   - ✅ Learning outcome processing
   - ✅ Pattern analysis and routing adjustments
   - ✅ Worker performance tracking
   - **Status**: Implemented but not fully connected to routing

8. **Model Performance Tracking** (`agent-research/src/performance_tracker.rs`, `agent-research/src/metrics_collector.rs`)

   - ✅ Benchmark result storage
   - ✅ Performance history tracking
   - ✅ Model performance summaries
   - ✅ Continuous benchmarking infrastructure
   - **Status**: Implemented but not connected to routing decisions

9. **CAWS Quality Gates** (`agent-orchestration/src/planning/caws_quality_gates.rs`)

   - ✅ Waiver-aware gate execution
   - ✅ Violation detection and reporting
   - ✅ Integration with adjudication cycle
   - **Status**: Implemented

10. **Provenance Tracking** (`data-infrastructure/src/models.rs`, `system-quality-security/src/provenance_types.rs`)

    - ✅ Comprehensive provenance data structures
    - ✅ Database persistence
    - ✅ Audit trail tracking
    - **Status**: Implemented

### ⚠️ Partially Implemented / Missing Connections

1. **Model Performance → Routing Integration**

   - ❌ Performance tracker not consulted during worker assignment
   - ❌ No performance-based model selection
   - ❌ Benchmark results not feeding into routing decisions
   - **Location**: `agent-orchestration/src/planning/worker_assignment.rs`

2. **Reflexive Learning → Worker Assignment**

   - ⚠️ Reflexive learner exists but adjustments not applied to assignment strategy
   - ⚠️ Learning outcomes not feeding back into worker selection
   - **Location**: `agent-orchestration/src/planning/reflexive_learner.rs` → `worker_assignment.rs`

3. **Claim Extraction Integration**

   - ⚠️ Claim extraction exists but only enabled with `research` feature flag
   - ⚠️ Should be always-on for examination stage
   - **Location**: `agent-orchestration/src/planning/caws_adjudication_cycle.rs:383`

4. **CAWS MCP Tool Registry**

   - ⚠️ Tool registry exists but only enabled with `mcp` feature flag
   - ⚠️ Tool discovery happens but tools not invoked
   - **Location**: `agent-orchestration/src/planning/caws_adjudication_cycle.rs:284`

5. **Model Hot-Swapping**

   - ⚠️ Infrastructure exists (`engine-coreml` RCU-safe swapping)
   - ❌ No automatic hot-swapping based on performance
   - ❌ No model lifecycle management integration
   - **Location**: `engine-coreml/src/lib.rs`

6. **Model Debate Protocol**

   - ⚠️ CAWS debate scoring exists (`caws_debate_scorer.rs`)
   - ⚠️ Integrated into deliberation stage
   - ⚠️ Needs verification that it's fully functional
   - **Location**: `agent-orchestration/src/planning/caws_adjudication_cycle.rs:504`

7. **End-to-End Flow Verification**

   - ⚠️ All components exist but need integration test
   - ⚠️ API server → orchestrator → workers → council → merge flow needs verification
   - **Location**: `data-interfaces-adapters/src/bin/api-server.rs` → `unified_orchestrator.rs`

## Required Connection Points

### Connection Point 1: Model Performance → Worker Routing

**Current State**: `worker_assignment.rs` uses capability matching but doesn't consult performance data.

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`
- Add `PerformanceTracker` dependency to `WorkerAssignmentStrategy`
- In `assign_worker()`, query performance tracker for model scores
- Apply performance-based weighting to worker selection
- Use composite score: `0.4 * capability_match + 0.3 * performance_score + 0.2 * availability + 0.1 * historical_success`

**Integration Code**:

```rust
// In WorkerAssignmentStrategy::assign_worker()
let performance_scores = self.performance_tracker
    .get_model_performance_scores(&available_workers)
    .await?;

// Weight worker selection by performance
let weighted_workers = workers.iter()
    .map(|w| {
        let perf_score = performance_scores.get(&w.id).unwrap_or(&0.5);
        (w, perf_score)
    })
    .sorted_by(|a, b| b.1.partial_cmp(a.1).unwrap())
    .collect();
```

### Connection Point 2: Reflexive Learning → Worker Assignment

**Current State**: `ReflexiveLearner` processes outcomes but adjustments aren't applied.

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/reflexive_learner.rs`
- Ensure `apply_adjustment()` actually updates `WorkerAssignmentStrategy`
- Connect learning outcomes to assignment strategy's routing weights
- File: `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`
- Add method to accept routing adjustments from learner

**Integration Code**:

```rust
// In ReflexiveLearner::apply_adjustment()
async fn apply_adjustment(&self, adjustment: &RoutingAdjustment) -> Result<()> {
    match adjustment {
        RoutingAdjustment::PreferWorker { worker_id, reason } => {
            self.worker_assignment_strategy
                .set_worker_preference(*worker_id, 1.2) // 20% boost
                .await?;
        }
        RoutingAdjustment::AvoidWorker { worker_id, reason } => {
            self.worker_assignment_strategy
                .set_worker_preference(*worker_id, 0.8) // 20% penalty
                .await?;
        }
        // ... other adjustments
    }
    Ok(())
}
```

### Connection Point 3: Claim Extraction Always-On

**Current State**: Claim extraction only runs if `research` feature enabled.

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs:383`
- Remove feature flag conditional
- Ensure claim extractor is always initialized
- Make claim extraction a required step in examination stage

**Integration Code**:

```rust
// Remove #[cfg(feature = "research")] conditional
// Always initialize claim extractor
let claim_extractor = Arc::new(agent_research::ClaimExtractionProcessor::new());

// In stage_examination(), always run claim extraction
let extraction_result = claim_extractor.run(&text_content, &processing_context).await?;
```

### Connection Point 4: CAWS MCP Tool Invocation

**Current State**: Tools are discovered but not invoked.

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs:310`
- Actually invoke discovered tools instead of just logging
- Use tool results in examination stage validation
- Integrate tool outputs into claim verification

**Integration Code**:

```rust
// In stage_examination(), after tool discovery
for tool in &compliance_tools {
    let tool_result = registry.invoke_tool(
        &tool.tool_id,
        &tool.input_schema,
        &artifact_data
    ).await?;
    
    // Use tool result for validation
    if !tool_result.passed {
        return Err(anyhow!("Compliance check failed: {}", tool_result.message));
    }
}
```

### Connection Point 5: Model Hot-Swapping Integration

**Current State**: Hot-swapping infrastructure exists but not triggered by performance.

**Required Changes**:

- File: `iterations/v3/engine-coreml/src/lib.rs`
- Add method to trigger model swap based on performance threshold
- File: `iterations/v3/agent-orchestration/src/planning/model_lifecycle.rs` (NEW)
- Create model lifecycle manager that monitors performance
- Trigger hot-swaps when performance degrades below threshold

**Integration Code**:

```rust
// New file: agent-orchestration/src/planning/model_lifecycle.rs
pub struct ModelLifecycleManager {
    performance_tracker: Arc<PerformanceTracker>,
    coreml_manager: Arc<CoreMLManager>,
}

impl ModelLifecycleManager {
    async fn check_and_swap_if_needed(&self) -> Result<()> {
        let current_performance = self.performance_tracker
            .get_current_performance().await?;
        
        if current_performance.composite_score < 0.7 {
            // Performance degraded - swap to better model
            let better_model = self.find_better_model().await?;
            self.coreml_manager.swap_model(better_model).await?;
        }
        Ok(())
    }
}
```

### Connection Point 6: API Server → Unified Orchestrator

**Current State**: API server exists but connection to `UnifiedOrchestrator` needs verification.

**Required Changes**:

- File: `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`
- Verify `UnifiedOrchestrator` is initialized and accessible
- Ensure task submission routes to orchestrator
- Verify response handling and error propagation

**Integration Verification**:

```rust
// In api-server.rs, verify orchestrator initialization
let orchestrator = UnifiedOrchestrator::new(
    config,
    plan_generator,
    plan_executor,
    parallel_coordinator,
    council,
    worker_bridge,
    // ... all dependencies
).await?;

// In task submission handler
async fn submit_task(State(state): State<AppState>, Json(spec): Json<WorkingSpec>) {
    let result = state.orchestrator.execute_plan(spec).await?;
    // Return result
}
```

### Connection Point 7: End-to-End Flow Verification

**Required**: Create integration test that verifies complete flow.

**Test File**: `iterations/v3/agent-orchestration/tests/integration_e2e_flow.rs` (NEW)

**Test Flow**:

1. Submit task via API
2. Verify plan generation
3. Verify council plan review
4. Verify worker assignment
5. Verify worktree creation
6. Verify milestone execution
7. Verify council presentation
8. Verify CAWS adjudication cycle
9. Verify claim extraction
10. Verify quality gates
11. Verify refinement loop (if needed)
12. Verify worktree merge
13. Verify provenance tracking

## Implementation Priority

### Priority 1: Critical Connections (Required for Basic Functionality)

1. **API Server → Unified Orchestrator** (Connection Point 6)

   - **Effort**: 2-4 hours
   - **Impact**: Enables task submission and execution

2. **Claim Extraction Always-On** (Connection Point 3)

   - **Effort**: 1-2 hours
   - **Impact**: Ensures factual verification in examination stage

3. **End-to-End Flow Verification** (Connection Point 7)

   - **Effort**: 4-6 hours
   - **Impact**: Validates entire system works together

### Priority 2: Performance Optimization (Required for Theory.md Compliance)

4. **Model Performance → Routing** (Connection Point 1)

   - **Effort**: 3-4 hours
   - **Impact**: Enables performance-based model selection

5. **Reflexive Learning → Assignment** (Connection Point 2)

   - **Effort**: 2-3 hours
   - **Impact**: Enables continuous learning and adaptation

6. **CAWS MCP Tool Invocation** (Connection Point 4)

   - **Effort**: 3-4 hours
   - **Impact**: Enables dynamic tool discovery and usage

### Priority 3: Advanced Features (Nice-to-Have)

7. **Model Hot-Swapping Integration** (Connection Point 5)

   - **Effort**: 4-6 hours
   - **Impact**: Enables automatic model optimization

## Files to Modify

### High Priority

1. `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`

   - Verify orchestrator initialization
   - Ensure task routing works

2. `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`

   - Remove feature flag from claim extraction
   - Implement tool invocation

3. `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`

   - Add performance tracker integration
   - Add reflexive learning integration

4. `iterations/v3/agent-orchestration/src/planning/reflexive_learner.rs`

   - Ensure adjustments are applied

### Medium Priority

5. `iterations/v3/agent-orchestration/src/planning/model_lifecycle.rs` (NEW)

   - Create model lifecycle manager

6. `iterations/v3/agent-orchestration/tests/integration_e2e_flow.rs` (NEW)

   - Create end-to-end integration test

## Testing Strategy

1. **Unit Tests**: Verify each connection point independently
2. **Integration Tests**: Verify component interactions
3. **End-to-End Test**: Verify complete flow from API to merge
4. **Performance Tests**: Verify model routing improves over time
5. **Claim Verification Tests**: Verify claim extraction accuracy

## Success Criteria

- [ ] Task can be submitted via API and completes successfully
- [ ] Council adjudication cycle executes all 5 stages
- [ ] Claim extraction runs in examination stage
- [ ] Quality gates execute with waiver recognition
- [ ] Worker assignment considers performance data
- [ ] Reflexive learning updates routing decisions
- [ ] Worktrees are created, used, and merged correctly
- [ ] Provenance is tracked throughout execution
- [ ] End-to-end test passes

## Estimated Total Effort

- **Priority 1**: 7-12 hours
- **Priority 2**: 8-11 hours  
- **Priority 3**: 4-6 hours
- **Total**: 19-29 hours (2.5-4 days)

## Next Steps

1. Start with Priority 1 connections (critical path)
2. Verify end-to-end flow works
3. Add Priority 2 optimizations
4. Add Priority 3 advanced features
5. Comprehensive testing and validation

### To-dos

- [ ] Connect API server to UnifiedOrchestrator - initialize orchestrator with all dependencies and route task submission
- [ ] Remove feature flag from claim extraction - make it always-on in CAWS adjudication cycle examination stage
- [ ] Implement actual tool invocation in CAWS adjudication cycle - invoke discovered tools and use results for validation
- [ ] Connect performance tracker to worker routing - ensure performance scores are always consulted in worker assignment
- [ ] Connect reflexive learning adjustments to worker assignment - ensure learning outcomes update routing decisions
- [ ] Create model lifecycle manager - monitor performance and trigger hot-swaps when performance degrades
- [ ] Create end-to-end integration test - verify complete flow from API submission through council review to merge