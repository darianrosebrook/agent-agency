# Agent Orchestration Flow Audit

Generated: 2025-01-XX

## Executive Summary

This audit evaluates the current state of the agent-orchestration system against the desired flow:

1. **Orchestration/arbitration** through judge, planning, and assigning tasks to agent-workers
2. **Council review loop** - agents present work to council for approval/refinement
3. **Work judgment and merge** - approved work gets merged
4. **Git worktree integration** - parallel workers use isolated git worktrees

## Current State Analysis

### ✅ What Exists

#### 1. Planning & Orchestration Infrastructure

**Components Found:**
- `planning/plan_executor.rs` - Core plan execution with milestone processing
- `planning/parallel_coordinator.rs` - Coordinates parallel milestone execution
- `planning/worker_assignment.rs` - Worker assignment strategy with capability matching
- `planning/plan_generator.rs` - Plan generation from working specs
- `planning/council_review.rs` - Council review integration for plans
- `planning/council_monitor.rs` - Council oversight during execution
- `planning/scope_guard.rs` - File locking for parallel execution

**Current Flow:**
```
PlanGenerator → ExecutionPlan → PlanExecutor → WorkerAssignment → WorkerPool
                                                      ↓
                                            ParallelCoordinator
                                                      ↓
                                            CouncilMonitor (oversight)
```

**Status**: ✅ **Well Implemented**
- Planning engine exists and generates execution plans
- Worker assignment with capability matching implemented
- Parallel coordination with scope guards exists
- Council integration for plan review exists

#### 2. Council & Judge Integration

**Components Found:**
- `council.rs` - Main council coordination system
- `council_review.rs` - Plan review integration
- `judge_backup/` - Judge implementations (ethics, quality, security)
- `verdict_aggregation.rs` - Aggregates judge verdicts
- `decision_making.rs` - Decision engine for final verdicts

**Current Flow:**
```
Council → Select Judges → Parallel Reviews → Verdict Aggregation → Decision Engine → Final Decision
```

**Status**: ✅ **Well Implemented**
- Council system fully functional
- Judge selection and parallel review implemented
- Verdict aggregation and decision making exists
- Integration with planning exists via `council_review.rs`

#### 3. Refinement Loop Infrastructure

**Components Found:**
- `autonomous_executor.rs` - Contains refinement loop (lines 1282-1532)
- `council_review.rs` - Council review with conditional approval
- `planning/quality_gates.rs` - Quality gate enforcement

**Current Flow:**
```
Execute Task → Council Review → Conditional Approval? → Refine → Re-execute → Council Review
```

**Status**: ⚠️ **Partially Implemented**
- Refinement loop exists in `autonomous_executor.rs`
- Council conditional approval exists
- **Gap**: Refinement loop is in autonomous executor, not integrated with plan executor
- **Gap**: No explicit "present to council" step after worker completion

#### 4. Git Worktree Integration

**Components Found:**
- `autonomous_executor.rs` - References git branch (lines 980-998)
- `planning/parallel_coordinator.rs` - Parallel execution coordination
- `planning/scope_guard.rs` - File locking (could be extended for worktrees)

**Status**: ❌ **Not Implemented**
- No git worktree creation/management code found
- No worktree isolation for parallel workers
- Git branch detection exists but no worktree usage

### ❌ What's Missing

#### 1. Worker Completion → Council Presentation Flow

**Current Gap:**
- Workers complete tasks but there's no explicit "present to council" step
- No integration between worker completion and council review
- Missing: `worker_completed()` → `present_to_council()` → `council_review()` flow

**Needed:**
```rust
// Missing integration point
async fn on_worker_completion(
    worker_id: Uuid,
    milestone_id: String,
    artifacts: ExecutionArtifacts,
) -> Result<()> {
    // 1. Collect worker output
    // 2. Present to council for review
    // 3. Handle refinement loop if needed
    // 4. Merge if approved
}
```

#### 2. Git Worktree Management

**Current Gap:**
- No worktree creation/cleanup code
- No worktree isolation per worker
- No worktree-based merge process

**Needed:**
```rust
// Missing worktree management
struct WorktreeManager {
    base_repo: PathBuf,
    worktrees: HashMap<Uuid, WorktreeHandle>,
}

impl WorktreeManager {
    async fn create_worktree(&self, worker_id: Uuid, branch: &str) -> Result<WorktreeHandle>;
    async fn cleanup_worktree(&self, worker_id: Uuid) -> Result<()>;
    async fn merge_worktree(&self, worker_id: Uuid, target_branch: &str) -> Result<()>;
}
```

#### 3. Unified Execution Flow

**Current Gap:**
- Multiple execution paths (`PlanExecutor`, `AutonomousExecutor`, `ParallelCoordinator`)
- No unified flow that combines all components
- Refinement loop exists but not integrated with plan execution

**Needed:**
```rust
// Missing unified orchestrator
struct UnifiedOrchestrator {
    plan_executor: Arc<PlanExecutor>,
    council: Arc<Council>,
    worktree_manager: Arc<WorktreeManager>,
    refinement_loop: RefinementLoop,
}

impl UnifiedOrchestrator {
    async fn execute_plan(&self, plan: ExecutionPlan) -> Result<ExecutionResult> {
        // 1. Plan → Judge → Planning → Assign workers
        // 2. Create worktrees for workers
        // 3. Execute milestones in parallel
        // 4. On completion: present to council
        // 5. Refinement loop if needed
        // 6. Merge approved work
    }
}
```

## Detailed Component Analysis

### Planning System (`planning/`)

**Strengths:**
- ✅ Complete plan generation from working specs
- ✅ Dependency resolution
- ✅ Worker assignment with capability matching
- ✅ Parallel execution coordination
- ✅ Scope guards for file locking
- ✅ Council integration for plan review

**Gaps:**
- ⚠️ No worker completion callback integration
- ⚠️ No worktree management
- ⚠️ No merge process

### Council System (`council.rs`, `council_review.rs`)

**Strengths:**
- ✅ Full council implementation with judge selection
- ✅ Parallel judge reviews
- ✅ Verdict aggregation
- ✅ Decision engine
- ✅ Conditional approval support

**Gaps:**
- ⚠️ No explicit "present work to council" API
- ⚠️ Council review exists for plans, but not for completed work

### Execution System (`plan_executor.rs`, `autonomous_executor.rs`)

**Strengths:**
- ✅ Plan execution with milestone processing
- ✅ Worker assignment and execution
- ✅ Evidence collection
- ✅ Refinement loop (in autonomous_executor)

**Gaps:**
- ⚠️ Refinement loop not integrated with plan executor
- ⚠️ No council presentation after worker completion
- ⚠️ No worktree isolation

## Recommended Refactoring & Integration

### Priority 1: Create Unified Orchestration Flow

**Action:** Create `orchestration/unified_orchestrator.rs`

**Responsibilities:**
1. Coordinate plan → judge → planning → worker assignment
2. Manage worktree lifecycle per worker
3. Handle worker completion → council presentation
4. Manage refinement loops
5. Handle merge process

**Integration Points:**
- Use existing `PlanExecutor` for execution
- Use existing `Council` for reviews
- Use existing `WorkerAssignmentStrategy` for assignments
- Create new `WorktreeManager` for git worktrees
- Create new `RefinementLoop` coordinator

### Priority 2: Add Worktree Management

**Action:** Create `planning/worktree_manager.rs`

**Features:**
- Create isolated worktree per worker
- Cleanup worktrees after completion
- Merge worktrees back to main branch
- Handle worktree conflicts

**Integration:**
- Integrate with `ParallelCoordinator` for parallel worktree creation
- Integrate with `PlanExecutor` for worker execution context
- Integrate with merge process

### Priority 3: Add Worker Completion → Council Flow

**Action:** Extend `plan_executor.rs` and create `planning/council_presentation.rs`

**Flow:**
```rust
// In PlanExecutor
async fn on_milestone_completion(
    &self,
    milestone_id: String,
    artifacts: ExecutionArtifacts,
) -> Result<()> {
    // 1. Collect artifacts from worker
    // 2. Present to council via CouncilPresentation
    // 3. Handle refinement if needed
    // 4. Merge if approved
}
```

**New Component:**
- `planning/council_presentation.rs` - Handles presenting completed work to council

### Priority 4: Integrate Refinement Loop

**Action:** Extract refinement loop from `autonomous_executor.rs` and integrate with `PlanExecutor`

**Current State:**
- Refinement loop exists in `autonomous_executor.rs` (lines 1282-1532)
- Not integrated with `PlanExecutor`

**Needed:**
- Extract refinement logic to `planning/refinement_loop.rs`
- Integrate with `PlanExecutor` for milestone refinement
- Connect with council presentation flow

## Architecture Recommendations

### Proposed Unified Flow

```
┌─────────────────────────────────────────────────────────────┐
│                    Unified Orchestrator                      │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  1. Plan Generation                                          │
│     PlanGenerator → ExecutionPlan                           │
│                                                              │
│  2. Council Plan Review                                      │
│     Council → Judge Selection → Review → Approval          │
│                                                              │
│  3. Worker Assignment                                       │
│     WorkerAssignmentStrategy → Assign Workers               │
│                                                              │
│  4. Worktree Creation                                       │
│     WorktreeManager → Create Worktree per Worker            │
│                                                              │
│  5. Parallel Execution                                      │
│     ParallelCoordinator → Execute Milestones                │
│                                                              │
│  6. Worker Completion                                       │
│     Worker → Collect Artifacts → Present to Council         │
│                                                              │
│  7. Council Review of Completed Work                        │
│     Council → Review Artifacts → Conditional Approval       │
│                                                              │
│  8. Refinement Loop (if needed)                             │
│     RefinementLoop → Refine → Re-execute → Re-review        │
│                                                              │
│  9. Merge Approved Work                                     │
│     WorktreeManager → Merge Worktree → Cleanup              │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

### Component Dependencies

```
UnifiedOrchestrator
├── PlanGenerator (exists)
├── Council (exists)
├── WorkerAssignmentStrategy (exists)
├── WorktreeManager (NEW)
├── ParallelCoordinator (exists)
├── PlanExecutor (exists)
├── CouncilPresentation (NEW)
├── RefinementLoop (extract from autonomous_executor)
└── MergeManager (NEW)
```

## Deduplication Opportunities

### 1. Execution Paths

**Current:** Multiple execution paths
- `PlanExecutor` - Plan-based execution
- `AutonomousExecutor` - Autonomous task execution
- `ParallelCoordinator` - Parallel milestone execution

**Recommendation:** 
- Keep `PlanExecutor` as core execution engine
- Extract refinement loop from `AutonomousExecutor` to shared component
- Use `ParallelCoordinator` as coordinator layer

### 2. Council Integration

**Current:** Multiple council integration points
- `council_review.rs` - Plan review
- `council_monitor.rs` - Execution monitoring
- `autonomous_executor.rs` - Refinement loop council calls

**Recommendation:**
- Create unified `CouncilIntegration` trait
- Implement for plan review, execution monitoring, and work presentation
- Single point of council interaction

### 3. Worker Management

**Current:** Worker assignment scattered
- `worker_assignment.rs` - Assignment strategy
- `plan_executor.rs` - Worker execution
- `parallel_coordinator.rs` - Parallel worker coordination

**Recommendation:**
- Keep separation but add `WorkerLifecycleManager`
- Manages: assignment → execution → completion → presentation

## Integration Checklist

### Phase 1: Core Integration (High Priority)

- [ ] Create `WorktreeManager` for git worktree management
- [ ] Create `UnifiedOrchestrator` to coordinate all components
- [ ] Add worker completion callback to `PlanExecutor`
- [ ] Create `CouncilPresentation` component for presenting completed work
- [ ] Integrate refinement loop with `PlanExecutor`

### Phase 2: Refinement & Merge (Medium Priority)

- [ ] Extract refinement loop from `autonomous_executor.rs`
- [ ] Create `RefinementLoop` coordinator component
- [ ] Create `MergeManager` for worktree merging
- [ ] Add conflict resolution for worktree merges

### Phase 3: Deduplication & Cleanup (Low Priority)

- [ ] Create unified `CouncilIntegration` trait
- [ ] Consolidate execution paths
- [ ] Create `WorkerLifecycleManager`
- [ ] Remove duplicate council integration code

## Conclusion

**Current State:** ~70% complete

**What Works:**
- ✅ Planning and plan generation
- ✅ Council and judge system
- ✅ Worker assignment
- ✅ Parallel execution coordination
- ✅ Refinement loop (exists but not integrated)

**What's Missing:**
- ❌ Git worktree management
- ❌ Worker completion → council presentation flow
- ❌ Unified orchestrator coordinating all components
- ❌ Merge process for approved work

**Estimated Effort:**
- Phase 1 (Core Integration): 2-3 days
- Phase 2 (Refinement & Merge): 1-2 days
- Phase 3 (Deduplication): 1 day

**Total: 4-6 days to complete desired flow**

