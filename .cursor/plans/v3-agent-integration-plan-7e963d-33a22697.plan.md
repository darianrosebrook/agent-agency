<!-- 33a22697-e539-4652-a677-a49ef46ddd79 409534e0-7f54-470c-a2d3-eb9f2b406b5a -->
# V3 Agent Integration Plan

## Executive Summary

The v3 agent system has most components implemented but lacks the final integration wiring to make it work cohesively. This plan identifies what exists, what's missing, and the specific connection points needed to create a functional end-to-end system matching `theory.md` requirements.

## Current State Analysis

### What Exists (Functional Components)

#### Core Orchestration System

- **UnifiedOrchestrator** (`agent-orchestration/src/orchestration/unified_orchestrator.rs`)
  - Full planning/execution/council integration
  - Worktree management, refinement loops, CAWS adjudication
  - State persistence for pause/resume/cancel
  - Memory system integration (optional feature)

- **Planning Components** (`agent-orchestration/src/planning/`)
  - PlanGenerator, PlanExecutor, ParallelCoordinator
  - CouncilIntegration, WorkerAssignmentStrategy
  - ReflexiveLearner, WorktreeManager
  - All components created via PlanningSystemFactory

#### Worker System

- **MCPWorkerPool** (`agent-workers/src/`)
  - MCP-based worker management
  - TaskExecutor with HTTP execution
  - Worker discovery, health monitoring, capability matching
  - Circuit breaker patterns

#### Council System

- **Council** (`agent-orchestration/src/council/`)
  - Multi-judge constitutional oversight
  - Verdict aggregation, consensus strategies
  - EthicsJudge, QualityAssuranceJudge, SecurityJudge
  - Risk-tiered evaluation framework

#### API Layer

- **RestApi** (`data-infrastructure/src/api/server.rs`)
  - Task submission endpoint (`POST /api/v1/tasks`)
  - Task status, cancellation, pause/resume endpoints
  - Progress tracking integration

- **OrchestratorService** (`data-infrastructure/src/orchestrator_service.rs`)
  - Observational API design
  - Task state management
  - Chain-of-thought tracking
  - Council decision logging

#### Adapter Layer

- **UnifiedOrchestratorAdapter** (`data-interfaces-adapters/src/orchestration_adapter.rs`)
  - Implements `OrchestrationService` trait
  - Connects to UnifiedOrchestrator
  - Task status, pause/resume/cancel support
  - Factory method `create_with_dependencies()` exists

### What's Missing (Critical Gaps)

#### 1. TaskExecutor Implementation

**Location**: `data-infrastructure/src/orchestrator_service.rs`

**Problem**: `OrchestratorService` has `TaskExecutor` trait but no implementation that connects to `UnifiedOrchestrator`.

**Current State**:

```rust
pub trait TaskExecutor: Send + Sync {
    async fn execute_task(&self, task_descriptor: &TaskDescriptor) -> Result<ExecutionArtifacts>;
}

// OrchestratorService uses Option<Arc<dyn TaskExecutor>>
// But no concrete implementation exists
```

**Required**: Create `UnifiedOrchestratorTaskExecutor` that:

- Implements `TaskExecutor` trait
- Wraps `UnifiedOrchestrator`
- Converts `TaskDescriptor` to `WorkingSpec`
- Calls `orchestrator.execute_plan()`
- Returns `ExecutionArtifacts`

#### 2. API Server Initialization

**Location**: `data-infrastructure/src/api/server.rs` and main entry point

**Problem**: API server creates `OrchestratorService` but doesn't wire `UnifiedOrchestrator` via `TaskExecutor`.

**Current State**:

```rust
// RestApi::with_orchestrator_service() creates OrchestratorService
// But OrchestratorService has task_executor: None
// No initialization code wires UnifiedOrchestrator
```

**Required**:

- Create `UnifiedOrchestrator` instance in main entry point
- Wrap it in `UnifiedOrchestratorTaskExecutor`
- Pass to `OrchestratorService::with_task_executor()`
- Initialize API server with configured service

#### 3. Database Adapter Implementation

**Location**: `data-interfaces-adapters/src/orchestration_adapter.rs`

**Problem**: `UnifiedOrchestratorAdapter::create_with_dependencies()` uses `StubDatabaseOperations` even when `db_client` is provided.

**Current State**:

```rust
let db_ops: Arc<dyn DatabaseOperations> = if let Some(db_client) = db_client {
    warn!("Database client provided but adapter not yet fully implemented");
    Arc::new(StubDatabaseOperations) // Always uses stub!
} else {
    Arc::new(StubDatabaseOperations)
};
```

**Required**: Create `DatabaseOperationsAdapter` that:

- Implements `agent_orchestration::planning::DatabaseOperations` trait
- Wraps `data_infrastructure::DatabaseClient`
- Maps between data-infrastructure types and agent-orchestration types
- Provides real persistence for execution plans, audit trails, etc.

#### 4. Main Entry Point Integration

**Location**: `data-infrastructure/src/main.rs` or new binary

**Problem**: No single entry point that initializes all components together.

**Current State**: Components exist but aren't wired together in a main binary.

**Required**: Create main entry point that:

- Initializes database connection
- Creates `UnifiedOrchestrator` via adapter factory
- Wraps in `TaskExecutor` implementation
- Initializes `OrchestratorService` with executor
- Starts API server with configured service
- Handles graceful shutdown

#### 5. Worker Endpoint Configuration

**Location**: `agent-workers/src/executor.rs`

**Problem**: Worker execution uses HTTP but worker endpoints aren't configured/registered.

**Current State**:

```rust
let worker_base_url = self.resolve_worker_endpoint(worker_id).await
    .unwrap_or_else(|_| format!("http://worker-{}", worker_id));
```

**Required**:

- Worker registry/discovery system
- Configuration for worker endpoints
- Health check integration
- Service discovery mechanism

## Connection Points Required

### Connection Point 1: API → OrchestratorService → UnifiedOrchestrator

**Flow**:

```
POST /api/v1/tasks
  → RestApi::submit_task()
  → OrchestratorService::execute_task()
  → TaskExecutor::execute_task() [MISSING]
  → UnifiedOrchestrator::execute_plan()
```

**Files to Modify**:

- `data-infrastructure/src/orchestrator_service.rs` - Add `UnifiedOrchestratorTaskExecutor`
- `data-infrastructure/src/api/server.rs` - Wire executor in initialization
- `data-infrastructure/src/main.rs` - Create orchestrator and wire together

### Connection Point 2: UnifiedOrchestrator → Workers

**Flow**:

```
UnifiedOrchestrator::execute_plan()
  → PlanExecutor::execute_milestone()
  → WorkerExecutionBridge::execute_task()
  → MCPWorkerPool::execute_task()
  → TaskExecutor (worker) HTTP call
```

**Status**: Mostly connected, but worker endpoints need configuration.

**Files to Modify**:

- `agent-workers/src/executor.rs` - Worker endpoint resolution
- Configuration system for worker registry

### Connection Point 3: Database Persistence

**Flow**:

```
UnifiedOrchestrator operations
  → DatabaseOperations trait methods
  → DatabaseOperationsAdapter [MISSING]
  → data_infrastructure::DatabaseClient
  → PostgreSQL
```

**Files to Modify**:

- `data-interfaces-adapters/src/orchestration_adapter.rs` - Implement `DatabaseOperationsAdapter`
- Create type mapping layer between agent-orchestration and data-infrastructure types

### Connection Point 4: Council Integration

**Flow**:

```
UnifiedOrchestrator::execute_plan()
  → CouncilIntegration::review_plan()
  → Council::start_session()
  → Judges evaluate
  → VerdictAggregator aggregates
  → Decision returned
```

**Status**: Fully connected, no changes needed.

## Implementation Tasks

### Task 1: Create UnifiedOrchestratorTaskExecutor

**File**: `data-infrastructure/src/orchestrator_service.rs`

**Implementation**:

```rust
pub struct UnifiedOrchestratorTaskExecutor {
    orchestrator: Arc<UnifiedOrchestrator>,
}

impl TaskExecutor for UnifiedOrchestratorTaskExecutor {
    async fn execute_task(&self, task_descriptor: &TaskDescriptor) -> Result<ExecutionArtifacts> {
        // Convert TaskDescriptor to WorkingSpec
        // Call orchestrator.execute_plan()
        // Convert ExecutionResult to ExecutionArtifacts
    }
}
```

**Dependencies**:

- `UnifiedOrchestrator` from `agent-orchestration`
- Conversion logic for `TaskDescriptor` → `WorkingSpec`

### Task 2: Implement DatabaseOperationsAdapter

**File**: `data-interfaces-adapters/src/orchestration_adapter.rs` (or new file)

**Implementation**:

```rust
pub struct DatabaseOperationsAdapter {
    db_client: Arc<DatabaseClient>,
}

#[async_trait]
impl agent_orchestration::planning::DatabaseOperations for DatabaseOperationsAdapter {
    // Implement all trait methods
    // Map between agent-orchestration types and data-infrastructure types
}
```

**Dependencies**:

- Type mapping layer
- SQL query implementations for each operation

### Task 3: Create Main Entry Point

**File**: `data-infrastructure/src/main.rs` (or new binary)

**Implementation**:

```rust
#[tokio::main]
async fn main() {
    // 1. Initialize database
    // 2. Create UnifiedOrchestrator via UnifiedOrchestratorAdapter::create_with_dependencies()
    // 3. Wrap in UnifiedOrchestratorTaskExecutor
    // 4. Create OrchestratorService with executor
    // 5. Initialize RestApi with service
    // 6. Start HTTP server
}
```

**Dependencies**: All previous tasks

### Task 4: Worker Endpoint Configuration

**File**: `agent-workers/src/executor.rs` and configuration

**Implementation**:

- Worker registry system
- Environment variable or config file for worker endpoints
- Service discovery integration

**Dependencies**: Configuration system

## Testing Strategy

### Integration Test 1: End-to-End Task Execution

- Submit task via API
- Verify UnifiedOrchestrator receives it
- Verify workers execute
- Verify results returned

### Integration Test 2: Database Persistence

- Execute task
- Verify execution plan persisted
- Verify audit trail entries created
- Verify state can be retrieved

### Integration Test 3: Council Integration

- Submit task requiring council review
- Verify council evaluates
- Verify verdict affects execution

## Risk Assessment

### High Risk

- **Database adapter complexity**: Type mapping between two different type systems
- **UnifiedOrchestrator initialization**: Many dependencies, complex factory

### Medium Risk

- **TaskExecutor conversion**: TaskDescriptor → WorkingSpec mapping
- **Worker endpoint configuration**: Service discovery complexity

### Low Risk

- **API server wiring**: Straightforward dependency injection
- **Council integration**: Already connected

## Success Criteria

1. **API → Orchestrator → Workers**: Task submitted via API executes through UnifiedOrchestrator to workers
2. **Database Persistence**: Execution plans and audit trails persist to PostgreSQL
3. **Council Review**: Tasks go through council evaluation before execution
4. **State Management**: Task status, pause/resume/cancel work end-to-end
5. **Error Handling**: Failures propagate correctly with proper error messages

## Next Steps

1. Implement `UnifiedOrchestratorTaskExecutor` (Task 1)
2. Wire in main entry point (Task 3) - can use stub database initially
3. Test end-to-end flow without database persistence
4. Implement `DatabaseOperationsAdapter` (Task 2)
5. Test with real database persistence
6. Configure worker endpoints (Task 4)
7. Full integration testing
8. **Update Documentation** (Task 5) - Document connected infrastructure

## Documentation Updates Required

### Task 5: Update Documentation

**Purpose**: Document the currently connected infrastructure so developers understand the system architecture.

**Files to Update**:

1. **`iterations/v3/README.md`**

   - Add section: "System Architecture & Connection Flow"
   - Document: API → OrchestratorService → UnifiedOrchestrator → Workers flow
   - Include connection diagram showing all components
   - Document MCP-based execution path vs HTTP fallback

2. **`iterations/v3/agent-orchestration/README.md`**

   - Document `WorkerExecutionBridge` and its role
   - Explain how `UnifiedOrchestrator` connects to `MCPWorkerPool`
   - Document type conversions (Milestone ↔ TaskDefinition)

3. **`iterations/v3/agent-workers/README.md`**

   - Document `MCPWorkerPool` architecture
   - Explain MCP protocol integration
   - Document shared memory system integration
   - Document HTTP fallback path for distributed workers

4. **`iterations/v3/data-infrastructure/README.md`**

   - Document `OrchestratorService` observational API design
   - Explain `TaskExecutor` trait and its role
   - Document connection to `UnifiedOrchestrator`

5. **`iterations/v3/data-interfaces-adapters/README.md`**

   - Document `UnifiedOrchestratorAdapter` factory pattern
   - Explain adapter's role in dependency injection
   - Document database adapter stub status

**Content to Include**:

- Connection flow diagrams (Mermaid)
- Component responsibility matrix
- Current status of each connection point
- Configuration requirements
- Known limitations/gaps