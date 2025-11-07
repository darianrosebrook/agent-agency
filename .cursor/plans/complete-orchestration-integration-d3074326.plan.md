<!-- d3074326-11b3-4f77-991d-e9ed07603709 9183d741-2a14-4c43-abc3-95cb61b617c3 -->
# eComplete Orchestration Integration

## Current Status Assessment

After reviewing the codebase against the orchestration cleanup plan, most integrations are complete:

### ✅ Already Completed

- CawsDebateScorer integrated into CawsAdjudicationCycle (line 53, 231)
- RefinementLoopCoordinator integrated into UnifiedOrchestrator (lines 293-357)
- WorkerAssignmentStrategy integrated (used in UnifiedOrchestrator.assign_worker_to_milestone)
- PlanExecutor uses WorkerLifecycleManager (lines 1123, 1169)
- ParallelCoordinator uses WorktreeManager (lines 390-414)
- WorktreeManager Git commands implemented (real git commands, lines 124-160)
- CawsAdjudicationCycle publication stage merge logic complete (lines 294-417)
- UnifiedOrchestrator merge logic complete (lines 372-426)

### ❌ Remaining Tasks

#### Task 1: Replace Custom Parallel Execution with ParallelCoordinator

**File**: `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`

**Current State**:

- UnifiedOrchestrator has `parallel_coordinator: Arc<ParallelCoordinator>` field (line 99)
- But `execute_plan_milestones()` uses custom parallel execution logic (lines 440-549) with DependencyResolver and manual batch processing
- ParallelCoordinator is not actually used

**Required Changes**:

- Replace custom parallel execution in `execute_plan_milestones()` with `ParallelCoordinator.execute_parallel()`
- Remove custom DependencyResolver-based batching logic
- Delegate parallel execution coordination to ParallelCoordinator
- Ensure worktree isolation is handled by ParallelCoordinator

**Integration Points**:

- Line 265: `execute_plan_milestones()` currently implements custom logic
- Should call `self.parallel_coordinator.execute_parallel()` instead
- ParallelCoordinator already has WorktreeManager integration (line 42)

#### Task 2: Remove Deprecated Types

**File**: `iterations/v3/agent-orchestration/src/types.rs`

**Current State**:

- Deprecated `WorkingSpec` struct still exists (lines 201-223)
- Deprecated `AcceptanceCriterion` struct still exists (line 224)
- Both marked with `#[deprecated]` but not removed
- Only one import found: commented out (line 193)

**Required Changes**:

- Remove deprecated `WorkingSpec` struct definition (lines 201-223)
- Remove deprecated `AcceptanceCriterion` struct definition (line 224)
- Verify no remaining imports or usages
- Update any type adapters if they reference these types

**Verification**:

- Search for any remaining imports: `use.*types::WorkingSpec` or `use.*types::AcceptanceCriterion`
- Check type adapters in `src/planning/type_adapters.rs` if it exists

#### Task 3: Verify WorkerAssignmentStrategy Integration

**File**: `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`

**Current State**:

- `worker_assignment_strategy` field exists (line 120)
- `assign_worker_to_milestone()` method exists (line 558)
- Need to verify it's actually used in execution flow

**Required Changes**:

- Verify `assign_worker_to_milestone()` is called during milestone execution
- Ensure worker assignment uses strategy when available
- Fallback to default assignment if strategy not provided

## Implementation Order

1. **Task 2** (Type Cleanup) - Simplest, no dependencies
2. **Task 1** (ParallelCoordinator Integration) - Core functionality
3. **Task 3** (Verification) - Ensure completeness

## Success Criteria

- [ ] UnifiedOrchestrator uses ParallelCoordinator.execute_parallel() instead of custom logic
- [ ] Deprecated WorkingSpec and AcceptanceCriterion types removed from types.rs
- [ ] No compilation errors after type removal
- [ ] WorkerAssignmentStrategy verified to be used in execution flow
- [ ] All integration tests pass

## Files to Modify

1. `iterations/v3/agent-orchestration/src/orchestration/unified_orchestrator.rs`

- Replace `execute_plan_milestones()` implementation
- Verify `assign_worker_to_milestone()` usage

2. `iterations/v3/agent-orchestration/src/types.rs`

- Remove deprecated type definitions
- Clean up related comments

3. Verify imports across codebase

- Search for any remaining references to deprecated types