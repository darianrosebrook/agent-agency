# Critical Infrastructure Implementation Tickets

**Created:** October 28, 2025
**Status:** Active Implementation
**Priority:** Critical Path - Core Execution

---

## TICKET-001: Worker Pool Integration

**Priority:** P0 - Critical
**Tier:** 1 (Core Execution Path)
**Assignee:** TBD
**Estimate:** 2-3 days
**Dependencies:** MCPWorkerPool trait implementation in `agent-workers`

### Description
Integrate the orchestrator with actual MCPWorkerPool to enable worker discovery and assignment. Currently returns empty worker list, blocking task execution.

### Location
- File: `iterations/v3/agent-orchestration/src/planning/orchestrator_integration.rs`
- Lines: 526-625

### Current State
```rust
// For now, return empty list
let workers: Vec<crate::planning::plan_executor::WorkerInfo> = Vec::new();
```

### Acceptance Criteria
- [ ] MCPWorkerPool is queried for available workers
- [ ] WorkerHandle objects are converted to WorkerInfo format
- [ ] Worker capabilities are extracted from worker metadata
- [ ] Worker load is calculated from active tasks
- [ ] Worker health status is retrieved
- [ ] Unit tests added with mock worker pool
- [ ] Integration tests added with real worker pool
- [ ] Orchestrator can discover and assign workers

### Dependencies
- MCPWorkerPool trait implementation in agent-workers crate
- Worker registry service (optional for initial implementation)

---

## TICKET-002: Worker Execution Logic

**Priority:** P0 - Critical
**Tier:** 1 (Core Execution Path)
**Assignee:** TBD
**Estimate:** 3-4 days
**Dependencies:** MCP tool integration, execution context

### Description
Implement real worker execution logic instead of placeholder simulation. Currently just sleeps for 10ms, blocking actual task execution.

### Location
- File: `iterations/v3/agent-workers/src/worker.rs`
- Lines: 81-100

### Current State
```rust
// PLACEHOLDER: Real execution logic would go here
// For now, simulate execution
let start_time = std::time::Instant::now();
tokio::time::sleep(std::time::Duration::from_millis(10)).await;
```

### Acceptance Criteria
- [ ] Worker's execute_subtask method is invoked with proper parameters
- [ ] Worker errors and timeouts are handled gracefully
- [ ] Execution metrics are tracked (start time, duration, resource usage)
- [ ] Cancellation support is implemented
- [ ] Progress reporting callbacks are integrated
- [ ] Telemetry system integration for observability
- [ ] Unit tests for execution paths
- [ ] Integration tests with real workers
- [ ] Tasks execute successfully end-to-end

### Dependencies
- MCP tool integration
- Execution context setup
- Telemetry/observability system (optional)

---

## TICKET-003: Worker Health & Performance Metrics

**Priority:** P0 - Critical
**Tier:** 1 (Core Execution Path)
**Assignee:** TBD
**Estimate:** 2-3 days
**Dependencies:** Worker status API, metrics collection

### Description
Implement actual worker health and performance metrics collection. Currently returns placeholder values, blocking intelligent worker selection and load balancing.

### Location
- File: `iterations/v3/agent-orchestration/src/planning/orchestrator_integration.rs`
- Lines: 603-625

### Current State
```rust
// Mock health and performance for now
let health = crate::planning::plan_executor::WorkerHealth::Healthy;
// PLACEHOLDER
avg_completion_time_ms: 1000.0, // PLACEHOLDER
```

### Acceptance Criteria
- [ ] Worker health status is queried from worker
- [ ] Performance metrics are retrieved (tasks completed, failed, avg time)
- [ ] Success rate is calculated from performance data
- [ ] Missing performance data is handled gracefully
- [ ] Unit tests with mock worker data
- [ ] Integration tests with real worker performance
- [ ] Intelligent worker selection works based on metrics
- [ ] Load balancing decisions are data-driven

### Dependencies
- Worker status API implementation
- Metrics collection system
- Worker health monitoring

---

## TICKET-004: Task Queue Implementation

**Priority:** P0 - Critical
**Tier:** 1 (Core Execution Path)
**Assignee:** TBD
**Estimate:** 3-4 days
**Dependencies:** Task queue service

### Description
Implement proper task queue tracking for active and queued tasks. Currently returns hardcoded zeros, blocking task lifecycle management.

### Location
- File: `iterations/v3/agent-workers/src/executor.rs`
- Lines: 1389-1390

### Current State
```rust
active_tasks: 0, // TODO: Track active tasks separately when task queue is implemented
queued_tasks: 0, // TODO: Track queued tasks separately when task queue is implemented
```

### Acceptance Criteria
- [ ] Active tasks are tracked separately from queued tasks
- [ ] Task queue service provides real-time counts
- [ ] Task lifecycle events update queue metrics
- [ ] Queue depth monitoring is implemented
- [ ] Task prioritization affects queue ordering
- [ ] Unit tests for queue operations
- [ ] Integration tests with task execution
- [ ] Proper task lifecycle management works

### Dependencies
- Task queue service implementation
- Task lifecycle event system
- Queue persistence (optional)

---

## TICKET-005: CAWS Runtime Validator

**Priority:** P1 - High
**Tier:** 2 (Quality & Validation)
**Assignee:** TBD
**Estimate:** 2-3 days
**Dependencies:** CAWS validation library

### Description
Replace placeholder runtime validator with real CAWS compliance validation. Currently uses MockCawsRuntimeValidator, blocking quality gate enforcement.

### Location
- File: `iterations/v3/agent-orchestration/src/lib.rs`
- Lines: 318-324

### Current State
```rust
// PLACEHOLDER: runtime_validator - proper implementation needed
Arc::new(MockCawsRuntimeValidator),
```

### Acceptance Criteria
- [ ] CAWS runtime validator crate is integrated
- [ ] Real CawsRuntimeValidator instance is created
- [ ] Validator configured with CAWS rules and settings
- [ ] Unit tests with mock validators
- [ ] Integration tests with real CAWS validation
- [ ] Quality gates are enforced at runtime
- [ ] CAWS compliance validation works correctly

### Dependencies
- CAWS runtime validator crate
- CAWS rule definitions
- Configuration system for validator settings

---

## TICKET-006: Verdict Writer

**Priority:** P1 - High
**Tier:** 2 (Quality & Validation)
**Assignee:** TBD
**Estimate:** 2-3 days
**Dependencies:** Provenance system integration

### Description
Replace placeholder verdict writer with real verdict persistence. Currently uses MockVerdictWriter, blocking provenance tracking and audit trails.

### Location
- File: `iterations/v3/agent-orchestration/src/lib.rs`
- Lines: 327-333

### Current State
```rust
// PLACEHOLDER: verdict_writer - proper implementation needed
Arc::new(MockVerdictWriter {}),
```

### Acceptance Criteria
- [ ] Verdict storage system is integrated (database, file system, etc.)
- [ ] Verdict persistence with proper error handling
- [ ] Verdict retrieval and query capabilities
- [ ] Unit tests with mock verdict storage
- [ ] Integration tests with real verdict persistence
- [ ] Provenance tracking works correctly
- [ ] Audit trails are maintained

### Dependencies
- Provenance system implementation
- Verdict storage backend (database/file system)
- Error handling for storage operations

---

## TICKET-007: TaskExecutor Factory

**Priority:** P1 - High
**Tier:** 2 (Quality & Validation)
**Assignee:** TBD
**Estimate:** 3-4 days
**Dependencies:** TaskExecutor trait implementation

### Description
Implement real TaskExecutor factory instead of panicking. Currently throws error on creation, blocking task execution initialization.

### Location
- File: `iterations/v3/agent-orchestration/src/lib.rs`
- Lines: 341-352

### Current State
```rust
// PLACEHOLDER: Real TaskExecutor implementation needed
panic!("TaskExecutor factory not implemented - requires agent-workers integration")
```

### Acceptance Criteria
- [ ] Dependency issues with agent-workers crate are resolved
- [ ] TaskExecutor instances are created from agent-workers
- [ ] Executors configured with proper settings and capabilities
- [ ] Executor creation errors handled gracefully
- [ ] Unit tests with mock executors
- [ ] Integration tests with real TaskExecutor instances
- [ ] Task execution initialization works correctly

### Dependencies
- TaskExecutor trait implementation in agent-workers
- Configuration system for executor settings
- Error handling for creation failures

---

## TICKET-008: CoreML Embedding Provider

**Priority:** P2 - Medium
**Tier:** 3 (Data & Memory)
**Assignee:** TBD
**Estimate:** 4-5 days
**Dependencies:** CoreML model loading, tokenization

### Description
Implement CoreML embedding provider to replace deprecated Ollama provider. Currently uses dummy embeddings, blocking vector embeddings and similarity search.

### Location
- File: `iterations/v3/data-infrastructure/src/embedding/provider.rs`
- Lines: 72-92, 421-423

### Current State
```rust
// PLACEHOLDER: Deprecated - will be replaced with CoreML-based embeddings
#[deprecated(note = "Ollama provider deprecated - use CoreML embeddings instead")]
```

### Acceptance Criteria
- [ ] CoreML embedding provider is implemented
- [ ] CoreML model loading and inference works
- [ ] Tokenization is properly integrated
- [ ] Vector embeddings are generated correctly
- [ ] Similarity search functionality works
- [ ] Performance meets requirements (latency, throughput)
- [ ] Unit tests with mock CoreML models
- [ ] Integration tests with real embedding generation

### Dependencies
- CoreML framework integration
- Model loading and caching
- Tokenization library
- Performance optimization

---

## TICKET-009: Embedding Model Loading

**Priority:** P2 - Medium
**Tier:** 3 (Data & Memory)
**Assignee:** TBD
**Estimate:** 3-4 days
**Dependencies:** CoreML/ONNX model integration

### Description
Replace placeholder embedding model with real model loading. Currently returns dummy embeddings, blocking embedding generation.

### Location
- File: `iterations/v3/data-infrastructure/src/embedding/model_loading.rs`
- Lines: 133-135

### Current State
```rust
// TODO: Replace placeholder model with real embedding model
// PLACEHOLDER
```

### Acceptance Criteria
- [ ] Real embedding model is loaded (CoreML, ONNX, or SafeTensors)
- [ ] Proper token-to-embedding forward pass implemented
- [ ] Model loading failures handled gracefully
- [ ] Fallback to placeholder only when necessary
- [ ] Unit tests with real models
- [ ] Integration tests with embedding generation
- [ ] Embedding quality meets requirements

### Dependencies
- CoreML/ONNX model format support
- Model storage and versioning
- Memory management for large models
- Fallback mechanisms

---

## TICKET-010: Worker Endpoint Resolution

**Priority:** P2 - Medium
**Tier:** 4 (Integration Points)
**Assignee:** TBD
**Estimate:** 2-3 days
**Dependencies:** Worker registry service

### Description
Implement proper worker endpoint resolution for service discovery. Currently returns placeholder, blocking worker discovery and communication.

### Location
- File: `iterations/v3/agent-workers/src/executor.rs`
- Lines: 1510-1512

### Current State
```rust
// TODO: Implement proper worker endpoint resolution
```

### Acceptance Criteria
- [ ] Worker registry service is integrated
- [ ] Worker endpoint queried by worker ID
- [ ] Worker health and availability checked
- [ ] Resolved endpoints cached for performance
- [ ] Worker failover and load balancing handled
- [ ] Unit tests with mock worker registry
- [ ] Integration tests with real service discovery
- [ ] Worker communication works correctly

### Dependencies
- Worker registry service implementation
- Service discovery mechanism
- Caching layer for endpoints

---

## TICKET-011: Database Config

**Priority:** P2 - Medium
**Tier:** 4 (Integration Points)
**Assignee:** TBD
**Estimate:** 1-2 days
**Dependencies:** Database configuration system

### Description
Create proper database configuration for DatabaseClient instead of using default. Currently uses placeholder config, blocking database persistence.

### Location
- File: `iterations/v3/agent-workers/src/coordinator.rs`
- Lines: 210-212

### Current State
```rust
// TODO: Create proper database config for DatabaseClient::new()
// For now, use a placeholder - this will need to be fixed when database integration is complete
let db_config = data_infrastructure::DatabaseConfig::default();
```

### Acceptance Criteria
- [ ] Proper database config created for DatabaseClient::new()
- [ ] Database integration dependencies resolved
- [ ] Configuration includes connection parameters
- [ ] Error handling for database operations
- [ ] Unit tests with mock database config
- [ ] Integration tests with real database operations
- [ ] Database persistence works correctly

### Dependencies
- Database configuration system
- Connection parameter management
- Database migration system

---

## Implementation Order & Dependencies

### Dependency Graph Analysis

```
TICKET-001 (Worker Pool Integration)
├── Depends on: MCPWorkerPool trait in agent-workers
└── Blocks: TICKET-003 (needs worker discovery for metrics)

TICKET-002 (Worker Execution Logic)
├── Depends on: MCP tool integration, execution context
└── Blocks: Core task execution path

TICKET-003 (Worker Health & Performance)
├── Depends on: Worker status API, TICKET-001 (worker discovery)
├── Parallel with: TICKET-001, TICKET-002
└── Enables: Intelligent load balancing

TICKET-004 (Task Queue Implementation)
├── Depends on: Task queue service
├── Parallel with: TICKET-001, TICKET-002, TICKET-003
└── Enables: Proper task lifecycle management

TICKET-005 (CAWS Runtime Validator)
├── Independent
└── Enables: Quality gate enforcement

TICKET-006 (Verdict Writer)
├── Independent
└── Enables: Provenance tracking and audit trails

TICKET-007 (TaskExecutor Factory)
├── Depends on: agent-workers TaskExecutor trait implementation
├── Depends on: TICKET-002 (worker execution logic)
└── Enables: Task execution initialization

TICKET-008 (CoreML Embedding Provider)
├── Independent
└── Enables: Vector embeddings and similarity search

TICKET-009 (Embedding Model Loading)
├── Independent
├── Parallel with: TICKET-008
└── Enables: Embedding generation

TICKET-010 (Worker Endpoint Resolution)
├── Depends on: Worker registry service
├── Depends on: TICKET-001 (worker discovery)
└── Enables: Service discovery and communication

TICKET-011 (Database Config)
├── Depends on: Database configuration system
└── Enables: Database persistence
```

### Recommended Implementation Order

#### Phase 1: Core Execution (Week 1-2, Parallel Implementation)
**Focus:** Unblock basic task execution end-to-end
1. **TICKET-001** + **TICKET-002** (Parallel) - Core execution foundation
2. **TICKET-003** + **TICKET-004** (Parallel) - Execution optimization

**Milestone:** Tasks can be submitted and executed end-to-end

#### Phase 2: Quality & Validation (Week 3, Sequential)
**Focus:** Enable quality gates and provenance tracking
3. **TICKET-005** + **TICKET-006** (Parallel) - Independent quality systems
4. **TICKET-007** (Sequential) - Depends on agent-workers integration

**Milestone:** Quality gates enforced, audit trails maintained

#### Phase 3: Data & Memory (Week 4, Parallel)
**Focus:** Enable AI/memory features
5. **TICKET-008** + **TICKET-009** (Parallel) - Embedding systems

**Milestone:** Vector embeddings and similarity search work

#### Phase 4: Integration (Week 5, Sequential)
**Focus:** Final system integration
6. **TICKET-010** (Worker registry integration)
7. **TICKET-011** (Database persistence)

**Milestone:** Full system integration complete

### Critical Path Dependencies

**Hard Dependencies (Must Complete First):**
- TICKET-001 → TICKET-003 (worker discovery needed for metrics)
- TICKET-002 → TICKET-007 (execution logic needed for factory)
- TICKET-001 → TICKET-010 (worker discovery needed for endpoint resolution)

**Soft Dependencies (Can Work Around):**
- All other dependencies can be mocked/stubbed initially
- Full implementations can be added incrementally

### Risk Mitigation

**High-Risk Items:**
- TICKET-008/009: CoreML integration may require hardware-specific testing
- TICKET-007: Cross-crate dependencies may cause compilation issues

**Low-Risk Items:**
- TICKET-005/006: Independent infrastructure, can be implemented safely
- TICKET-010/011: Standard integration patterns

---

## Effort Estimation Summary

### Phase 1: Core Execution (8-10 days total)
- **TICKET-001**: 2-3 days (Worker Pool Integration)
- **TICKET-002**: 3-4 days (Worker Execution Logic)
- **TICKET-003**: 2-3 days (Worker Health & Performance)
- **TICKET-004**: 3-4 days (Task Queue Implementation)

### Phase 2: Quality & Validation (7-9 days total)
- **TICKET-005**: 2-3 days (CAWS Runtime Validator)
- **TICKET-006**: 2-3 days (Verdict Writer)
- **TICKET-007**: 3-4 days (TaskExecutor Factory)

### Phase 3: Data & Memory (7-9 days total)
- **TICKET-008**: 4-5 days (CoreML Embedding Provider)
- **TICKET-009**: 3-4 days (Embedding Model Loading)

### Phase 4: Integration (3-5 days total)
- **TICKET-010**: 2-3 days (Worker Endpoint Resolution)
- **TICKET-011**: 1-2 days (Database Config)

### Risk-Adjusted Timeline: 4-6 weeks

**Optimistic (Parallel implementation, no blockers):** 25-32 days
**Realistic (Some sequential dependencies, minor issues):** 28-38 days
**Conservative (Major integration issues, CoreML complexity):** 32-42 days

### Effort Breakdown by Category

- **Core Execution Logic**: 40% (10-14 days) - Most critical, highest complexity
- **Quality & Validation**: 30% (9-11 days) - Independent, moderate complexity
- **Data & AI Systems**: 20% (7-9 days) - Specialized, hardware dependencies
- **Integration Points**: 10% (3-5 days) - Standard integration patterns

### Resource Requirements

**High-Complexity Tasks (Parallelizable):**
- TICKET-001, TICKET-002, TICKET-003, TICKET-004 (Core execution)

**Medium-Complexity Tasks (Independent):**
- TICKET-005, TICKET-006, TICKET-008, TICKET-009

**Low-Complexity Tasks (Integration):**
- TICKET-007, TICKET-010, TICKET-011

### Risk Factors

**High Risk (+20% effort):**
- TICKET-008/009: CoreML hardware-specific testing and optimization

**Medium Risk (+10% effort):**
- TICKET-007: Cross-crate dependency resolution
- TICKET-001/002: MCP integration complexity

**Low Risk (Baseline):**
- TICKET-005/006: Independent infrastructure
- TICKET-010/011: Standard patterns

---

## Success Criteria

- [ ] All tickets implemented and closed
- [ ] System can execute tasks end-to-end without placeholders
- [ ] Core execution path is fully functional
- [ ] Quality gates are enforced
- [ ] Data and memory systems work
- [ ] Integration points are complete
- [ ] No blocking TODOs in critical paths

---

## Notes

- Tickets should be implemented in the order specified above
- Dependencies between tickets must be respected
- Each ticket should have its own branch and PR
- All tickets require both unit and integration tests
- Performance requirements must be met for production readiness
- Effort estimates include testing and documentation
- Parallel implementation possible in Phase 1 and Phase 3
