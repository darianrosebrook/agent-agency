<!-- 7a7233aa-3380-4026-97fb-bf58811f8eec 7307b2e8-8ec0-4fc1-9071-bdeaa9ec0f15 -->
# V3 Agent Integration Plan

## Executive Summary

The v3 agent system has **~85% of core functionality implemented** but lacks critical connection points between components. The system architecture is sound with UnifiedOrchestrator, Council, Workers, and API infrastructure, but the integration layer is incomplete.

## Current State Analysis

### ✅ What Exists and Works

#### 1. Core Orchestration Engine

- **UnifiedOrchestrator** (`agent-orchestration/src/orchestration/unified_orchestrator.rs`)
  - `execute_plan()` method fully implemented
  - Plan generation → Council review → Execution → Refinement loop flow
  - Worktree management integration
  - State persistence and crash recovery
  - Memory system integration (feature-gated)

#### 2. Council & Constitutional Governance

- **Council System** (`agent-orchestration/src/council/`)
  - Four-judge constitutional oversight (Ethics, Quality, Security, Technical)
  - CAWS adjudication cycle with debate scoring
  - Verdict aggregation and consensus building
  - Conditional approval support

#### 3. Planning & Execution

- **PlanGenerator** - Converts WorkingSpec to ExecutionPlan
- **PlanExecutor** - Executes milestones with parallel coordination
- **ParallelCoordinator** - Manages parallel milestone execution
- **WorkerExecutionBridge** - Bridges to agent-workers MCP pool
- **WorktreeManager** - Git worktree isolation per worker

#### 4. API Infrastructure

- **API Server** (`data-interfaces-adapters/src/bin/api-server.rs`)
  - Full REST API with 30+ endpoints
  - Health checks, task management, progress tracking
  - Chain-of-thought observation endpoints
  - Database integration

#### 5. Adapter Layer

- **UnifiedOrchestratorAdapter** (`data-interfaces-adapters/src/orchestration_adapter.rs`)
  - `orchestrate_task()` method implemented
  - Converts WorkingSpec → UnifiedOrchestrator.execute_plan()
  - Returns TaskExecutionResult

### ❌ What's Missing or Not Connected

#### 1. API → Orchestrator Connection (CRITICAL)

**Problem**: API handlers don't use UnifiedOrchestratorAdapter

**Current State**:

```rust
// api-server.rs:433 - submit_task_handler uses old RestApi
match api.submit_task(request).await {
    // Uses legacy OrchestratorService, not UnifiedOrchestratorAdapter
}
```

**Missing**:

- Conversion from `TaskSubmissionRequest` → `WorkingSpec`
- Call to `UnifiedOrchestratorAdapter.orchestrate_task()`
- Task ID tracking and status persistence

**Files Affected**:

- `iterations/v3/data-interfaces-adapters/src/bin/api-server.rs` (lines 433-470)
- Need new handler that converts request → WorkingSpec → UnifiedOrchestratorAdapter

#### 2. WorkingSpec Generation from API Requests

**Problem**: No conversion from API task description to WorkingSpec

**Missing Component**:

- `TaskRequestToWorkingSpecConverter`
- Should use `ResearchServiceAdapter.generate_working_spec()` or create inline
- Must populate: id, title, description, goals, risk_tier, acceptance_criteria, scope, change_budget

**Location**: New file or extend `orchestration_adapter.rs`

#### 3. Task Status Tracking

**Problem**: `UnifiedOrchestratorAdapter.get_task_status()` returns placeholder

**Current State**:

```rust
// orchestration_adapter.rs:476
async fn get_task_status(&self, task_id: &Uuid) -> Result<TaskStatus, ServiceError> {
    // TODO: Implement actual status retrieval
    Ok(TaskStatus { status: TaskStatusEnum::Running, ... })
}
```

**Missing**:

- Integration with UnifiedOrchestrator's state persistence
- Real-time status updates from execution state
- Progress percentage calculation

#### 4. Task Lifecycle Management

**Problem**: Pause/resume/cancel not implemented

**Missing**:

- `pause_task()` - Signal orchestrator to pause execution
- `resume_task()` - Resume from checkpoint
- `cancel_task()` - Graceful cancellation with cleanup

**Integration Points**:

- UnifiedOrchestrator needs pause/resume/cancel methods
- State persistence must support pause/resume
- Worktree cleanup on cancel

#### 5. Main Entry Point

**Problem**: `agent-orchestration/src/main.rs` is placeholder

**Current State**:

```rust
// main.rs:14-23
// TODO: Initialize the orchestration service
// This is a placeholder implementation
```

**Missing**:

- Service initialization with all dependencies
- HTTP server setup (or delegate to api-server)
- Configuration loading
- Graceful shutdown

#### 6. Configuration & Environment Setup

**Missing**:

- Environment variable configuration
- Database connection initialization in main
- Model configuration (CoreML, Ollama, etc.)
- Worker pool configuration

## Connection Points Needed

### Connection Point 1: API Request → WorkingSpec

**Location**: `data-interfaces-adapters/src/bin/api-server.rs`

**Implementation**:

```rust
async fn submit_task_handler(
    State(state): State<AppState>,
    Json(payload): Json<JsonValue>,
) -> Result<Json<JsonValue>, StatusCode> {
    // 1. Extract task description from payload
    // 2. Convert to WorkingSpec (use ResearchServiceAdapter or inline)
    // 3. Call UnifiedOrchestratorAdapter.orchestrate_task()
    // 4. Return task_id and status
}
```

**Dependencies**:

- ResearchServiceAdapter for WorkingSpec generation (optional)
- Or create WorkingSpec directly from task description

### Connection Point 2: UnifiedOrchestratorAdapter → UnifiedOrchestrator

**Status**: ✅ **EXISTS** - `orchestrate_task()` calls `orchestrator.execute_plan()`

**Location**: `data-interfaces-adapters/src/orchestration_adapter.rs:430`

**Note**: This connection works, but needs status tracking integration

### Connection Point 3: Execution State → API Status Endpoints

**Missing**: Real-time status updates

**Implementation Needed**:

- UnifiedOrchestrator must expose execution state
- UnifiedOrchestratorAdapter must query state for status
- API handlers must return real status, not placeholders

### Connection Point 4: Task Lifecycle → UnifiedOrchestrator

**Missing**: Pause/resume/cancel methods in UnifiedOrchestrator

**Implementation Needed**:

```rust
impl UnifiedOrchestrator {
    async fn pause_execution(&self, plan_id: Uuid) -> Result<()>;
    async fn resume_execution(&self, plan_id: Uuid) -> Result<()>;
    async fn cancel_execution(&self, plan_id: Uuid) -> Result<()>;
}
```

## Implementation Plan

### Phase 1: Core API Integration (Priority 1)

**Goal**: Make API server functional end-to-end

**Tasks**:

1. **Create WorkingSpec converter**

   - File: `data-interfaces-adapters/src/working_spec_converter.rs`
   - Convert `TaskSubmissionRequest` → `WorkingSpec`
   - Use ResearchServiceAdapter if available, otherwise create inline

2. **Update submit_task_handler**

   - File: `data-interfaces-adapters/src/bin/api-server.rs`
   - Use UnifiedOrchestratorAdapter instead of legacy RestApi
   - Convert request → WorkingSpec → orchestrate_task()
   - Return task_id and initial status

3. **Implement status tracking**

   - File: `data-interfaces-adapters/src/orchestration_adapter.rs`
   - Query UnifiedOrchestrator state persistence
   - Return real TaskStatus with progress

**Estimated Effort**: 4-6 hours

### Phase 2: Task Lifecycle Management (Priority 2)

**Goal**: Enable pause/resume/cancel operations

**Tasks**:

1. **Add lifecycle methods to UnifiedOrchestrator**

   - File: `agent-orchestration/src/orchestration/unified_orchestrator.rs`
   - Implement pause_execution(), resume_execution(), cancel_execution()
   - Integrate with state persistence

2. **Wire up adapter methods**

   - File: `data-interfaces-adapters/src/orchestration_adapter.rs`
   - Implement pause_task(), resume_task(), cancel_task()
   - Call UnifiedOrchestrator methods

3. **Update API handlers**

   - File: `data-interfaces-adapters/src/bin/api-server.rs`
   - Wire pause/resume/cancel endpoints to adapter

**Estimated Effort**: 3-4 hours

### Phase 3: Main Entry Point (Priority 3)

**Goal**: Functional standalone orchestration service

**Tasks**:

1. **Implement main.rs**

   - File: `agent-orchestration/src/main.rs`
   - Initialize UnifiedOrchestrator with dependencies
   - Load configuration from environment
   - Optionally start HTTP server (or delegate to api-server)

2. **Configuration management**

   - Create OrchestrationConfig struct
   - Load from environment variables
   - Database, model, worker pool configuration

**Estimated Effort**: 2-3 hours

### Phase 4: Testing & Validation (Priority 4)

**Goal**: Verify end-to-end flow works

**Tasks**:

1. **Integration test**

   - Submit task via API
   - Verify execution starts
   - Check status updates
   - Verify completion

2. **Error handling**

   - Test invalid requests
   - Test pause/resume edge cases
   - Test cancellation during execution

**Estimated Effort**: 2-3 hours

## File Changes Summary

### New Files Needed

1. `data-interfaces-adapters/src/working_spec_converter.rs` - API → WorkingSpec conversion
2. `agent-orchestration/src/config.rs` - Configuration management (if not exists)

### Files to Modify

1. `data-interfaces-adapters/src/bin/api-server.rs`

   - Update submit_task_handler (lines 433-470)
   - Wire UnifiedOrchestratorAdapter

2. `data-interfaces-adapters/src/orchestration_adapter.rs`

   - Implement get_task_status() (line 476)
   - Implement pause/resume/cancel (lines 489-502)

3. `agent-orchestration/src/orchestration/unified_orchestrator.rs`

   - Add pause_execution(), resume_execution(), cancel_execution()

4. `agent-orchestration/src/main.rs`

   - Replace placeholder with real initialization

## Dependencies & Prerequisites

### Required Services

- PostgreSQL database (for state persistence)
- Ollama or CoreML model runtime (for council judges)
- MCP worker pool (for task execution)

### Configuration Needed

- `DATABASE_URL` - PostgreSQL connection string
- `OLLAMA_BASE_URL` - Ollama API endpoint (if using)
- `WORKER_POOL_SIZE` - Number of concurrent workers
- `COUNCIL_MODEL` - Model for council judges

## Success Criteria

### Phase 1 Complete When:

- [ ] POST /api/v1/tasks accepts task description
- [ ] Task converts to WorkingSpec successfully
- [ ] UnifiedOrchestrator.execute_plan() is called
- [ ] Task ID returned to client
- [ ] GET /api/v1/tasks/:task_id returns real status

### Phase 2 Complete When:

- [ ] POST /api/v1/tasks/:task_id/pause pauses execution
- [ ] POST /api/v1/tasks/:task_id/resume resumes from checkpoint
- [ ] POST /api/v1/tasks/:task_id/cancel cancels gracefully

### Phase 3 Complete When:

- [ ] `cargo run --bin agent-orchestration` starts service
- [ ] Service initializes all dependencies
- [ ] Configuration loads from environment

### Full Integration Complete When:

- [ ] End-to-end test: API → Orchestrator → Workers → Council → Results
- [ ] Status tracking works throughout execution
- [ ] Pause/resume/cancel work correctly
- [ ] Error handling is robust

## Risk Assessment

### Low Risk

- WorkingSpec conversion (straightforward mapping)
- Status tracking (query existing state)

### Medium Risk

- Pause/resume implementation (state management complexity)
- Configuration loading (dependency initialization)

### High Risk

- None identified - all components exist, just need wiring

## Next Steps

1. **Start with Phase 1** - Get basic API → Orchestrator flow working
2. **Test incrementally** - Verify each connection point before moving on
3. **Add logging** - Ensure visibility into execution flow
4. **Document API** - Update API documentation with new endpoints

## Estimated Total Effort

- Phase 1: 4-6 hours
- Phase 2: 3-4 hours  
- Phase 3: 2-3 hours
- Phase 4: 2-3 hours
- **Total: 11-16 hours** (1.5-2 days)