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

### OrchestrationServiceAdapter

Implements `OrchestrationService` using `agent-orchestration`'s `OrchestrationAdapter`:

```rust
use data_interfaces_adapters::OrchestrationServiceAdapter;

let adapter = OrchestrationServiceAdapter::with_defaults();
let result = adapter.orchestrate_task(task_id).await?;
```

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

REST API server:

```bash
cargo run --bin agent-agency-api-server \
    -- --host 127.0.0.1 \
    --port 8080 \
    --enable-cors
```

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

