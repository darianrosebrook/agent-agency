# Benchmark Integration Guide

## Overview

This document outlines the integration strategy for the benchmark suites with the existing Agent Agency V3 codebase.

## Current Implementation Status

### ✅ Implemented Components

1. **Benchmark Suites**
   - E2E Autonomous Flow Benchmarks (`e2e_autonomous_flow_benchmarks.rs`)
   - Load & Performance Benchmarks (`load_performance_benchmarks.rs`)
   - Module exports (`benchmarks/mod.rs`)

2. **Existing Agent Components**
   - `agent-research::SelfPromptingAgent` - Self-prompting agent implementation
   - `agent-research::SelfPromptingLoop` - Loop controller with iterative refinement
   - `agent-workers::ParallelCoordinator` - Parallel task execution coordinator

### ⚠️ Integration Gaps

1. **Type Mismatches**
   - Benchmark expects `agent_orchestration::types::Task` but implementation uses `agent_research::prompting_types::Task`
   - Need adapter/converter between these types

2. **Configuration Mismatches**
   - `SelfPromptingConfig` has different structure than expected
   - Need to align configuration types

3. **Result Types**
   - Execution results have different structures
   - Need result adapters

4. **Missing Components**
   - `LoadBalancer` from `agent-model-management` not yet implemented
   - `WorkspaceFactory` from `agent-data-processing` not yet implemented
   - `SystemMetrics` from `agent-health-monitoring` not yet implemented

## Integration Strategy

### Phase 1: Type Adapters (High Priority)

Create adapter modules to bridge type mismatches:

```rust
// In agent-research or shared types crate
pub mod adapters {
    use agent_research::prompting_types::Task as ResearchTask;
    use agent_orchestration::types::Task as OrchestrationTask;
    
    pub fn convert_task(from: OrchestrationTask) -> ResearchTask {
        ResearchTask {
            id: uuid::Uuid::parse_str(&from.id).unwrap_or_default(),
            description: from.description,
            task_type: infer_task_type(&from.description),
            target_files: from.target_files.unwrap_or_default(),
            constraints: from.constraints.unwrap_or_default(),
            refinement_context: vec![],
        }
    }
    
    fn infer_task_type(description: &str) -> TaskType {
        // Simple inference logic
        if description.to_lowercase().contains("test") {
            TaskType::Testing
        } else if description.to_lowercase().contains("refactor") {
            TaskType::CodeRefactor
        } else {
            TaskType::CodeGeneration
        }
    }
}
```

### Phase 2: Stub Implementations (Medium Priority)

Create stub implementations for missing components:

```rust
// In agent-model-management
pub struct LoadBalancer;

impl LoadBalancer {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn execute_task(
        &self,
        task: Task,
        model: &str
    ) -> Result<ExecutionHandle> {
        // Stub implementation for benchmarking
        Ok(ExecutionHandle {
            task_id: task.id,
            model: model.to_string(),
            // ... stub fields
        })
    }
}

// In agent-data-processing
pub struct WorkspaceFactory;

impl WorkspaceFactory {
    pub fn create_sandbox() -> SandboxEnvironment {
        SandboxEnvironment::new(None).expect("Failed to create sandbox")
    }
}
```

### Phase 3: Updated Benchmarks (High Priority)

Update benchmark code to work with actual implementations:

```rust
// In benchmarks/e2e_autonomous_flow_benchmarks.rs
use agent_research::adapters::convert_task;
use agent_research::prompting_types::{Task as ResearchTask, SelfPromptingAgent};

async fn benchmark_self_prompting_loop(&self) -> Result<E2EAutonomousFlowMetrics> {
    let config = SelfPromptingAgentConfig::default();
    let model_registry = Arc::new(ModelRegistry::new());
    let evaluator = Arc::new(EvaluationOrchestrator::new());
    
    let agent = SelfPromptingAgent::new(
        config,
        model_registry,
        evaluator
    ).await?;
    
    // Convert orchestration task to research task
    let orchestration_task = Task {
        id: "benchmark-1".to_string(),
        description: "...".to_string(),
        ..Default::default()
    };
    
    let research_task = convert_task(orchestration_task);
    
    // Execute
    let result = agent.execute_task(research_task).await?;
    
    // Convert results to metrics
    Ok(E2EAutonomousFlowMetrics {
        test_name: "self_prompting_loop".to_string(),
        iterations: result.iterations,
        quality_score: result.result.final_report.score,
        quality_improvement: 0.0, // Calculate from history
        completion_time: Duration::from_millis(result.result.execution_time_ms),
        token_usage: 0, // Extract from metadata
        success: result.result.final_report.status == EvalStatus::Pass,
        metadata: HashMap::new(),
    })
}
```

### Phase 4: Implement Missing Components (Low Priority)

1. **LoadBalancer** (`agent-model-management/deployment.rs`)
   - Model selection logic
   - Load balancing algorithm
   - Hot-swapping functionality

2. **WorkspaceFactory** (`agent-data-processing/operations.rs`)
   - Sandbox creation
   - File operations
   - Allowlist enforcement

3. **SystemMetrics** (`agent-health-monitoring/metrics.rs`)
   - CPU/memory monitoring
   - Performance tracking
   - Telemetry collection

## Testing Strategy

### Unit Tests

Test each adapter independently:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_task_conversion() {
        let orig = Task {
            id: "test-1".to_string(),
            description: "Test task".to_string(),
            ..Default::default()
        };
        
        let converted = convert_task(orig);
        assert_eq!(converted.description, "Test task");
    }
}
```

### Integration Tests

Test full benchmark flow:

```rust
#[tokio::test]
async fn test_self_prompting_benchmark() {
    let bench = E2EAutonomousFlowBenchmarks::new();
    let result = bench.benchmark_self_prompting_loop().await;
    
    assert!(result.is_ok());
    let metrics = result.unwrap();
    assert!(metrics.success);
    assert!(metrics.iterations <= 5);
}
```

## Next Steps

1. **Immediate** (Today)
   - Create type adapters in `agent-research`
   - Update benchmark imports
   - Add stub implementations

2. **Short-term** (This week)
   - Implement missing components (LoadBalancer, WorkspaceFactory)
   - Run baseline benchmarks
   - Document performance targets

3. **Medium-term** (This month)
   - Optimize based on benchmark results
   - Add more comprehensive benchmarks
   - Set up CI integration

## Running Benchmarks

Once integration is complete:

```bash
# Run all benchmarks
cargo test --package testing-validation --lib benchmarks:: -- --nocapture

# Run specific benchmark
cargo test --package testing-validation --lib e2e_autonomous_flow_benchmarks::test_benchmark_self_prompting_loop

# Generate report
cargo test --package testing-validation --lib benchmarks:: -- --nocapture > benchmark_report.txt
```

## Success Criteria

- [ ] All benchmarks compile without errors
- [ ] Type adapters handle conversions correctly
- [ ] Stub implementations provide realistic behavior
- [ ] Benchmarks can run against actual implementations
- [ ] Performance metrics meet documented targets
- [ ] CI integration passes
