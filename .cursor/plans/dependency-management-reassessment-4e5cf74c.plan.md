<!-- 4e5cf74c-efc5-47f0-9be2-b547bb1375d4 dfa0ab56-4729-4f06-bcaa-85956081ca66 -->
# Path to Full Autonomous Operation - Comprehensive Roadmap

## Overview

Transform Agent Agency V3 from current state (~60% dependency management complete, 88+ compilation errors) to fully autonomous operation where:

- Users provide natural-language tasks
- System plans, implements, audits, and refines solutions iteratively
- Council judges provide constitutional oversight at each step
- Memory system preserves context across long-horizon tasks
- Satisficing logic prevents infinite refinement loops
- Real CoreML inference powers all judge deliberations

**Current State**: Dependency management ~60% complete, 88 compilation errors blocking build, CoreML inference mocked, memory system not integrated, autonomous executor incomplete.

**Target State**: Fully autonomous system with iterative refinement, council governance, long-horizon context preservation, and production-ready CoreML inference.

---

## Non-negotiable:

Our rule for our session is no mock implementation, we're going to be hooking things up to the real services and modules that we've defined. If it relies on a dependency we haven't made yet, then we need to outline what is necessary for the dependency to be functionally complete, then implement it. Our goal is to be unblocked by focusing on implementing the required code for it to be considered functionally complete and integrated.

---

## Milestone 1: Complete Dependency Management Foundation

**Goal**: Resolve all compilation errors and establish single source of truth for shared types.

**Current Status**: ~60% complete, 88 compilation errors blocking build

### Task 1.1: Fix Schemars Attribute Errors (P0 - 2-3 hours)

**Files to Fix**:

- `iterations/v3/agent-agency-contracts/src/planning_io.rs` - Add `#[derive(JsonSchema)]` to ~9 structs
- `iterations/v3/agent-agency-contracts/src/task_executor.rs` - Add `#[derive(JsonSchema)]` to 3 structs  
- `iterations/v3/agent-agency-contracts/src/planning.rs` - Add `#[derive(JsonSchema)]` to 4 structs
- `iterations/v3/agent-agency-contracts/src/types/*.rs` - Verify and fix similar issues

**Pattern**:

```rust
// BEFORE
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
}

// AFTER
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskExecutionResult {
    #[schemars(with = "String")]
    pub started_at: DateTime<Utc>,
}
```

**Acceptance**: `cargo check --package agent-agency-contracts` passes with 0 errors

### Task 1.2: Remove Duplicate Type Definitions (P0 - 2-3 hours)

**Files to Fix**:

- `iterations/v3/agent-orchestration/src/autonomous_executor.rs:213` - Remove `RiskTier` redefinition
- `iterations/v3/agent-orchestration/src/multimodal_orchestration.rs:76-80` - Remove duplicate type aliases (`ConsensusCoordinator`, `KnowledgeSeeker`, `OrchestratorConfig`)
- `iterations/v3/agent-orchestration/src/council_types.rs` - Verify `TaskPriority` uses contracts version

**Pattern**:

```rust
// BEFORE
pub type RiskTier = String; // ❌ Duplicate

// AFTER
use agent_agency_contracts::prelude::*; // ✅ Import from contracts
```

**Acceptance**: No duplicate type definitions, all imports use `agent_agency_contracts::prelude::*`

### Task 1.3: Fix Type Mismatches and Missing Imports (P0 - 4-6 hours)

**Files to Fix**:

- `iterations/v3/agent-orchestration/src/lib.rs:266` - Fix `Council::new()` call signature
- `iterations/v3/agent-orchestration/src/lib.rs:273` - Fix `AutonomousExecutor::new()` call signature
- `iterations/v3/agent-orchestration/src/multimodal_orchestration.rs` - Fix missing `agent_data_processing` imports
- All files using `WorkingSpec`, `ExecutionArtifacts`, `FileChange` - Update field access to match contracts

**References**:

- `Council::new()` signature: `iterations/v3/agent-orchestration/src/council.rs:223-228`
- `AutonomousExecutor::new()` signature: `iterations/v3/agent-orchestration/src/autonomous_executor.rs:865-898`

**Acceptance**: `cargo check --package agent-orchestration` passes with 0 errors

### Task 1.4: Complete Port Implementations (P1 - 3-4 hours)

**Files to Update**:

- `iterations/v3/agent-orchestration/src/planning/planning_engine_impl.rs` - Migrate to use contracts types
- `iterations/v3/agent-orchestration/src/planning/tool_chain_planner.rs` - Remove placeholder implementations
- `iterations/v3/agent-agency-contracts/src/ports/consensus_coordinator.rs` - Add `ConsensusCoordinator` port trait if missing

**Acceptance**: All port traits implemented using contracts types, no placeholder TODOs in production code

---

## Milestone 2: Enable Real CoreML Inference

**Goal**: Replace mocked CoreML responses with actual ANE-accelerated inference.

**Current Status**: CoreML infrastructure exists but inference disabled (returns mock at line 266)

### Task 2.1: Enable CoreML Mistral Inference (P0 - 2-3 hours)

**File**: `iterations/v3/engine-coreml/src/lib.rs:266`

**Changes**:

1. Uncomment real inference call (currently commented)
2. Fix model cloning issue if present
3. Enable ANE acceleration path
4. Add proper error handling for inference failures

**Reference Implementation**: CoreML integration pattern from `iterations/v3/engine-coreml/src/inference.rs`

**Acceptance**: Real Mistral inference executes, returns actual text responses (not mocks)

### Task 2.2: Integrate CoreML with Council Judges (P0 - 2-3 hours)

**Files**:

- `iterations/v3/agent-constitutional-council/src/judges/constitutional_judge.rs`
- `iterations/v3/agent-constitutional-council/src/judges/technical_auditor.rs`
- `iterations/v3/agent-constitutional-council/src/judges/quality_evaluator.rs`
- `iterations/v3/agent-constitutional-council/src/judges/integration_validator.rs`

**Changes**:

1. Replace any mock/Ollama calls with CoreML inference
2. Ensure all judges use CoreML Mistral for deliberations
3. Add performance tracking (inference time, ANE utilization)

**Acceptance**: All 4 council judges use real CoreML inference, performance metrics logged

### Task 2.3: Remove Ollama References (P1 - 4-6 hours)

**Files**: 25 files with Ollama references (per README)

**Search Pattern**: `rg -i "ollama" iterations/v3/`

**Changes**:

1. Remove Ollama provider implementations
2. Update model routing to CoreML-only
3. Remove Ollama dependencies from `Cargo.toml` files
4. Update documentation references

**Acceptance**: Zero Ollama references in codebase, `cargo check` passes, CoreML-only architecture

---

## Milestone 3: Complete Autonomous Execution Loop

**Goal**: Full end-to-end autonomous task execution with progress tracking and error recovery.

**Current Status**: Basic structure exists but incomplete integration, missing refinement loops

### Task 3.1: Complete `execute_task` Implementation (P0 - 4-6 hours)

**File**: `iterations/v3/agent-orchestration/src/autonomous_executor.rs:1110`

**Current State**: Basic structure exists, missing:

- Full council integration
- Artifact collection and validation
- Quality gate enforcement
- Error recovery mechanisms

**Changes**:

1. Complete council review integration (lines 1140-1200)
2. Add artifact collection from workers (lines 1200-1300)
3. Implement quality gate checks (lines 1300-1400)
4. Add error recovery and retry logic (lines 1400-1500)
5. Remove all TODO placeholders in execution path

**References**:

- Council integration: `iterations/v3/agent-orchestration/src/adapter.rs:209-214`
- Artifact review: `iterations/v3/agent-orchestration/src/adapter.rs:218-230`

**Acceptance**: `execute_task` completes full cycle: plan → council → execute → validate → finish

### Task 3.2: Implement Iterative Refinement Loop (P0 - 6-8 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/autonomous_executor.rs` - Add refinement loop
- `iterations/v3/agent-agency-contracts/src/refinement_decision.rs` - Ensure `RefinementDecision` types complete

**Pattern** (from theory.md):

```rust
loop {
    let result = execute_task(task).await?;
    let council_verdict = council.review(result).await?;
    
    match council_verdict {
        CouncilDecision::Accept => break,
        CouncilDecision::Refine { changes } => {
            task = apply_refinements(task, changes).await?;
            continue;
        },
        CouncilDecision::Reject => return Err(...),
    }
}
```

**Integration Points**:

- Council refinement decisions: `iterations/v3/agent-orchestration/src/council.rs:1425`
- Self-prompting loop: `iterations/v3/agent-research/src/self_prompting_agent/loop_controller.rs:100-133`

**Acceptance**: Tasks refine iteratively based on council feedback until acceptance or max iterations

### Task 3.3: Implement Satisficing Logic (P0 - 3-4 hours)

**File**: `iterations/v3/agent-orchestration/src/autonomous_executor.rs`

**Requirements** (from theory.md):

- Quantitative thresholds for acceptance (quality score ≥ 0.9)
- Diminishing returns detection (no improvement after 2 iterations)
- Maximum iteration limits (default: 5)
- Delta threshold checks (improvement < 0.05 after 2 iterations)

**Pattern**:

```rust
fn should_continue_refining(
    current_score: f64,
    previous_score: f64,
    iteration: u32,
    max_iterations: u32,
) -> bool {
    if current_score >= 0.9 { return false; } // Satisficing threshold
    if iteration >= max_iterations { return false; } // Max iterations
    if iteration >= 2 && (current_score - previous_score) < 0.05 { return false; } // Diminishing returns
    true
}
```

**Acceptance**: Refinement loop terminates on satisficing conditions, prevents infinite loops

---

## Milestone 4: Wire Memory System for Long-Horizon Tasks

**Goal**: Enable context preservation across long-running tasks and multiple sessions.

**Current Status**: Memory system exists but not integrated into autonomous executor

### Task 4.1: Integrate Memory System into Autonomous Executor (P0 - 4-6 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/autonomous_executor.rs` - Add memory integration
- `iterations/v3/agent-memory/src/lib.rs` - Verify memory system API

**Changes**:

1. Add memory system parameter to `AutonomousExecutor::new()` (feature-gated)
2. Store task context before execution
3. Retrieve relevant memories for task planning
4. Update memories after successful execution
5. Implement context offloading for long tasks

**Memory Integration Points**:

- Context preservation: `iterations/v3/agent-memory/src/context_management.rs:62-67`
- Memory retrieval: `iterations/v3/agent-memory/src/memory_types.rs:1-73`

**Pattern**:

```rust
#[cfg(feature = "memory")]
async fn execute_with_memory(&self, task: TaskDescriptor) -> Result<()> {
    // Retrieve relevant memories
    let context = self.memory_system.retrieve_context(&task).await?;
    
    // Execute with context
    let result = self.execute_task_with_context(task, context).await?;
    
    // Store execution memory
    self.memory_system.store_experience(result).await?;
    
    Ok(())
}
```

**Acceptance**: Memory system integrated, context preserved across executions, long-horizon tasks supported

### Task 4.2: Implement Context Offloading for Long Tasks (P1 - 3-4 hours)

**Files**:

- `iterations/v3/agent-memory/src/context_management.rs` - Enhance context folding
- `iterations/v3/agent-orchestration/src/autonomous_executor.rs` - Add offloading triggers

**Requirements** (from theory.md):

- Automatic context folding when working memory exceeds limits
- Semantic similarity filtering for context retrieval
- Temporal relevance weighting
- Multi-modal context preservation

**Pattern**:

```rust
async fn offload_context_if_needed(&self, task_id: Uuid) -> Result<()> {
    let memory_stats = self.memory_system.get_stats().await?;
    
    if memory_stats.active_contexts > self.config.max_active_contexts {
        let low_priority_contexts = self.memory_system.identify_foldable_contexts().await?;
        for context in low_priority_contexts {
            self.memory_system.fold_context(context.id).await?;
        }
    }
    
    Ok(())
}
```

**Acceptance**: Context automatically offloaded when limits exceeded, relevant context retrieved efficiently

---

## Milestone 5: Complete Council Integration

**Goal**: Full council governance with all 4 judges participating in iterative refinement.

**Current Status**: Council framework exists, partial integration

### Task 5.1: Complete Council Review Integration (P0 - 4-6 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/adapter.rs:209-214` - Enhance council review
- `iterations/v3/agent-orchestration/src/council.rs:1405-1476` - Complete `review_task` implementation
- `iterations/v3/agent-constitutional-council/src/council.rs` - Verify council coordination

**Current Issue**: `CouncilSession::review_task()` requires pre-populated `final_decision` (see line 1446-1458)

**Changes**:

1. Fix `CouncilSession::review_task()` to perform full review if decision missing
2. Ensure all 4 judges participate in reviews
3. Complete verdict aggregation logic
4. Add proper error handling for quorum failures

**Pattern**:

```rust
pub async fn review_task(&self, task: &OrchestratedTask) -> CouncilResult<ConsensusResult> {
    // If already reviewed, return cached decision
    if let Some(ref decision) = self.final_decision {
        return Ok(convert_decision(decision));
    }
    
    // Otherwise, perform full review
    let review_context = self.build_review_context(task).await?;
    self.council.run_review_process(self, review_context).await?;
    
    // Now final_decision is populated
    Ok(convert_decision(self.final_decision.as_ref().unwrap()))
}
```

**Acceptance**: Council reviews work end-to-end, all judges participate, verdicts aggregated correctly

### Task 5.2: Implement Debate Protocol for Competing Solutions (P1 - 6-8 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/council.rs` - Add debate orchestration
- `iterations/v3/agent-constitutional-council/src/judges/*.rs` - Enable judge participation in debates

**Requirements** (from theory.md):

- Multiple worker models propose competing solutions
- Each worker defends its solution with evidence
- Judge model evaluates arguments (not raw data)
- Highest-scoring solution wins

**Pattern**:

```rust
async fn conduct_debate(
    solutions: Vec<WorkerSolution>,
    judge: Arc<dyn Judge>,
) -> Result<DebateResult> {
    let mut pleas = Vec::new();
    
    // Each worker pleads its case
    for solution in solutions {
        let plea = worker.defend_solution(&solution).await?;
        pleas.push(plea);
    }
    
    // Judge evaluates arguments
    let verdict = judge.evaluate_pleas(&pleas).await?;
    
    // Select highest-scoring solution
    Ok(DebateResult {
        winner: verdict.highest_score_solution,
        confidence: verdict.confidence,
    })
}
```

**Acceptance**: Debate protocol operational, judges evaluate competing solutions, best solution selected

---

## Milestone 6: Enable Long-Horizon Iterative Execution

**Goal**: System can handle tasks that span multiple sessions with context preservation.

**Current Status**: Infrastructure exists, integration incomplete

### Task 6.1: Implement Task State Persistence (P0 - 3-4 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/autonomous_executor.rs` - Add state persistence
- `iterations/v3/data-interfaces/src/persistence.rs` - Verify persistence layer

**Changes**:

1. Save execution state after each iteration
2. Enable task resumption from saved state
3. Track iteration history and refinement changes
4. Implement state recovery on restart

**Pattern**:

```rust
async fn save_execution_state(&self, task_id: Uuid, state: ExecutionState) -> Result<()> {
    let state_record = ExecutionStateRecord {
        task_id,
        state: serde_json::to_value(&state)?,
        iteration: state.current_iteration,
        timestamp: Utc::now(),
    };
    
    self.persistence.save_state(state_record).await?;
    Ok(())
}

async fn resume_task(&self, task_id: Uuid) -> Result<ExecutionState> {
    let state_record = self.persistence.load_state(task_id).await?;
    let state: ExecutionState = serde_json::from_value(state_record.state)?;
    Ok(state)
}
```

**Acceptance**: Tasks can be paused and resumed, state persists across restarts

### Task 6.2: Implement Multi-Session Context Continuity (P0 - 4-6 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/autonomous_executor.rs` - Add session continuity
- `iterations/v3/agent-memory/src/memory_types.rs` - Ensure memory types support sessions

**Changes**:

1. Link task executions to session IDs
2. Retrieve context from previous sessions
3. Maintain conversation history across sessions
4. Enable cross-session learning

**Pattern**:

```rust
async fn execute_with_session_context(
    &self,
    task: TaskDescriptor,
    session_id: Uuid,
) -> Result<()> {
    // Retrieve previous session context
    let previous_context = self.memory_system
        .retrieve_session_context(session_id)
        .await?;
    
    // Execute with historical context
    let result = self.execute_task_with_context(task, previous_context).await?;
    
    // Update session context
    self.memory_system.update_session_context(session_id, result).await?;
    
    Ok(())
}
```

**Acceptance**: Tasks maintain context across sessions, long-horizon tasks supported

### Task 6.3: Implement Progress Tracking Across Iterations (P1 - 3-4 hours)

**Files**:

- `iterations/v3/agent-orchestration/src/progress_tracker.rs` - Enhance progress tracking
- `iterations/v3/agent-orchestration/src/autonomous_executor.rs` - Integrate progress updates

**Changes**:

1. Track progress metrics per iteration
2. Detect quality plateaus (diminishing returns)
3. Generate progress reports for long-running tasks
4. Enable progress monitoring via CLI/API

**Pattern**:

```rust
async fn track_iteration_progress(
    &self,
    task_id: Uuid,
    iteration: u32,
    metrics: IterationMetrics,
) -> Result<()> {
    let progress = ExecutionProgress {
        task_id,
        iteration,
        quality_score: metrics.quality_score,
        improvement_delta: metrics.improvement_delta,
        timestamp: Utc::now(),
    };
    
    self.progress_tracker.record_progress(progress).await?;
    
    // Detect plateaus
    if iteration >= 2 {
        let recent_progress = self.progress_tracker.get_recent_progress(task_id, 3).await?;
        if self.detect_plateau(&recent_progress) {
            warn!("Quality plateau detected for task {}", task_id);
        }
    }
    
    Ok(())
}
```

**Acceptance**: Progress tracked across iterations, plateaus detected, monitoring available

---

## Verification & Testing Strategy

### Compilation Verification

- `cargo check --workspace --all-features` passes with 0 errors
- `cargo test --workspace` passes with all tests green
- No duplicate type definitions (grep gates)

### Integration Testing

- End-to-end task execution: User task → Plan → Council → Execute → Refine → Accept
- Long-horizon task: Task spans 3+ sessions, context preserved
- Council governance: All 4 judges participate, verdicts aggregated
- Memory integration: Context retrieved and updated correctly

### Performance Verification

- CoreML inference: <50ms judge deliberations (ANE-accelerated)
- Task execution: P95 latency <5 seconds for standard tasks
- Memory operations: Context retrieval <100ms

### Quality Gates

- All acceptance criteria from working specs met
- Council approval required for all changes
- CAWS compliance validated at each step
- Provenance tracked for all operations

---

## Success Criteria

### Minimum Viable Autonomous Operation

- [ ] Dependency management complete (0 compilation errors)
- [ ] Real CoreML inference enabled (not mocked)
- [ ] Autonomous executor completes full cycle
- [ ] Council reviews work end-to-end
- [ ] Memory system integrated (optional feature flag)

### Full Autonomous Operation

- [ ] Iterative refinement loop operational
- [ ] Satisficing logic prevents infinite loops
- [ ] Long-horizon tasks supported (multi-session)
- [ ] Council debate protocol implemented
- [ ] Progress tracking across iterations
- [ ] All Ollama references removed

### Production Readiness

- [ ] All tests passing
- [ ] Performance benchmarks met
- [ ] Error handling comprehensive
- [ ] Documentation complete
- [ ] CI/CD gates prevent regressions

---

## Dependencies & Prerequisites

### Blocking Dependencies

1. **Milestone 1** must complete before others (blocks compilation)
2. **Milestone 2** enables Milestone 3 (council needs real inference)
3. **Milestone 3** enables Milestone 4 (executor needs to use memory)
4. **Milestone 4** enables Milestone 6 (memory needed for long-horizon)

### Parallel Work

- Milestone 5 (Council Integration) can proceed in parallel with Milestone 4
- Documentation updates can proceed throughout
- Performance optimization can proceed after Milestone 2

---

## Risk Mitigation

### High Risk Areas

1. **Breaking Changes**: Type migrations may break existing code

   - **Mitigation**: Use adapter pattern, gradual migration, comprehensive testing

2. **Performance**: CoreML inference may be slower than expected

   - **Mitigation**: Profile early, optimize bottlenecks, CPU fallback available

3. **Complexity**: Iterative refinement may cause infinite loops

   - **Mitigation**: Strict satisficing logic, max iteration limits, quality plateau detection

4. **Memory Integration**: Context offloading may lose important information

   - **Mitigation**: Semantic similarity filtering, importance weighting, comprehensive testing

---

## Timeline Estimate

### Optimistic (Focused Work)

- Milestone 1: 8-12 hours
- Milestone 2: 4-6 hours  
- Milestone 3: 13-18 hours
- Milestone 4: 7-10 hours
- Milestone 5: 10-14 hours
- Milestone 6: 10-13 hours

**Total**: 52-73 hours (~1.5-2 weeks of focused work)

### Realistic (With Testing & Iteration)

- Add 30% buffer for testing, debugging, and iteration
- **Total**: 68-95 hours (~2-3 weeks of focused work)

---

## Next Steps

1. **Immediate**: Complete Milestone 1 (dependency management) to unblock compilation
2. **Short-term**: Enable real CoreML inference (Milestone 2) to unblock council
3. **Medium-term**: Complete autonomous execution loop (Milestone 3)
4. **Long-term**: Integrate memory and enable long-horizon tasks (Milestones 4-6)

### To-dos

- [ ] Fix schemars attribute errors: Add JsonSchema derive to 16 files in agent-agency-contracts (planning_io.rs, task_executor.rs, planning.rs, types/*.rs)