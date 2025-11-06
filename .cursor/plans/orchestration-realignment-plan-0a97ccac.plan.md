<!-- 0a97ccac-5af9-4b10-8213-d92b6cca5d77 1c91f573-18fd-4e55-a409-90b8a30157cd -->
# Orchestration Realignment Plan

## Current State Assessment

### Fragmentation Issues Identified

1. **Multiple Execution Paths** (5+ orchestrators):

   - `PlanExecutor` - Plan-based execution (KEEP as core)
   - `AutonomousExecutor` - Autonomous task execution (EXTRACT refinement loop, then DEPRECATE)
   - `ParallelCoordinator` - Parallel milestone execution (KEEP as coordinator layer)
   - `AuditedOrchestrator` - Legacy orchestrator (CULL)
   - `MultimodalOrchestrator` - Multimodal processing (ASSESS if needed)
   - `OrchestratorPlanningIntegration` - Planning integration (KEEP but refactor)

2. **Scattered Council Integration** (3+ integration points):

   - `planning/council_review.rs` - Plan review (KEEP)
   - `planning/council_monitor.rs` - Execution monitoring (KEEP)
   - `autonomous_executor.rs` - Refinement loop council calls (EXTRACT to unified trait)
   - `council.rs` - Main council (KEEP)

3. **Worker Management Fragmentation**:

   - `planning/worker_assignment.rs` - Assignment strategy (KEEP)
   - `planning/plan_executor.rs` - Worker execution (KEEP)
   - `planning/parallel_coordinator.rs` - Parallel coordination (KEEP)
   - Missing: Worker lifecycle manager (BUILD)

4. **CAWS Integration Partial**:

   - `planning/caws_integration.rs` - Working spec validation (KEEP)
   - `planning/waiver_integration.rs` - Waiver system (KEEP)
   - Missing: Full CAWS Adjudication Cycle implementation (BUILD)

### Missing Critical Components

1. **Git Worktree Management**: Completely missing (required for parallel isolation)
2. **Unified Orchestrator**: No single entry point coordinating all components
3. **Worker Completion → Council Flow**: No explicit presentation step
4. **CAWS Adjudication Cycle**: Partial implementation, missing Pleading/Publication stages
5. **Claim Extraction Integration**: Exists in `agent-research` but not integrated with orchestration
6. **Refinement Loop Coordinator**: Exists in autonomous_executor but not reusable
7. **MCP Integration**: MCP server exists but not integrated with UnifiedOrchestrator
8. **Model Performance Benchmarking**: Exists in `agent-research` but not used for worker routing
9. **Reflexive Learning**: Basic progress tracker exists but lacks turn-level RL capabilities

## Refactoring Strategy

### Phase 1: Consolidate Execution Paths

**Goal**: Single unified execution flow through `UnifiedOrchestrator`

**Actions**:

1. Create `src/orchestration/unified_orchestrator.rs`:

   - Coordinates: PlanGenerator → Council → PlanExecutor → ParallelCoordinator
   - Manages: Worktree lifecycle → Worker execution → Council presentation → Refinement → Merge
   - Single entry point: `execute_plan(working_spec: WorkingSpec) -> Result<ExecutionResult>`

2. Extract refinement loop from `autonomous_executor.rs`:

   - Create `src/planning/refinement_loop.rs`
   - Extract lines 1282-1532 from autonomous_executor.rs
   - Make it reusable by PlanExecutor and UnifiedOrchestrator

3. Deprecate redundant orchestrators:

   - Mark `AuditedOrchestrator` as deprecated (replace with UnifiedOrchestrator)
   - Assess `MultimodalOrchestrator` - if needed, integrate into UnifiedOrchestrator
   - Keep `AutonomousExecutor` temporarily for backward compatibility, mark deprecated

**Files to Modify**:

- `src/lib.rs` - Export UnifiedOrchestrator as primary entry point
- `src/autonomous_executor.rs` - Extract refinement loop, mark deprecated
- `src/audited_orchestrator.rs` - Mark deprecated
- `src/planning/plan_executor.rs` - Add worker completion callback hook

### Phase 2: Unify Council Integration

**Goal**: Single `CouncilIntegration` trait for all council interactions

**Actions**:

1. Create `src/council/integration.rs`:
   ```rust
   pub trait CouncilIntegration {
       async fn review_plan(&self, plan: &ExecutionPlan) -> Result<CouncilReviewResult>;
       async fn review_completed_work(&self, artifacts: &ExecutionArtifacts) -> Result<CouncilReviewResult>;
       async fn monitor_execution(&self, context: &ExecutionContext) -> Result<CouncilOversight>;
   }
   ```

2. Implement trait for existing council integrations:

   - `planning/council_review.rs` - Implement `review_plan`
   - `planning/council_monitor.rs` - Implement `monitor_execution`
   - Create `planning/council_presentation.rs` - Implement `review_completed_work`

3. Update all council call sites to use unified trait

**Files to Create**:

- `src/council/integration.rs` - Unified CouncilIntegration trait
- `src/planning/council_presentation.rs` - Work presentation to council

**Files to Modify**:

- `src/planning/council_review.rs` - Implement CouncilIntegration trait
- `src/planning/council_monitor.rs` - Implement CouncilIntegration trait
- `src/autonomous_executor.rs` - Use CouncilIntegration trait instead of direct calls

### Phase 3: Implement Git Worktree Management

**Goal**: Complete worktree isolation per worker

**Actions**:

1. Create `src/planning/worktree_manager.rs`:
   ```rust
   pub struct WorktreeManager {
       base_repo: PathBuf,
       worktrees: HashMap<Uuid, WorktreeHandle>,
   }
   
   impl WorktreeManager {
       async fn create_worktree(&self, worker_id: Uuid, branch: &str) -> Result<WorktreeHandle>;
       async fn cleanup_worktree(&self, worker_id: Uuid) -> Result<()>;
       async fn merge_worktree(&self, worker_id: Uuid, target_branch: &str) -> Result<MergeResult>;
   }
   ```

2. Integrate with ParallelCoordinator:

   - Create worktree before worker assignment
   - Pass worktree path to worker execution context
   - Cleanup on completion or failure

3. Add merge conflict resolution:

   - Detect conflicts during merge
   - Present conflicts to council for resolution
   - Implement automatic conflict resolution strategies

**Files to Create**:

- `src/planning/worktree_manager.rs` - Complete worktree lifecycle management

**Files to Modify**:

- `src/planning/parallel_coordinator.rs` - Integrate worktree creation/cleanup
- `src/planning/plan_executor.rs` - Pass worktree context to workers

### Phase 4: Complete CAWS Adjudication Cycle

**Goal**: Full implementation of Pleading → Examination → Deliberation → Verdict → Publication

**Actions**:

1. Create `src/caws/adjudication_cycle.rs`:

   - Implement all 5 stages as explicit functions
   - Map each stage to enforcement mechanisms (Rust validators, local plugins, git integration)
   - Track CAWS-VERDICT-ID through git trailers

2. Integrate with UnifiedOrchestrator:

   - Worker completion triggers Pleading stage
   - Examination checks CAWS budgets and structural diffs
   - Deliberation runs verifier tests and collects gate metrics
   - Verdict issues PASS/FAIL/WAIVER_REQUIRED
   - Publication commits verdict + provenance to git

3. Implement CAWS Debate scoring:

   - Extract scoring logic from theory.md (S = 0.4E + 0.3B + 0.2G + 0.1P)
   - Apply when multiple workers propose competing solutions
   - Log superseded submissions

**Files to Create**:

- `src/caws/adjudication_cycle.rs` - Complete CAWS Adjudication Cycle
- `src/caws/debate_scorer.rs` - CAWS Debate scoring logic

**Files to Modify**:

- `src/orchestration/unified_orchestrator.rs` - Integrate adjudication cycle
- `src/council.rs` - Add CAWS clause citation to judge prompts

### Phase 5: Worker Lifecycle Management

**Goal**: Unified worker lifecycle: assignment → execution → completion → presentation

**Actions**:

1. Create `src/planning/worker_lifecycle.rs`:
   ```rust
   pub struct WorkerLifecycleManager {
       assignment_strategy: Arc<WorkerAssignmentStrategy>,
       worktree_manager: Arc<WorktreeManager>,
       council_integration: Arc<dyn CouncilIntegration>,
   }
   
   impl WorkerLifecycleManager {
       async fn assign_worker(&self, milestone: &Milestone) -> Result<WorkerAssignment>;
       async fn execute_milestone(&self, assignment: &WorkerAssignment) -> Result<ExecutionArtifacts>;
       async fn on_completion(&self, artifacts: ExecutionArtifacts) -> Result<CouncilReviewResult>;
   }
   ```

2. Integrate with PlanExecutor:

   - Replace scattered worker management with lifecycle manager
   - Add completion callback: `on_milestone_completion(artifacts) -> Result<()>`

**Files to Create**:

- `src/planning/worker_lifecycle.rs` - Unified worker lifecycle management

**Files to Modify**:

- `src/planning/plan_executor.rs` - Use WorkerLifecycleManager
- `src/planning/parallel_coordinator.rs` - Delegate to WorkerLifecycleManager

### Phase 6: Cleanup & Deprecation

**Goal**: Remove deprecated code and consolidate remaining fragments

**Actions**:

1. Mark deprecated modules:

   - `src/audited_orchestrator.rs` - Add `#[deprecated]` attribute
   - `src/autonomous_executor.rs` - Mark as deprecated, keep for migration period
   - `src/multimodal_orchestrator.rs` - Assess if still needed, otherwise deprecate

2. Consolidate duplicate types:

   - Review `src/types.rs` vs `agent_agency_contracts` types
   - Remove duplicates, use contracts as source of truth
   - Update all imports to use contracts types

3. Remove unused code:

   - Search for unused functions/modules
   - Remove commented-out code blocks
   - Clean up TODO comments that reference removed features

**Files to Modify**:

- `src/lib.rs` - Update exports, mark deprecated modules
- `src/types.rs` - Remove duplicates, use contracts
- All files importing deprecated types - Update to contracts

## Implementation Order

1. **Week 1: Foundation** (Phases 1-2)

   - Create AgentOrchestrator skeleton
   - Extract refinement loop
   - Create CouncilIntegration trait
   - Create CouncilPresentation component

2. **Week 2: Core Integration** (Phases 3-4)

   - Implement WorktreeManager
   - Complete CAWS Adjudication Cycle
   - Integrate with UnifiedOrchestrator

3. **Week 3: Lifecycle & Cleanup** (Phases 5-6)

   - Create WorkerLifecycleManager
   - Deprecate old orchestrators
   - Consolidate types and remove duplicates

## Success Criteria

- Single entry point: `AgentOrchestrator::execute_plan()` handles full workflow
- Complete CAWS Adjudication Cycle: All 5 stages implemented and tested
- Git worktree isolation: Each worker operates in isolated worktree
- Unified council integration: All council interactions go through CouncilIntegration trait
- No duplicate execution paths: Deprecated orchestrators removed or clearly marked
- Full end-to-end flow: Plan → Council → Workers → Council → Refine → Merge

## Risk Mitigation

- Keep deprecated modules temporarily for backward compatibility with a clear re
- Add feature flags for new unified flow vs legacy paths
- Comprehensive integration tests before removing deprecated code
- Migration guide documenting changes for existing users

### To-dos

- [ ] Create AgentOrchestrator as single entry point coordinating PlanGenerator → Council → PlanExecutor → ParallelCoordinator → WorktreeManager → CouncilPresentation → RefinementLoop → MergeManager
- [ ] Extract refinement loop from autonomous_executor.rs (lines 1282-1532) into planning/refinement_loop.rs as reusable component
- [ ] Create CouncilIntegration trait unifying council_review.rs, council_monitor.rs, and new council_presentation.rs into single interface
- [ ] Create planning/council_presentation.rs component for presenting completed work artifacts to council for review
- [ ] Create planning/worktree_manager.rs with create_worktree, cleanup_worktree, and merge_worktree methods for git worktree isolation
- [ ] Add worker completion callback to PlanExecutor that triggers council presentation and refinement loop integration
- [ ] Create caws/adjudication_cycle.rs implementing full Pleading → Examination → Deliberation → Verdict → Publication workflow with git integration
- [ ] Implement CAWS Debate scoring (S = 0.4E + 0.3B + 0.2G + 0.1P) for evaluating competing worker submissions
- [ ] Create planning/worker_lifecycle.rs unifying worker assignment → execution → completion → presentation flow
- [ ] Mark AuditedOrchestrator and AutonomousExecutor as deprecated, update lib.rs exports to prioritize UnifiedOrchestrator
- [ ] Remove duplicate types from src/types.rs, ensure all code uses agent_agency_contracts types as source of truth
- [ ] Create comprehensive integration tests for full end-to-end flow: Plan → Council → Workers → Council → Refine → Merge