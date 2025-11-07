<!-- 67483c44-3278-459e-8cb0-bfb8f940c45d 96db41b3-8d41-49b1-8471-46383707a2b7 -->
# V3 Arbiter Stack Integration Plan

## Executive Summary

The v3 implementation has **~90% of required functionality** already built, but critical connection points are missing or behind feature flags. This plan identifies the specific gaps and connection points needed to achieve the arbiter stack described in `theory.md`.

## Current State Assessment

### Fully Implemented Components

1. **CoreML Mistral Inference** (`engine-coreml`) - Production-ready
2. **Constitutional Council** (`agent-constitutional-council`) - Production-ready  
3. **CAWS Adjudication Cycle** - All 5 stages implemented
4. **Unified Orchestrator** - Complete execution flow implemented
5. **Git Worktree Management** - Production-ready
6. **Claim Extraction** - Four-stage pipeline implemented
7. **Reflexive Learning** - Learning and adjustment logic implemented
8. **Model Performance Tracking** - Benchmark infrastructure complete
9. **CAWS Quality Gates** - Waiver-aware execution implemented
10. **Provenance Tracking** - Database persistence complete

### Critical Gaps Identified

1. **API Server → UnifiedOrchestrator Connection**: API server uses `OrchestratorService` wrapper, not `UnifiedOrchestrator` directly
2. **Feature Flag Dependencies**: Claim extraction and performance tracking behind feature flags
3. **MCP Tool Invocation**: Tools discovered but not actually invoked
4. **Performance Tracker Integration**: Exists but only used when `research` feature enabled
5. **Model Hot-Swapping**: Infrastructure exists but no lifecycle manager triggers swaps

## Required Connection Points

### Connection Point 1: API Server → UnifiedOrchestrator Bridge

**Current State**: API server uses `OrchestratorService` which wraps `LegacyOrchestratorAdapter`, not `UnifiedOrchestrator`.

**Required Changes**:

- File: `iterations/v3/data-interfaces-adapters/src/orchestration_adapter.rs`
- Create new adapter that wraps `UnifiedOrchestrator` instead of `LegacyOrchestratorAdapter`
- File: `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`
- Initialize `UnifiedOrchestrator` with all dependencies and connect to API handlers

**Implementation Steps**:

1. Create `UnifiedOrchestratorAdapter` in `orchestration_adapter.rs`
2. Initialize `UnifiedOrchestrator` in `api-server.rs` main() with all required dependencies
3. Update `OrchestratorService` to use `UnifiedOrchestratorAdapter`
4. Verify task submission routes through unified orchestrator

### Connection Point 2: Claim Extraction Always-On

**Current State**: Claim extraction only runs if `research` feature enabled (line 383 in `caws_adjudication_cycle.rs`).

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`
- Remove `#[cfg(feature = "research")]` conditional around claim extraction
- Ensure claim extractor is always initialized in `new()` method
- Make claim extraction required in examination stage

**Implementation Steps**:

1. Remove feature flag conditional at line 383
2. Ensure claim extractor initialization doesn't require feature flag
3. Update `CawsAdjudicationCycle::new()` to always create claim extractor
4. Verify claim extraction runs in all examination stages

### Connection Point 3: Performance Tracker Always-On

**Current State**: Performance tracker only consulted when `research` feature enabled (line 1018 in `worker_assignment.rs`).

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`
- Remove `#[cfg(feature = "research")]` conditional around performance tracker usage
- Ensure performance tracker is always initialized when available
- Make performance scores always factor into worker selection

**Implementation Steps**:

1. Remove feature flag conditional at line 1016-1039
2. Ensure `performance_tracker` field is always available (not feature-gated)
3. Update `get_performance_score()` to always use tracker when present
4. Verify performance scores factor into worker assignment

### Connection Point 4: CAWS MCP Tool Invocation

**Current State**: Tools are discovered but only logged, not invoked (lines 310-320 in `caws_adjudication_cycle.rs`).

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`
- Actually invoke discovered tools instead of just logging
- Use tool results for validation in examination stage
- Integrate tool outputs into claim verification

**Implementation Steps**:

1. Add tool invocation logic after tool discovery (line 310)
2. Call `registry.invoke_tool()` for each compliance/quality tool
3. Validate tool results and fail examination if tools report violations
4. Integrate tool outputs into claim verification results

### Connection Point 5: Reflexive Learning → Worker Assignment Verification

**Current State**: Reflexive learner applies adjustments via `update_worker_performance()` (line 519 in `reflexive_learner.rs`), but needs verification that this actually affects routing.

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/reflexive_learner.rs`
- Verify `apply_adjustment()` correctly updates worker performance cache
- File: `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`
- Ensure performance cache updates immediately affect `get_performance_score()`

**Implementation Steps**:

1. Verify performance cache is shared between reflexive learner and worker assignment
2. Ensure cache updates are immediately visible to `get_performance_score()`
3. Add integration test to verify learning outcomes affect routing
4. Verify capability adjustments are applied correctly

### Connection Point 6: Model Hot-Swapping Lifecycle Manager

**Current State**: Hot-swapping infrastructure exists in `engine-coreml` but no lifecycle manager triggers swaps.

**Required Changes**:

- File: `iterations/v3/agent-orchestration/src/planning/model_lifecycle.rs` (NEW)
- Create model lifecycle manager that monitors performance
- Trigger hot-swaps when performance degrades below threshold
- File: `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`
- Integrate lifecycle manager into unified orchestrator

**Implementation Steps**:

1. Create `ModelLifecycleManager` struct
2. Add performance monitoring loop
3. Implement threshold-based swap logic
4. Integrate into `UnifiedOrchestrator` initialization
5. Add periodic performance checks

### Connection Point 7: End-to-End Flow Integration Test

**Required**: Create comprehensive integration test that verifies complete flow.

**Test File**: `iterations/v3/agent-orchestration/tests/integration_e2e_flow.rs` (NEW)

**Test Flow**:

1. Initialize all components (orchestrator, council, workers, etc.)
2. Submit task via API
3. Verify plan generation
4. Verify council plan review
5. Verify worker assignment (with performance consideration)
6. Verify worktree creation
7. Verify milestone execution
8. Verify council presentation
9. Verify CAWS adjudication cycle (all 5 stages)
10. Verify claim extraction runs
11. Verify quality gates execute
12. Verify refinement loop (if needed)
13. Verify worktree merge
14. Verify provenance tracking
15. Verify reflexive learning processes outcomes

## Implementation Priority

### Priority 1: Critical Path (Required for Basic Functionality)

1. **API Server → UnifiedOrchestrator Bridge** (Connection Point 1)

- **Effort**: 4-6 hours
- **Impact**: Enables task submission and execution
- **Files**: `orchestration_adapter.rs`, `api-server.rs`

2. **Claim Extraction Always-On** (Connection Point 2)

- **Effort**: 1-2 hours
- **Impact**: Ensures factual verification in examination stage
- **Files**: `caws_adjudication_cycle.rs`

3. **End-to-End Flow Test** (Connection Point 7)

- **Effort**: 4-6 hours
- **Impact**: Validates entire system works together
- **Files**: `tests/integration_e2e_flow.rs` (NEW)

### Priority 2: Performance Optimization (Required for Theory.md Compliance)

4. **Performance Tracker Always-On** (Connection Point 3)

- **Effort**: 1-2 hours
- **Impact**: Enables performance-based model selection
- **Files**: `worker_assignment.rs`

5. **CAWS MCP Tool Invocation** (Connection Point 4)

- **Effort**: 3-4 hours
- **Impact**: Enables dynamic tool discovery and usage
- **Files**: `caws_adjudication_cycle.rs`, `caws_tool_registry.rs`

6. **Reflexive Learning Verification** (Connection Point 5)

- **Effort**: 2-3 hours
- **Impact**: Ensures learning outcomes update routing
- **Files**: `reflexive_learner.rs`, `worker_assignment.rs`

### Priority 3: Advanced Features (Nice-to-Have)

7. **Model Hot-Swapping Lifecycle Manager** (Connection Point 6)

- **Effort**: 4-6 hours
- **Impact**: Enables automatic model optimization
- **Files**: `model_lifecycle.rs` (NEW), `unified_orchestrator.rs`

## Files to Modify

### High Priority

1. `iterations/v3/data-interfaces-adapters/src/orchestration_adapter.rs`

- Create `UnifiedOrchestratorAdapter`
- Bridge `UnifiedOrchestrator` to `OrchestrationService` trait

2. `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs`

- Initialize `UnifiedOrchestrator` with all dependencies
- Connect to task submission handlers

3. `iterations/v3/agent-orchestration/src/planning/caws_adjudication_cycle.rs`

- Remove feature flag from claim extraction (line 383)
- Implement tool invocation (lines 310-320)

4. `iterations/v3/agent-orchestration/src/planning/worker_assignment.rs`

- Remove feature flag from performance tracker (line 1016)

### Medium Priority

5. `iterations/v3/agent-orchestration/src/planning/reflexive_learner.rs`

- Verify adjustment application works correctly

6. `iterations/v3/agent-orchestration/src/planning/model_lifecycle.rs` (NEW)

- Create model lifecycle manager

7. `iterations/v3/agent-orchestration/tests/integration_e2e_flow.rs` (NEW)

- Create end-to-end integration test

## Testing Strategy

1. **Unit Tests**: Verify each connection point independently
2. **Integration Tests**: Verify component interactions
3. **End-to-End Test**: Verify complete flow from API to merge
4. **Performance Tests**: Verify model routing improves over time
5. **Claim Verification Tests**: Verify claim extraction accuracy

## Success Criteria

- [ ] Task can be submitted via API and completes successfully
- [ ] UnifiedOrchestrator is initialized and accessible from API
- [ ] Council adjudication cycle executes all 5 stages
- [ ] Claim extraction runs in examination stage (always-on)
- [ ] Quality gates execute with waiver recognition
- [ ] Worker assignment considers performance data (always-on)
- [ ] Reflexive learning updates routing decisions
- [ ] CAWS MCP tools are invoked and results used
- [ ] Worktrees are created, used, and merged correctly
- [ ] Provenance is tracked throughout execution
- [ ] End-to-end test passes

## Estimated Total Effort

- **Priority 1**: 9-14 hours (1.5-2 days)
- **Priority 2**: 6-9 hours (1 day)
- **Priority 3**: 4-6 hours (0.5-1 day)
- **Total**: 19-29 hours (2.5-4 days)

## Next Steps

1. Start with Priority 1 connections (critical path)
2. Verify end-to-end flow works
3. Add Priority 2 optimizations
4. Add Priority 3 advanced features
5. Comprehensive testing and validation

### To-dos

- [ ] Create UnifiedOrchestratorAdapter to bridge UnifiedOrchestrator to OrchestrationService trait in orchestration_adapter.rs
- [ ] Initialize UnifiedOrchestrator in api-server.rs main() with all required dependencies (plan_generator, plan_executor, council, worker_bridge, etc.)
- [ ] Remove #[cfg(feature = "research")] conditional from claim extraction in caws_adjudication_cycle.rs (line 383) and ensure it always runs
- [ ] Remove #[cfg(feature = "research")] conditional from performance tracker usage in worker_assignment.rs (line 1016) and ensure it always factors into selection
- [ ] Implement actual tool invocation in caws_adjudication_cycle.rs (lines 310-320) - invoke discovered tools and use results for validation
- [ ] Verify reflexive learner adjustments correctly update worker performance cache and affect routing decisions
- [ ] Create ModelLifecycleManager in new file model_lifecycle.rs that monitors performance and triggers hot-swaps when performance degrades
- [ ] Create comprehensive end-to-end integration test in tests/integration_e2e_flow.rs that verifies complete flow from API submission through council review to merge