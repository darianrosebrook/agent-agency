# V3 Circular Dependencies Documentation

This document catalogs all known circular dependencies in the V3 codebase, their current workarounds, and recommended resolution strategies.

## Summary

| Cycle | Crates Involved | Current Workaround | Priority |
|-------|-----------------|-------------------|----------|
| 1 | `agent-orchestration` <-> `agent-workers` | Local type definitions | High |
| 2 | `agent-orchestration` <-> `data-infrastructure` | Inline adapter module | High |
| 3 | `agent-mcp` <-> `data-infrastructure` | Runtime injection pattern | Medium |
| 4 | `agent-orchestration` <-> `agent-constitutional-council` | Local enum definitions | Low |
| 5 | `agent-orchestration` <-> `agent-research` | Local type definitions | Low |
| 6 | `agent-orchestration` <-> `agent-data-processing` | Local type definitions | Low |

---

## Cycle 1: agent-orchestration <-> agent-workers

### Description
`agent-orchestration` needs to execute tasks through workers, but `agent-workers` depends on orchestration types for task definitions.

### Locations

**agent-orchestration/src/adapter.rs:813-829**
```rust
/// This method requires the agent-workers crate which is not available due to
/// circular dependency constraints. Use the UnifiedOrchestrator for task execution instead.
///
/// Returns: Error indicating the dependency constraint
pub async fn execute_task_with_workers(
    &self,
    _task: &TaskDescriptor,
) -> anyhow::Result<()> {
    // This conversion requires agent-workers::ComplexTask which is not available
    // due to circular dependency constraints.
}
```

**agent-orchestration/src/multimodal_orchestration.rs:197-198**
```rust
// Local type definitions to avoid circular dependency with agent-workers
// These mirror types from agent-workers crate
```

### Current Workaround
- Local type definitions that mirror `agent-workers` types
- Task execution delegated to `UnifiedOrchestrator` which uses dependency injection

### Recommended Resolution
1. Extract shared task types to `agent-agency-contracts`
2. Define `TaskExecutorPort` trait in contracts crate
3. Use feature-gated optional dependency for full integration

---

## Cycle 2: agent-orchestration <-> data-infrastructure

### Description
`agent-orchestration` needs database operations, but `data-infrastructure` depends on orchestration types for service coordination.

### Locations

**agent-orchestration/src/orchestration/unified_orchestrator_factory.rs:790-793**
```rust
/// Database operations adapter - bridges data-infrastructure DatabaseClient to agent-orchestration DatabaseOperations
///
/// This adapter is implemented inline to avoid circular dependency with data-interfaces-adapters.
/// It provides full database operations by wrapping DatabaseClient and mapping between type systems.
mod database_operations_adapter {
```

**agent-orchestration/src/planning/data_infrastructure_types.rs:1**
```rust
//! Local type definitions for data infrastructure to avoid circular dependencies
```

**data-infrastructure/src/orchestrator_service.rs:130-131**
```rust
/// Trait for task execution (allows dependency injection without circular dependencies)
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
```

### Current Workaround
- `DatabaseOperations` trait defined in `agent-orchestration` with local types
- `TaskExecutor` trait defined in `data-infrastructure` for dependency injection
- Inline adapter module in `unified_orchestrator_factory.rs`

### Recommended Resolution
1. Move `DatabaseOperations` trait to `agent-agency-contracts`
2. Move `TaskExecutor` trait to `agent-agency-contracts`
3. Both crates depend only on contracts for shared interfaces

---

## Cycle 3: agent-mcp <-> data-infrastructure

### Description
`agent-mcp` needs file operations from `data-infrastructure`, but `data-infrastructure` may need MCP tool definitions.

### Locations

**agent-mcp/src/tool_registry.rs:26-56**
```rust
// File operations service - using runtime injection pattern to avoid circular dependencies
// Real implementations should be injected via ToolRegistry::with_file_ops()

/// This module provides documentation and examples for creating real FileOperationsService
/// implementations when data-infrastructure is available. Since there's a circular
/// dependency between agent-mcp and data-infrastructure, this must be called
/// from code that has access to both crates.
```

**agent-mcp/src/server.rs:185-186**
```rust
// Database client for rate limiting persistence
// Implemented locally to avoid circular dependency with agent-data-processing
```

### Current Workaround
- Runtime injection pattern via `ToolRegistry::with_file_ops()`
- `PlaceholderFileOperationsService` as default
- Local `DatabaseClient` implementation in `agent-mcp`

### Recommended Resolution
1. Define `FileOperationsPort` trait in `system-common-interfaces`
2. Inject real implementation at application startup
3. This is an acceptable pattern for optional integrations

---

## Cycle 4: agent-orchestration <-> agent-constitutional-council

### Description
`agent-orchestration` needs council review capabilities, but `agent-constitutional-council` depends on orchestration types.

### Locations

**agent-orchestration/src/planning/council_adapter.rs:30-33**
```rust
/// Review priority levels matching agent-constitutional-council::ReviewPriority
/// Defined locally to avoid circular dependency (agent-constitutional-council depends on agent-orchestration)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ReviewPriority {
```

### Current Workaround
- Local `ReviewPriority` enum definition
- Council integration through adapter pattern

### Recommended Resolution
1. Move `ReviewPriority` to `agent-agency-contracts`
2. Both crates import from contracts
3. Low priority - current workaround is acceptable

---

## Cycle 5: agent-orchestration <-> agent-research

### Description
`agent-orchestration` uses research evidence types, but `agent-research` depends on orchestration for planning.

### Locations

**agent-orchestration/src/planning/evidence.rs:16-17**
```rust
// Local type definitions to avoid circular dependency with agent-research
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ResearchEvidence {
```

**agent-orchestration/src/autonomous_integration.rs:24-25**
```rust
// Services will be injected to avoid circular dependencies
// use data_infrastructure::create_file_operations_service;
// use agent_research::create_learning_service;
```

### Current Workaround
- Local `ResearchEvidence` type definition
- Service injection pattern for learning services

### Recommended Resolution
1. Move `ResearchEvidence` to `agent-agency-contracts`
2. Use dependency injection for learning services
3. Low priority - current workaround is acceptable

---

## Cycle 6: agent-orchestration <-> agent-data-processing

### Description
`agent-orchestration` needs data processing capabilities for multimodal orchestration.

### Locations

**agent-orchestration/src/multimodal_orchestration.rs:61-62**
```rust
// Local type definitions to avoid circular dependency with agent-data-processing
// These mirror types from agent-data-processing crate
```

### Current Workaround
- Local type definitions mirroring `agent-data-processing` types
- Feature-gated integration when available

### Recommended Resolution
1. Move shared types to `agent-agency-contracts`
2. Use feature flags for optional data processing integration
3. Low priority - current workaround is acceptable

---

## Resolution Strategy

### Short-term (Milestone 2)

1. **Document all cycles** (this document)
2. **Verify workarounds are functional** - ensure all local types match their source definitions
3. **Add compile-time assertions** where possible to catch type drift

### Medium-term (Future milestone)

1. **Extract shared traits to agent-agency-contracts**:
   - `TaskExecutorPort`
   - `DatabaseOperationsPort`
   - `FileOperationsPort`
   - `ReviewPriority`
   - `ResearchEvidence`

2. **Implement port/adapter pattern**:
   - Define ports (interfaces) in contracts
   - Implement adapters in consuming crates
   - Inject implementations at runtime

### Long-term (Architecture improvement)

1. **Refactor crate boundaries**:
   - Consider merging tightly coupled crates
   - Or further splitting to create clear dependency hierarchy

2. **Use workspace-level type definitions**:
   - Single source of truth for all shared types
   - Automatic synchronization via re-exports

---

## Verification Commands

```bash
# Check for circular dependency patterns
rg -n "circular|workaround|avoid.*dependency" iterations/v3 --glob "*.rs"

# Verify local type definitions match source
# (Manual review required)

# Check that all tests compile
SQLX_OFFLINE=true cargo test --workspace --no-run
```

---

## Related Files

- `agent-agency-contracts/src/ports.rs` - Existing port definitions
- `data-infrastructure/src/orchestrator_service.rs` - TaskExecutor trait
- `agent-orchestration/src/planning/data_infrastructure_types.rs` - Local type definitions
