<!-- 0a97ccac-5af9-4b10-8213-d92b6cca5d77 1c91f573-18fd-4e55-a409-90b8a30157cd -->
# Orchestration Cleanup & Completion Plan

## Current Status Summary

### ✅ Completed Components

- UnifiedOrchestrator (skeleton with TODOs)
- CouncilIntegration trait (fully implemented)
- WorktreeManager (structure complete, git commands simulated)
- WorkerLifecycleManager (complete)
- CawsAdjudicationCycle (5 stages defined, 2 incomplete)
- CawsDebateScorer (complete but not integrated)
- RefinementLoopCoordinator (complete but not integrated)
- WorkerExecutionBridge (complete)

### ❌ Missing Integrations

1. CawsDebateScorer not integrated into CawsAdjudicationCycle deliberation stage
2. RefinementLoopCoordinator not integrated into UnifiedOrchestrator
3. ParallelCoordinator not used in UnifiedOrchestrator (sequential execution only)
4. WorkerAssignmentStrategy not used in UnifiedOrchestrator
5. PlanExecutor doesn't use WorkerLifecycleManager
6. ParallelCoordinator doesn't use WorktreeManager

### ⚠️ Incomplete Implementations

1. WorktreeManager: Git commands are TODOs (lines 124, 183, 217, 253)
2. CawsAdjudicationCycle: Deliberation stage missing debate integration (line 178)
3. CawsAdjudicationCycle: Publication stage missing merge logic (line 222)
4. UnifiedOrchestrator: Merge logic is TODO (line 285)
5. UnifiedOrchestrator: ParallelCoordinator integration TODO (line 311)
6. UnifiedOrchestrator: WorkerAssignmentStrategy integration TODO (line 358)

### 🔧 Type Consolidation Needed

1. `src/types.rs`: Still contains deprecated `WorkingSpec`, `AcceptanceCriterion` (marked deprecated but still present)
2. `src/council_types.rs`: Duplicate `ChangeBudget` definition
3. Need to audit imports to ensure contracts types are used everywhere

### 🐛 Compilation Issues

- 64 errors in deprecated modules (expected, but should be fixed or removed)
- Mostly in `audited_orchestrator.rs` (missing DatabaseClient, TaskDescriptor.id field)

## Cleanup Tasks

### Phase 1: Complete Core Integrations

#### Task 1.1: Integrate CawsDebateScorer into CawsAdjudicationCycle

**File**: `src/planning/caws_adjudication_cycle.rs`

- Add `CawsDebateScorer` as dependency to `CawsAdjudicationCycle`
- Implement `stage_deliberation` to use `CawsDebateScorer.score_debate()` when multiple artifacts present
- Update constructor to accept `CawsDebateScorer`

#### Task 1.2: Integrate RefinementLoopCoordinator into UnifiedOrchestrator

**File**: `src/orchestration/unified_orchestrator.rs`

- Complete refinement loop integration (line 276)
- Create trait implementations for `OrchestrationExecutor`, `ArtifactValidator`, `CouncilReviewer`, `SpecRefiner`, `ProgressTracker`
- Wire up refinement loop when `needs_refinement` is true

#### Task 1.3: Integrate ParallelCoordinator into UnifiedOrchestrator

**File**: `src/orchestration/unified_orchestrator.rs`

- Replace sequential execution (line 311) with `ParallelCoordinator.execute_parallel()`
- Group milestones by dependencies
- Handle parallel execution results

#### Task 1.4: Integrate WorkerAssignmentStrategy into UnifiedOrchestrator

**File**: `src/orchestration/unified_orchestrator.rs`

- Add `WorkerAssignmentStrategy` as dependency
- Replace placeholder worker assignment (line 358) with strategy-based assignment
- Use strategy to select appropriate worker for each milestone

### Phase 2: Complete WorktreeManager Git Integration

#### Task 2.1: Implement Git Worktree Commands

**File**: `src/planning/worktree_manager.rs`

- Implement `create_worktree`: Execute `git worktree add <path> <branch>` (line 124)
- Implement `merge_worktree`: Execute `git merge` with conflict detection (line 183)
- Implement `resolve_conflicts`: Execute conflict resolution strategies (line 217)
- Implement `cleanup_worktree`: Execute `git worktree remove` (line 253)
- Use `git2` crate or `std::process::Command` for git operations
- Add error handling for git command failures

#### Task 2.2: Integrate WorktreeManager into ParallelCoordinator

**File**: `src/planning/parallel_coordinator.rs`

- Add `WorktreeManager` as dependency
- Create worktree before milestone execution
- Pass worktree path to execution context
- Cleanup worktree on completion or failure

### Phase 3: Complete CAWS Adjudication Cycle

#### Task 3.1: Implement Publication Stage Merge Logic

**File**: `src/planning/caws_adjudication_cycle.rs`

- Implement `stage_publication` merge logic (line 222)
- Integrate with `WorktreeManager.merge_worktree()`
- Handle merge conflicts and present to council if needed
- Commit verdict to git with CAWS-VERDICT-ID trailer

#### Task 3.2: Complete UnifiedOrchestrator Merge Logic

**File**: `src/orchestration/unified_orchestrator.rs`

- Implement merge logic (line 285)
- Merge all worktrees back to main branch
- Handle conflicts and rollback on failure
- Update progress tracking

### Phase 4: Integrate WorkerLifecycleManager into PlanExecutor

#### Task 4.1: Add WorkerLifecycleManager to PlanExecutor

**File**: `src/planning/plan_executor.rs`

- Add `WorkerLifecycleManager` as optional dependency
- Call `handle_assignment` when milestone starts
- Call `handle_completion` when milestone completes
- Call `handle_failure` on milestone failure

### Phase 5: Type Consolidation

#### Task 5.1: Audit and Remove Duplicate Types

**Files**: `src/types.rs`, `src/council_types.rs`

- Review all usages of deprecated `WorkingSpec` in `types.rs`
- Replace with `agent_agency_contracts::WorkingSpec`
- Remove duplicate `ChangeBudget` from `council_types.rs` (use contracts version)
- Update all imports to use contracts types
- Remove type adapters if no longer needed

#### Task 5.2: Update All Imports

- Search for imports of `crate::types::WorkingSpec`
- Replace with `agent_agency_contracts::WorkingSpec`
- Search for imports of `crate::council_types::ChangeBudget`
- Replace with `agent_agency_contracts::planning_io::ChangeBudget`

### Phase 6: Fix Deprecated Module Compilation Errors

#### Task 6.1: Fix audited_orchestrator.rs Compilation Errors

**File**: `src/audited_orchestrator.rs`

- Fix `DatabaseClient` not found errors (lines 847, 865)
- Option A: Remove database-dependent code paths
- Option B: Add placeholder type for deprecated module
- Fix `TaskDescriptor.id` field access (line 84)
- Use correct field name from contracts type
- Comment out or stub problematic code paths since module is deprecated

#### Task 6.2: Ensure Deprecated Modules Compile

- Fix compilation errors in `autonomous_executor.rs` if any
- Fix compilation errors in `multimodal_orchestrator.rs` if any
- Add `#[allow(dead_code)]` where appropriate for deprecated code

### Phase 7: Integration Test Updates

#### Task 7.1: Update Integration Tests

**File**: `tests/integration_unified_orchestrator.rs`

- Update test setup to use real `WorkerAssignmentStrategy`
- Update test setup to use real `CawsDebateScorer`
- Add tests for parallel execution
- Add tests for refinement loop
- Add tests for worktree merge

## Implementation Order

1. **Phase 1** (Core Integrations) - Highest priority, enables full workflow
2. **Phase 2** (Worktree Git Integration) - Required for parallel isolation
3. **Phase 3** (CAWS Completion) - Completes adjudication cycle
4. **Phase 4** (PlanExecutor Integration) - Completes worker lifecycle
5. **Phase 5** (Type Consolidation) - Cleanup and consistency
6. **Phase 6** (Deprecated Module Fixes) - Ensure codebase compiles
7. **Phase 7** (Test Updates) - Verify everything works

## Success Criteria

- ✅ All TODOs in UnifiedOrchestrator resolved
- ✅ All TODOs in WorktreeManager resolved (real git commands)
- ✅ All TODOs in CawsAdjudicationCycle resolved
- ✅ CawsDebateScorer integrated into deliberation stage
- ✅ RefinementLoopCoordinator integrated into UnifiedOrchestrator
- ✅ ParallelCoordinator used for parallel execution
- ✅ WorkerAssignmentStrategy used for worker selection
- ✅ PlanExecutor uses WorkerLifecycleManager
- ✅ ParallelCoordinator uses WorktreeManager
- ✅ No duplicate types (all use contracts)
- ✅ All deprecated modules compile (even if stubbed)
- ✅ Integration tests pass with real components

## Risk Mitigation

- Git integration: Test with real git repository, handle edge cases
- Parallel execution: Ensure proper dependency handling
- Type consolidation: Use type adapters during migration, verify no breakage
- Deprecated modules: Keep stubbed for backward compatibility during transition

### To-dos

- [x] Integrate WorkerAssignmentStrategy into UnifiedOrchestrator.assign_worker_to_milestone()