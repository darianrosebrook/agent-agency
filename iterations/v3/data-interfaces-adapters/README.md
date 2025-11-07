# Data Interfaces Adapters

**Concrete Service Implementations for Data Interfaces**

This crate provides concrete implementations of the service trait interfaces defined in `data-interfaces`. It bridges the gap between interface contracts and actual implementation crates, enabling dependency injection patterns.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│ Interface Layer (Contracts Only)                        │
├─────────────────────────────────────────────────────────┤
│ data-interfaces                                         │
│    - Service trait definitions                          │
│    - Zero implementation dependencies                   │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Adapter Layer (Implementation Bridges)                  │
├─────────────────────────────────────────────────────────┤
│ data-interfaces-adapters ✅ (THIS CRATE)                │
│    - ResearchServiceAdapter ✅                          │
│    - OrchestrationServiceAdapter ✅                     │
│    - MemoryServiceAdapter ✅                            │
│    - WorkerServiceAdapter (structure ready)             │
│    - ProgressTrackingServiceAdapter (placeholder)      │
│    - ServiceContainer for DI ✅                         │
│    - Binaries moved ✅                                   │
└─────────────────────────────────────────────────────────┘
                        ↓
┌─────────────────────────────────────────────────────────┐
│ Implementation Layer                                    │
├─────────────────────────────────────────────────────────┤
│ agent-research                                          │
│ agent-orchestration                                     │
│ agent-workers                                           │
│ agent-memory                                            │
└─────────────────────────────────────────────────────────┘
```

## Service Adapters

### ResearchServiceAdapter

Implements `ResearchService` using `agent-research`'s `PlanningAgent`:

```rust
use data_interfaces_adapters::ResearchServiceAdapter;

let adapter = ResearchServiceAdapter::with_defaults();
let result = adapter.execute_task(task_request).await?;
```

### OrchestrationServiceAdapter (UnifiedOrchestratorAdapter)

Implements `OrchestrationService` using `agent-orchestration`'s `UnifiedOrchestrator`:

- **Factory Pattern**: `create_with_dependencies()` initializes all required components
- **Dependency Injection**: Accepts optional `DatabaseClient` for persistence
- **Database Adapter**: Uses `DatabaseOperationsAdapter` to bridge database operations (partial implementation)
- **Orchestrator Access**: Exposes `orchestrator()` method for creating `UnifiedOrchestratorTaskExecutor`

```rust
use data_interfaces_adapters::UnifiedOrchestratorAdapter;

// Create adapter with database client
let adapter = UnifiedOrchestratorAdapter::create_with_dependencies(Some(db_client)).await?;

// Execute task
let result = adapter.orchestrate_task(working_spec, task_context).await?;

// Access underlying orchestrator for TaskExecutor bridge
let orchestrator = adapter.orchestrator();
```

### UnifiedOrchestratorTaskExecutor

Implements `TaskExecutor` trait (from `data-infrastructure`) to bridge `OrchestratorService` with `UnifiedOrchestrator`:

- **Type Conversion**: Converts `TaskDescriptor` → `WorkingSpec`
- **Execution**: Delegates to `UnifiedOrchestrator::execute_plan()`
- **Result Extraction**: Extracts `ExecutionArtifacts` from `ExecutionResult`

```rust
use data_interfaces_adapters::UnifiedOrchestratorTaskExecutor;

let executor = Arc::new(UnifiedOrchestratorTaskExecutor::new(orchestrator));
let orchestrator_service = OrchestratorService::new(db_client)
    .with_task_executor(executor);
```

### DatabaseOperationsAdapter

Adapts `data-infrastructure::DatabaseClient` to `agent-orchestration::DatabaseOperations` trait:

- **Partial Implementation**: Core methods implemented, some marked as PLACEHOLDER
- **Type Mapping**: Maps between agent-orchestration types and data-infrastructure types
- **Future Work**: Database queries for execution plans, audit trails, workers, judges, waivers

**Status**: Partial implementation - basic structure in place, database queries pending

### MemoryServiceAdapter

Implements `MemoryService` using `agent-memory`'s `MemoryManager`:

```rust
use data_interfaces_adapters::MemoryServiceAdapter;

let adapter = MemoryServiceAdapter::new(db_pool).await?;
let memories = adapter.query_memories(query).await?;
```

## Service Container

The `ServiceContainer` provides a convenient way to initialize all services:

```rust
use data_interfaces_adapters::ServiceContainer;

// Initialize with default adapters
let services = ServiceContainer::new();

// Use services
let task_result = services.research_service.execute_task(request).await?;
let status = services.orchestration_service.get_task_status(task_id).await?;
```

### Custom Service Injection

For testing or advanced usage:

```rust
use data_interfaces_adapters::ServiceContainer;
use std::sync::Arc;

let custom_research = Arc::new(MyCustomResearchService::new());
let custom_orchestration = Arc::new(MyCustomOrchestrationService::new());

let services = ServiceContainer::with_services(
    custom_research,
    custom_orchestration,
    worker_service,
    progress_service,
    None, // memory service optional
);
```

## Binaries

This crate includes all binaries that require implementation dependencies:

### agent-agency-cli

Basic command-line interface:

```bash
cargo run --bin agent-agency-cli -- --help
```

### agent-agency-api-server

REST API server providing comprehensive observation endpoints for the orchestrator:

```bash
cargo run --bin agent-agency-api-server \
    -- --host 127.0.0.1 \
    --port 8080 \
    --enable-cors
```

#### CRITICAL: Observational API Design

**This API is designed for OBSERVATION, not manipulation.**

The API acts as a "doctor's MRI machine" - it observes what's happening inside
the orchestrator without directly controlling execution. This preserves research
integrity by ensuring the orchestrator maintains full autonomy over its execution
lifecycle.

**Design Principles:**

1. **Observation Only**: All endpoints observe orchestrator state, never manipulate it directly
2. **Request-Based Control**: Control operations (pause/resume/cancel) are requests that
   are logged in chain-of-thought, but the orchestrator decides whether to honor them
3. **Research Integrity**: No direct manipulation of execution state - orchestrator maintains
   full control over its own execution lifecycle
4. **Agent Autonomy**: Agents use their own connections to task execution, not through the API

**What This Means:**

- **Task Submission**: Requests orchestrator to start a task (orchestrator handles execution)
- **State Observation**: Query task status, chain of thought, council decisions, worker actions
- **Control Requests**: Request pause/resume/cancel (orchestrator decides if safe)
- **Never Manipulate**: Never directly change execution state - only observe and request

**Why This Matters:**

Direct manipulation of orchestrator execution state would compromise research integrity.
By maintaining strict observation boundaries, we ensure that:
- Orchestrator decisions are autonomous and reproducible
- Research results are not contaminated by external manipulation
- The orchestrator's chain of thought accurately reflects its own reasoning
- Agents maintain their own execution connections independently

See the API server source code (`src/bin/api-server.rs`) for detailed documentation.

## Dependencies

This crate includes implementation dependencies that are not allowed in `data-interfaces`:

- `agent-research` - Research service implementation
- `agent-orchestration` - Orchestration service implementation
- `agent-workers` - Worker service implementation
- `agent-memory` - Memory service implementation
- `data-infrastructure` - Infrastructure utilities

## Status

### Implemented Adapters ✅

- **ResearchServiceAdapter** - Complete implementation
- **OrchestrationServiceAdapter** - Complete implementation
- **MemoryServiceAdapter** - Complete implementation

### Placeholder Adapters ⚠️

- **WorkerServiceAdapter** - Structure ready, needs WorkerExecutor API integration
- **ProgressTrackingServiceAdapter** - Placeholder (may be handled by orchestration)

## Notes

### RestApi Integration

Currently, `RestApi` from `data-infrastructure` expects concrete types (`Arc<Orchestrator>`) rather than trait objects. The adapters bridge to concrete types for now. Future refactoring will update `RestApi` to accept service traits.

### Future Improvements

1. **Refactor RestApi** - Accept service traits instead of concrete types
2. **Complete WorkerServiceAdapter** - Integrate with WorkerExecutor API
3. **Complete ProgressTrackingServiceAdapter** - Implement actual progress tracking
4. **Test End-to-End** - Verify binaries work with adapters

## Related

- **data-interfaces** - Service trait definitions (contracts only)
- **agent-agency-contracts** - Core type definitions and contracts
- **agent-research** - Research service implementation
- **agent-orchestration** - Orchestration service implementation

