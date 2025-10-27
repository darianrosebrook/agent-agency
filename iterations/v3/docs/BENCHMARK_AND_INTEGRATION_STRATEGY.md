# Benchmark and Integration Test Strategy - V3

**Author**: @darianrosebrook  
**Date**: January 2025  
**Status**: Strategic Planning

---

## Executive Summary

This document outlines comprehensive benchmarking and integration testing strategies for Agent Agency V3, focusing on end-to-end autonomous flows, model integration, and system performance under load.

## Current Architecture Alignment Assessment

### Documented vs Implemented Components

Based on review of `agents.md`, `theory.md`, and `end-to-end-autonomous-flow-architecture.md`:

#### ✅ Fully Aligned Components

1. **agent-orchestration** - Council-based decision making operational
2. **agent-memory** - Multi-tenant memory with knowledge graphs operational
3. **agent-research** - Advanced AI research capabilities operational
4. **agent-mcp** - Model Context Protocol implementation operational
5. **agent-data-processing** - Data ingestion and enrichment operational
6. **system-observability** - Monitoring and telemetry operational
7. **testing-validation** - Comprehensive test framework operational

#### ⚠️ Partially Aligned Components

1. **agent-workers** - MCP-based execution exists but needs E2E testing
2. **agent-model-management** - Lifecycle management exists but integration gaps
3. **system-acceleration** - Hardware acceleration framework exists but untested

#### ❌ Gaps Identified

1. **Self-Prompting Loop** - Stubbed in benchmarks, needs real implementation
2. **Claim Extraction Pipeline** - Documented but not integrated E2E
3. **CAWS Runtime Integration** - Validation exists but workflow integration incomplete

---

## Benchmark Strategy

### 1. End-to-End Autonomous Flow Benchmarks

**Objective**: Verify complete self-governing agent workflows from theory.md

**Scenarios**:

#### A. Autonomous Task Execution Loop

```rust
// Benchmark: Self-prompting loop with evaluation
Test: agent-research::self_prompting_agent::SelfPromptingLoop
Metrics:
  - Iterations to convergence
  - Quality improvement per iteration
  - Token usage efficiency
  - Time to completion
Success Criteria:
  - Max 5 iterations before convergence
  - Quality improvement ≥ 10% per iteration
  - P95 completion time < 30s
```

#### B. Model Hot-Swapping Performance

```rust
// Benchmark: Hot-swap during active task
Test: agent-model-management::deployment::LoadBalancer
Metrics:
  - Swap latency
  - State preservation accuracy
  - Zero-downtime guarantee
  - Task continuity
Success Criteria:
  - Swap latency < 1s
  - 100% state preservation
  - Zero task failures
  - Seamless user experience
```

#### C. Multi-Agent Coordination

```rust
// Benchmark: Parallel task execution
Test: agent-workers::coordinator::ParallelCoordinator
Metrics:
  - Concurrent task throughput
  - Resource utilization
  - Coordination overhead
  - Load balancing efficiency
Success Criteria:
  - 50+ concurrent tasks
  - CPU utilization 60-80%
  - Overhead < 10%
  - Fair load distribution
```

### 2. Model Integration Benchmarks

**Objective**: Validate model lifecycle and inference performance

#### A. Model Loading and Warm-up

```rust
// Benchmark config for model loading
const MODEL_LOADING_BENCHMARK = BenchmarkConfig {
    models: vec![
        "gemma3:1b",
        "gemma3:4b", 
        "gemma3n:e2b",
        "gemma3n:e4b",
    ],
    iterations: 100,
    warmup_iterations: 10,
    metrics: vec![
        "load_time_ms",
        "warmup_time_ms",
        "memory_usage_mb",
        "inference_latency_ms",
    ],
};

// Success criteria from documented performance
let EXPECTED_PERFORMANCE = PerformanceTargets {
    gemma3_1b: InferenceProfile { tokens_per_sec: 130, latency_ms: 153 },
    gemma3_4b: InferenceProfile { tokens_per_sec: 260, latency_ms: 76 },
    gemma3n_e2b: InferenceProfile { tokens_per_sec: 66, latency_ms: 302 },
    gemma3n_e4b: InferenceProfile { tokens_per_sec: 47, latency_ms: 426 },
};
```

#### B. Inference Throughput Under Load

```rust
// Test concurrent inference performance
async fn benchmark_concurrent_inference() -> BenchmarkResult {
    let concurrent_requests = vec![1, 5, 10, 20, 50];
    
    for concurrency in concurrent_requests {
        let results = spawn_concurrent_requests(
            model: "gemma3n:e2b",
            concurrency,
            requests_per_client: 100,
        ).await;
        
        measure_throughput(results).await;
        measure_latency_distribution(results).await;
        measure_resource_usage(results).await;
    }
}

// Success criteria
assert!(p95_latency_ms < 500);  // From v2 benchmarks
assert!(throughput_tokens_per_sec >= 66);
assert!(cpu_utilization < 90%);
assert!(memory_growth < 10%);
```

### 3. System Integration Benchmarks

**Objective**: Validate cross-component integration and performance

#### A. Complete Workflow Benchmark

```rust
// End-to-end workflow: Task → Worker → Evaluation → Completion
async fn benchmark_complete_workflow() {
    let tasks = vec![
        WorkflowTask::text_transformation(),
        WorkflowTask::code_generation(),
        WorkflowTask::research_query(),
    ];
    
    for task in tasks {
        let metrics = execute_and_measure_workflow(task).await;
        
        // From performance-tracking.md:
        // Text Transformation: 2.1s avg, P95: 3.2s
        // Code Generation: 25s avg, P95: 35s
        // Research: TBD
        
        validate_sla_compliance(metrics, task.sla_targets).await;
    }
}

// Success criteria from documented performance
let SLA_TARGETS = HashMap::from([
    ("text_transformation", LatencyTarget { p95_ms: 3200, success_rate: 1.0 }),
    ("code_generation", LatencyTarget { p95_ms: 35000, success_rate: 0.8 }),
    ("research", LatencyTarget { p95_ms: 60000, success_rate: 0.7 }),
]);
```

#### B. Database and Persistence Performance

```rust
// Benchmark PostgreSQL + pgvector + Redis integration
async fn benchmark_data_persistence() {
    // From documented architecture
    test_scenarios(vec![
        "concurrent_agent_registry_queries",
        "vector_similarity_search",
        "multi_tenant_context_storage",
        "audit_trail_writes",
        "provenance_chain_creation",
    ]);
    
    // Success criteria
    assert!(registry_queries_p95_ms < 10);
    assert!(vector_search_p95_ms < 100);
    assert!(write_throughput > 1000_writes_per_sec);
    assert!(zero_data_loss);
}
```

---

## Integration Test Strategy

### 1. End-to-End Autonomous Flow Tests

**Based on end-to-end-autonomous-flow-architecture.md requirements**

#### Test Suite: Self-Governing Agent Workflow

```rust
#[test_suite]
mod e2e_autonomous_flow {
    
    #[test]
    async fn test_self_prompting_loop() {
        // Scenario: Agent iteratively improves output until quality threshold
        
        // Setup
        let agent = SelfPromptingAgent::new(config);
        let task = Task::text_rewrite("Informal email content");
        
        // Execute
        let result = agent.execute_task(task).await;
        
        // Verify
        assert_eq!(result.iterations, 2); // From POC: avg 2/3 iterations
        assert!(result.quality_score >= 0.85);
        assert!(result.completion_time.as_secs() < 5);
        assert!(result.token_usage < 5000);
    }
    
    #[test]
    async fn test_autonomous_file_editing() {
        // Scenario: Agent edits files in sandbox with proper safeguards
        
        // Setup
        let sandbox = WorkspaceFactory::create_sandbox();
        let allowlist = AllowList::new(vec!["src/".to_string()]);
        let task = Task::bug_fix("example.ts", "Fix null pointer");
        
        // Execute
        let result = agent.execute_with_safeguards(task, allowlist).await;
        
        // Verify
        assert!(result.diffs.len() > 0);
        assert!(result.diffs.all_in_allowlist());
        assert!(result.tests_pass);
        assert!(result.caws_compliance);
    }
    
    #[test]
    async fn test_model_hot_swap_mid_task() {
        // Scenario: Swap model during active task without loss of context
        
        // Setup
        let initial_model = "gemma3n:e2b";
        let replacement_model = "gemma3n:e4b";
        let task = Task::complex_reasoning();
        
        // Execute
        let mut agent = agent_with_model(initial_model);
        agent.start_task(task.clone()).await;
        
        // Swap mid-execution
        agent.hot_swap_model(replacement_model).await;
        
        let result = agent.complete_task().await;
        
        // Verify
        assert!(result.success);
        assert!(result.context_preserved);
        assert!(result.quality_maintained);
        assert!(result.swap_overhead_ms < 1000);
    }
}
```

### 2. Crate Integration Tests

**Verify crate boundaries and contracts**

#### Test Suite: Cross-Crate Integration

```rust
#[test_suite]
mod crate_integration {
    
    use agent_orchestration::Orchestrator;
    use agent_workers::WorkerPool;
    use agent_model_management::ModelManager;
    use agent_memory::MemorySystem;
    use agent_research::ResearchEngine;
    use agent_mcp::MCPServer;
    
    #[test]
    async fn test_orchestration_to_worker_flow() {
        // Orchestration → Worker → Model → Response
        
        let orchestrator = Orchestrator::new();
        let worker_pool = WorkerPool::new();
        let model_mgr = ModelManager::new();
        
        let task = create_complex_task();
        
        // Execute workflow
        let assignment = orchestrator.route_task(&task).await;
        let worker = worker_pool.get_worker(assignment.worker_id);
        let model = model_mgr.get_model(assignment.model_id);
        
        let result = worker.execute(task, model).await;
        
        assert!(result.success);
        assert!(result.provenance_recorded);
    }
    
    #[test]
    async fn test_memory_integration_across_components() {
        // Memory system accessible from all agent crates
        
        let memory = MemorySystem::new();
        let orchestrator = Orchestrator::with_memory(&memory);
        let research = ResearchEngine::with_memory(&memory);
        
        // Store context in orchestration
        orchestrator.store_context("session-1", context_data).await;
        
        // Retrieve in research
        let retrieved = research.get_context("session-1").await;
        
        assert_eq!(retrieved, context_data);
        assert!(memory.multi_tenant_isolated());
    }
    
    #[test]
    async fn test_mcp_tool_discovery() {
        // MCP server exposes tools discoverable by workers
        
        let mcp_server = MCPServer::new();
        let worker_pool = WorkerPool::new();
        
        // Register tools
        mcp_server.register_tool(file_edit_tool).await;
        mcp_server.register_tool(code_analyze_tool).await;
        
        // Discover tools
        let discovered = worker_pool.discover_tools().await;
        
        assert!(discovered.contains(&"file_edit".to_string()));
        assert!(discovered.contains(&"code_analyze".to_string()));
        assert!(discovered.len() >= 2);
    }
}
```

### 3. Performance Under Load Tests

**Stress testing for production readiness**

#### Test Suite: Load and Stress Testing

```rust
#[test_suite]
mod load_tests {
    
    #[test]
    async fn test_concurrent_task_throughput() {
        // 100 concurrent tasks, measure throughput
        
        let concurrent_tasks = 100;
        let tasks = generate_tasks(concurrent_tasks);
        
        let start = Instant::now();
        let results = join_all(tasks).await;
        let duration = start.elapsed();
        
        // Metrics
        let success_rate = results.iter().filter(|r| r.success).count() as f64 / concurrent_tasks as f64;
        let avg_latency = duration.as_secs_f64() / concurrent_tasks as f64;
        let throughput = concurrent_tasks as f64 / duration.as_secs_f64();
        
        // Success criteria
        assert!(success_rate >= 0.95);
        assert!(avg_latency < 2.0); // 2 seconds average
        assert!(throughput > 50.0); // 50 tasks/sec
    }
    
    #[test]
    async fn test_sustained_load_endurance() {
        // 24-hour equivalent load in compressed time
        
        let load_duration = Duration::from_secs(60 * 60); // 1 hour compressed
        let tasks_per_second = 10.0;
        let total_tasks = (load_duration.as_secs() as f64 * tasks_per_second) as usize;
        
        let mut success_count = 0;
        let start = Instant::now();
        
        while start.elapsed() < load_duration {
            let batch = generate_task_batch((tasks_per_second as usize).min(100));
            let results = join_all(batch).await;
            success_count += results.iter().filter(|r| r.success).count();
            
            tokio::time::sleep(Duration::from_millis(100)).await;
        }
        
        // Success criteria
        let success_rate = success_count as f64 / total_tasks as f64;
        assert!(success_rate >= 0.90);
        
        // Resource usage
        let memory_growth = measure_memory_growth();
        assert!(memory_growth < 0.10); // Less than 10% growth
    }
    
    #[test]
    async fn test_error_recovery_resilience() {
        // Inject failures, verify recovery
        
        let failures_to_inject = vec![
            FailureType::DatabaseConnectionLoss,
            FailureType::ModelTimeout,
            FailureType::WorkerCrash,
            FailureType::NetworkPartition,
        ];
        
        for failure in failures_to_inject {
            let recovery = test_failure_scenario(failure).await;
            
            assert!(recovery.successful);
            assert!(recovery.time_to_recover < Duration::from_secs(30));
            assert!(recovery.data_integrity_maintained);
        }
    }
}
```

---

## Alignment Verification Matrix

### Theory.md Requirements → Implementation Status

| Requirement | Documented | Implemented | Tested | Gap |
|------------|------------|-------------|--------|-----|
| Local-first execution | ✅ | ✅ | ✅ | - |
| Model-agnostic design | ✅ | ✅ | ⚠️ | Needs hot-swap E2E tests |
| Hot-swapping capability | ✅ | ✅ | ❌ | No integration tests |
| Multi-agent coordination | ✅ | ✅ | ⚠️ | Needs load tests |
| Claim extraction & verification | ✅ | ⚠️ | ❌ | Stubbed, needs implementation |
| Reflexive learning | ✅ | ⚠️ | ❌ | Partial, needs feedback loop |
| Performance benchmarking | ✅ | ✅ | ⚠️ | Needs continuous monitoring |
| Quality gates enforcement | ✅ | ✅ | ✅ | - |
| Provenance tracking | ✅ | ✅ | ⚠️ | Needs E2E audit tests |

### End-to-End Flow Requirements → Implementation Status

| Flow | Documented | Implemented | Tested | Priority |
|------|------------|-------------|--------|----------|
| Autonomous loop execution | ✅ | ⚠️ | ❌ | HIGH |
| Self-prompting refinement | ✅ | ⚠️ | ❌ | HIGH |
| File editing with safeguards | ✅ | ⚠️ | ❌ | HIGH |
| Sandboxed execution | ✅ | ✅ | ⚠️ | MEDIUM |
| Diff-based changes | ✅ | ❌ | ❌ | MEDIUM |
| Git workflow integration | ✅ | ❌ | ❌ | LOW |
| Guardrails & permissions | ✅ | ✅ | ⚠️ | HIGH |

---

## Implementation Priority

### Phase 1: Critical E2E Flows (Week 1-2)

1. **Self-Prompting Loop** ✅ HIGH PRIORITY
   - Implement real SelfPromptingLoop (not stubs)
   - E2E test: task → iterate → evaluate → refine → complete
   - Measure: iterations, quality improvement, token efficiency

2. **Model Hot-Swapping** ✅ HIGH PRIORITY
   - Test mid-task swap scenarios
   - Verify context preservation
   - Measure swap latency and overhead

3. **Multi-Agent Coordination** ✅ MEDIUM PRIORITY
   - Concurrent task execution
   - Resource sharing and load balancing
   - Conflict resolution and consensus

### Phase 2: Integration Coverage (Week 3-4)

4. **Crate Boundary Testing**
   - Verify all 17 crates integrate properly
   - Test cross-crate data flows
   - Validate API contracts

5. **Database & Persistence**
   - PostgreSQL integration tests
   - Vector search performance
   - Multi-tenant isolation

6. **Memory System Integration**
   - Context preservation across crates
   - Knowledge graph queries
   - Federated learning data flow

### Phase 3: Load & Performance (Week 5-6)

7. **Throughput Benchmarks**
   - Concurrent task execution
   - Sustained load endurance
   - Resource utilization profiling

8. **Failure Recovery Tests**
   - Database failures
   - Model timeouts
   - Network partitions
   - Worker crashes

9. **Performance Regression Testing**
   - Compare against v2 benchmarks
   - Identify performance improvements
   - Track regressions

---

## Success Criteria

### Benchmark Targets

| Metric | Target | Source |
|--------|--------|--------|
| Inference latency (P95) | < 500ms | v2 benchmarks |
| Throughput (tokens/sec) | > 66 (gemma3n:e2b) | documented |
| Concurrent tasks | 50+ | theoretical max |
| Hot-swap latency | < 1s | documented |
| Memory growth | < 10% sustained load | v2 benchmarks |
| Success rate | > 95% | production target |
| CAWS compliance | 100% | constitutional requirement |

### Integration Test Coverage

| Component | Unit Tests | Integration Tests | E2E Tests | Status |
|-----------|------------|-------------------|-----------|--------|
| agent-orchestration | ✅ | ⚠️ | ❌ | Needs E2E |
| agent-workers | ✅ | ⚠️ | ❌ | Needs E2E |
| agent-model-management | ✅ | ❌ | ❌ | Needs both |
| agent-memory | ✅ | ⚠️ | ❌ | Needs E2E |
| agent-research | ✅ | ❌ | ❌ | Needs both |
| agent-mcp | ✅ | ⚠️ | ⚠️ | Needs E2E |
| All crates | 80%+ | 60%+ | 40%+ | Target |

---

## Next Steps

1. **Create benchmark test files** for priority scenarios
2. **Implement missing SelfPromptingLoop** functionality
3. **Add E2E integration tests** for critical workflows
4. **Set up continuous benchmarking** in CI/CD
5. **Document performance baselines** for comparison
6. **Implement performance regression detection**

This strategy ensures we validate the end-to-end autonomous flows while maintaining the high performance and quality standards documented in the architecture.
