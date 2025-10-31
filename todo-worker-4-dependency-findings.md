# Chunk 4 Dependency Findings & Integration Plan

## Summary

After thorough search of the codebase, found several dependencies that were thought to be missing:

## Found Dependencies

### 1. System Metrics (Disk/Network I/O) ✅
**Location**: `iterations/v3/system-observability/src/health_metrics.rs`
- **Type**: `MetricsCollector` struct
- **Method**: `collect_system_metrics() -> Result<SystemMetrics>`
- **Fields Available**:
  - `network_io: u64` (bytes/sec)
  - `disk_io: u64` (bytes/sec)
- **Status**: Type exists, but implementations are placeholders. However, the types are available and can be integrated.
- **Integration**: Can be used in `convert_to_learning_signals` to populate `disk_io_mb` and `network_io_mb`

### 2. Sandbox Execution ✅
**Location**: `iterations/v3/system-quality-security/src/sandbox.rs`
- **Type**: `ProductionSandbox` struct implementing `Sandbox` trait
- **Methods**: 
  - `execute(request: ExecutionRequest) -> SandboxResult<ExecutionResult>`
  - Supports Docker, Firejail, Nspawn, Bubblewrap modes
- **Status**: Full implementation available
- **Integration**: Can be used in `agent-research/src/self_prompting_agent/sandbox.rs` for real sandbox execution

### 3. Council System ✅
**Location**: `iterations/v3/agent-orchestration/src/council.rs`
- **Type**: `Council` struct
- **Features**: Memory integration, judge selection, verdict aggregation
- **Status**: Full implementation available
- **Integration**: Can send learning signals via memory system integration

### 4. ExecutionMetrics Type ❌
**Location**: Not found - needs definition
- **Usage**: Used in `analyze_failures` and `record_execution_metrics`
- **Fields needed**:
  - `execution_time_ms: Option<u64>`
  - `cpu_usage_percent: Option<f64>`
  - `memory_usage_mb: Option<f64>`
- **Action**: Define this type

### 5. TaskPattern Task ID ⚠️
**Location**: `iterations/v3/agent-workers/src/learning/types.rs`
- **Issue**: `TaskPattern` has `id: Uuid` (pattern ID), not task ID
- **Solution**: `update_worker_selection` needs task_id parameter from caller
- **Status**: Requires method signature change or parameter passing

## Integration Plan

### Priority 1: Define ExecutionMetrics
- Add `ExecutionMetrics` struct to `coordinator_old.rs` or appropriate module
- Fields: `execution_time_ms`, `cpu_usage_percent`, `memory_usage_mb`

### Priority 2: Integrate System Metrics
- Add `system-observability` dependency to `agent-workers/Cargo.toml`
- Use `MetricsCollector::collect_system_metrics()` in `convert_to_learning_signals`
- Convert `network_io` (u64 bytes/sec) to MB
- Convert `disk_io` (u64 bytes/sec) to MB

### Priority 3: Fix TaskPattern Task ID
- Update `update_worker_selection` signature to accept `task_id: &TaskId`
- Or extract task_id from execution records if available

### Priority 4: Integrate Sandbox
- Add `system-quality-security` dependency to `agent-research/Cargo.toml`
- Use `ProductionSandbox` in `execute_in_sandbox` method

### Priority 5: Council Integration
- Research council memory system integration
- Connect learning signals to council memory system

## Next Steps

1. Define `ExecutionMetrics` struct
2. Integrate system-observability for disk/network I/O
3. Fix TaskPattern task_id extraction
4. Integrate sandbox execution
5. Research council integration options
