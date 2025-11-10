//! Parallel Worker Integration
//!
//! Connects Tool Chain Executor with ParallelCoordinator for distributed
//! tool execution across multiple workers with load balancing and fault tolerance.

use schemars::JsonSchema;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::sync::RwLock;
use tracing::{info, error};

use crate::tool_chain_planner::{ToolChain, ToolNode};
use petgraph::visit::EdgeRef;
use crate::executor::{ChainExecutor, ExecutionResult};
use crate::tool_execution::{ToolExecutor, ToolResult};
use crate::tool_registry::ToolRegistry;

use agent_workers::{
    ParallelCoordinator,
    ParallelCoordinatorConfig,
    DecompositionEngine,
    TaskAnalysis,
    Dependency,
    WorkerManager,
    CommunicationHub,
    communication::ChannelConfig,
};

/// Parallel tool execution coordinator
pub struct ParallelToolCoordinator {
    #[allow(dead_code)]
    chain_executor: Arc<ChainExecutor>,
    #[allow(dead_code)]
    parallel_coordinator: Arc<ParallelCoordinator>,
    #[allow(dead_code)]
    worker_manager: Arc<WorkerManager>,
    #[allow(dead_code)]
    communication_hub: Arc<CommunicationHub>,
    #[allow(dead_code)]
    execution_cache: Arc<RwLock<HashMap<String, ExecutionResult>>>,
    #[allow(dead_code)]
    concurrency_limit: usize,
}

impl ParallelToolCoordinator {
    pub fn new(
        tool_executor: Arc<ToolExecutor>,
        _tool_registry: Arc<ToolRegistry>,
        config: ParallelCoordinatorConfig,
    ) -> Self {
        let chain_executor = Arc::new(ChainExecutor::new(
            tool_executor.clone(),
            Arc::new(crate::schema_registry::JsonSchemaRegistry::new()),
            8, // concurrency limit
            30000, // default timeout
        ));

        let parallel_coordinator = Arc::new(ParallelCoordinator::new(config));
        let worker_manager = Arc::new(WorkerManager::new());
        let communication_hub = Arc::new(CommunicationHub::new(ChannelConfig::default()));

        Self {
            chain_executor,
            parallel_coordinator,
            worker_manager,
            communication_hub,
            execution_cache: Arc::new(RwLock::new(HashMap::new())),
            concurrency_limit: 8,
        }
    }

    /// Execute tool chain with parallel workers
    pub async fn execute_parallel(
        &self,
        chain: &ToolChain,
        _cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<ExecutionResult, ParallelExecutionError> {
        // TODO: Implement actual parallel execution with real workers
        //       Currently uses placeholder implementation; should execute tool chain with actual parallel workers and proper task distribution.
        info!("Placeholder: Executing tool chain with simulated parallel workers");

        // Create mock execution results
        let mut results = HashMap::new();
        let mut total_time = 0u64;

        for node_idx in chain.dag.node_indices() {
            let node = &chain.dag[node_idx];
            let _task_id = format!("task_{}", node_idx.index());

            let tool_result = ToolResult {
                tool_name: node.tool_id.clone(),
                result: serde_json::json!({"status": "completed", "node": node_idx.index()}),
                metadata: crate::tool_execution::ExecutionMetadata {
                    execution_time_ms: 100,
                    memory_used_mb: 10.0,
                    success: true,
                    error_message: None,
                    resource_usage: crate::tool_execution::ResourceUsage {
                        cpu_time_ms: 50,
                        peak_memory_mb: 10.0,
                        io_operations: 0,
                        network_bytes: 0,
                    },
                },
                timestamp: chrono::Utc::now(),
            };

            results.insert(node_idx, tool_result.result.clone());
            total_time += 100;
        }

        let execution_result = ExecutionResult {
            chain_hash: chain.plan_hash,
            success: true,
            results,
            execution_time_ms: total_time,
            errors: vec![],
            cancelled_steps: vec![],
        };

        info!("Stub parallel execution completed successfully");
        Ok(execution_result)
    }

    /// Analyze chain for parallel execution opportunities
    #[allow(dead_code)]
    async fn analyze_chain_for_parallelism(
        &self,
        chain: &ToolChain,
    ) -> Result<TaskAnalysis, ParallelExecutionError> {
        let _decomposition_engine = DecompositionEngine::new();

        // Convert chain to task analysis format
        let mut dependencies = Vec::new();

        // Build dependency graph
        for edge_idx in chain.dag.edge_indices() {
            let (source, target) = chain.dag.edge_endpoints(edge_idx).unwrap();
            let _edge = chain.dag.edge_weight(edge_idx).unwrap();

            dependencies.push(Dependency {
                from: agent_workers::SubTaskId(uuid::Uuid::parse_str(&self.node_id_to_task_id(source)).unwrap_or_else(|_| uuid::Uuid::new_v4())),
                to: agent_workers::SubTaskId(uuid::Uuid::parse_str(&self.node_id_to_task_id(target)).unwrap_or_else(|_| uuid::Uuid::new_v4())),
                dependency_type: agent_workers::DependencyType::Data,
            });
        }

        // TODO: Convert ToolChain to ComplexTask and use DecompositionEngine for proper analysis
        //       Currently creates minimal TaskAnalysis; should convert ToolChain to ComplexTask and use DecompositionEngine.
        //
        // COMPLETION CHECKLIST:
        // [ ] Convert ToolChain to ComplexTask structure
        // [ ] Use DecompositionEngine to analyze task properly
        // [ ] Extract complexity scores from decomposition
        // [ ] Extract estimated durations from decomposition
        // [ ] Identify task patterns from decomposition
        // [ ] Add unit tests for ToolChain conversion
        // [ ] Add integration tests with DecompositionEngine
        // [ ] Verify task analysis accuracy
        //
        // ACCEPTANCE CRITERIA:
        // - ToolChain is converted to ComplexTask correctly
        // - DecompositionEngine analyzes task properly
        // - Complexity scores and durations are extracted accurately
        // - Task patterns are identified correctly
        //
        // DEPENDENCIES:
        // - ComplexTask structure (Required)
        // - DecompositionEngine (Required)
        // - ToolChain conversion utilities (Required)
        //
        // ESTIMATED EFFORT: 4-5 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (task analysis feature)
        // - Change Budget: ~100 LOC
        // - Reviewer Requirements: Task decomposition expertise
        let task_analysis = TaskAnalysis {
            task_id: agent_workers::TaskId::new(),
            complexity_score: 0.5, // Temporary: default until ComplexTask conversion is implemented
            patterns: vec![], // Temporary: no patterns until DecompositionEngine analysis
            dependencies,
            subtask_scores: agent_workers::SubtaskScores {
                parallelization_score: if self.can_chain_parallelize(chain) { 0.8 } else { 0.2 },
                complexity_scores: vec![], // Temporary: empty until decomposition analysis
                estimated_durations: vec![], // Temporary: empty until decomposition analysis
            },
            recommended_workers: self.estimate_worker_requirements(chain),
            should_parallelize: self.can_chain_parallelize(chain),
        };

        Ok(task_analysis)
    }

    /// Decompose chain into parallel tasks
    async fn decompose_chain_into_tasks(
        &self,
        chain: &ToolChain,
        _analysis: &TaskAnalysis,
    ) -> Result<Vec<ParallelTask>, ParallelExecutionError> {
        let mut parallel_tasks = Vec::new();

        // Group nodes by parallel execution levels
        let execution_levels = self.compute_execution_levels(chain)?;

        for (level, nodes) in execution_levels {
            for node_idx in nodes {
                let node = &chain.dag[node_idx];
                let task_id = self.node_id_to_task_id(node_idx);

                let parallel_task = ParallelTask {
                    task_id: task_id.clone(),
                    node_idx,
                    node: node.clone(),
                    execution_level: level,
                    dependencies: self.get_node_dependencies(chain, node_idx),
                    estimated_duration_ms: self.estimate_node_duration(node),
                    resource_requirements: self.estimate_node_resources(node),
                };

                parallel_tasks.push(parallel_task);
            }
        }

        Ok(parallel_tasks)
    }

    /// Execute parallel tasks
    async fn execute_parallel_tasks(
        &self,
        tasks: Vec<ParallelTask>,
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<HashMap<String, ToolResult>, ParallelExecutionError> {
        let (result_tx, mut result_rx) = mpsc::channel(100);
        let semaphore = Arc::new(Semaphore::new(self.concurrency_limit));

        // Spawn worker tasks
        let mut handles = Vec::new();

        for task in tasks {
            let semaphore = semaphore.clone();
            let result_tx = result_tx.clone();
            let cancel_token = cancel_token.clone();
            let worker_manager = self.worker_manager.clone();
            let communication_hub = self.communication_hub.clone();
            let task_executor = self.chain_executor.clone();

            let handle = tokio::spawn(async move {
                let _permit = match semaphore.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => return,
                };

                if cancel_token.is_cancelled() {
                    return;
                }

                // Execute task with worker
                let result = Self::execute_single_task_with_worker_static(
                    &task_executor,
                    task,
                    worker_manager,
                    communication_hub,
                ).await;

                let _ = result_tx.send(result).await;
            });

            handles.push(handle);
        }

        // Collect results
        let mut results = HashMap::new();
        let mut completed_tasks = 0;

        drop(result_tx); // Close sender

        while let Some(result) = result_rx.recv().await {
            match result {
                Ok((task_id, tool_result)) => {
                    results.insert(task_id, tool_result);
                    completed_tasks += 1;
                }
                Err(e) => {
                    error!("Task execution failed: {}", e);
                    return Err(e);
                }
            }

            if completed_tasks >= handles.len() {
                break;
            }
        }

        // Wait for all tasks to complete
        for handle in handles {
            if let Err(e) = handle.await {
                error!("Worker task panicked: {}", e);
            }
        }

        Ok(results)
    }

    /// Execute single task with worker (static version for async spawn)
    async fn execute_single_task_with_worker_static(
        _task_executor: &Arc<ChainExecutor>,
        task: ParallelTask,
        _worker_manager: Arc<WorkerManager>,
        _communication_hub: Arc<CommunicationHub>,
    ) -> Result<(String, ToolResult), ParallelExecutionError> {
        // TODO: Create actual WorkerHandle or refactor to remove agent_memory dependency
        //       Currently uses placeholder; should create WorkerHandle or refactor to remove agent_memory requirement.
        //
        // COMPLETION CHECKLIST:
        // [ ] Add agent_memory dependency to this module
        // [ ] Or refactor WorkerHandle to not require agent_memory
        // [ ] Create actual WorkerHandle instance
        // [ ] Integrate WorkerHandle with worker execution
        // [ ] Handle memory access requirements
        // [ ] Add unit tests for WorkerHandle creation
        // [ ] Add integration tests with real workers
        // [ ] Verify worker execution with WorkerHandle
        //
        // ACCEPTANCE CRITERIA:
        // - WorkerHandle is created successfully
        // - Worker execution works with WorkerHandle
        // - Memory access requirements are satisfied
        // - Worker integration is functional
        //
        // DEPENDENCIES:
        // - agent_memory module (Required if adding dependency)
        // - WorkerHandle structure (Required)
        // - Worker execution infrastructure (Required)
        //
        // ESTIMATED EFFORT: 3-4 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (worker integration feature)
        // - Change Budget: ~60 LOC
        // - Reviewer Requirements: Worker infrastructure expertise
        let _worker = (); // Temporary: placeholder until WorkerHandle creation is implemented

        // Create worker task
        let _worker_task = WorkerTask {
            task_id: task.task_id.clone(),
            tool_id: task.node.tool_id.clone(),
            // TODO: Extract actual parameters from task node
            //       Currently uses placeholder JSON; should extract actual parameters from task node structure.
            //
            // COMPLETION CHECKLIST:
            // [ ] Extract parameters from task node structure
            // [ ] Map task inputs to worker task parameters
            // [ ] Handle parameter serialization correctly
            // [ ] Support various parameter types
            // [ ] Add unit tests for parameter extraction
            // [ ] Add integration tests with real tasks
            // [ ] Verify parameter extraction accuracy
            //
            // ACCEPTANCE CRITERIA:
            // - Parameters are extracted from task node correctly
            // - Task inputs are mapped to worker parameters
            // - Parameter serialization works correctly
            // - Various parameter types are supported
            //
            // DEPENDENCIES:
            // - Task node structure (Required)
            // - Parameter extraction utilities (Required)
            // - Parameter serialization utilities (Required)
            //
            // ESTIMATED EFFORT: 2-3 hours (medium confidence)
            // PRIORITY: Medium
            // BLOCKING: No
            //
            // GOVERNANCE:
            // - CAWS Tier: 2 (parameter handling feature)
            // - Change Budget: ~50 LOC
            // - Reviewer Requirements: Task parameter expertise
            parameters: serde_json::json!({"task": task.task_id, "inputs": []}), // Temporary: placeholder until parameter extraction is implemented
            timeout_ms: task.estimated_duration_ms * 2,
            priority: 1, // Default priority
        };

        // Stub: simulate task execution
        let result = ToolResult {
            tool_name: "stub_tool".to_string(),
            result: serde_json::json!({"status": "completed", "task_id": task.task_id}),
            metadata: crate::tool_execution::ExecutionMetadata {
                execution_time_ms: 100,
                memory_used_mb: 10.0,
                success: true,
                error_message: None,
                resource_usage: crate::tool_execution::ResourceUsage {
                    cpu_time_ms: 50,
                    peak_memory_mb: 10.0,
                    io_operations: 0,
                    network_bytes: 0,
                },
            },
            timestamp: chrono::Utc::now(),
        };

        Ok((task.task_id, result))
    }

    /// Execute single task with worker
    async fn execute_single_task_with_worker(
        &self,
        task: ParallelTask,
        _worker_manager: Arc<WorkerManager>,
        _communication_hub: Arc<CommunicationHub>,
    ) -> Result<(String, ToolResult), ParallelExecutionError> {
        // TODO: Implement proper WorkerHandle creation for parallel execution
        //       Currently uses placeholder; should implement comprehensive WorkerHandle creation that either adds agent_memory dependency or refactors WorkerHandle to not require it for proper parallel execution integration.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - WorkerHandle is created properly for parallel execution
        // - Memory access dependency is resolved (either added or refactored)
        // - Worker handle integrates correctly with parallel execution system
        // - Handle creation handles missing dependencies gracefully
        //
        // DEPENDENCIES:
        // - agent_memory dependency addition OR WorkerHandle refactoring (Required)
        // - Parallel execution integration (Required)
        // - Worker handle creation utilities (Required)
        //
        // ESTIMATED EFFORT: 8-12 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: Yes – Blocks parallel execution functionality
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (parallel execution integration)
        // - Change Budget: ~200 LOC
        // - Reviewer Requirements: Parallel execution and dependency management expertise
        let _worker = (); // Placeholder - actual WorkerHandle creation requires agent_memory

        // Create worker task
        let _worker_task = WorkerTask {
            task_id: task.task_id.clone(),
            tool_id: task.node.tool_id.clone(),
            parameters: serde_json::Value::Null, // Would be populated with actual inputs
            timeout_ms: task.estimated_duration_ms * 2,
            priority: self.calculate_task_priority(&task),
        };

        // Stub: simulate task execution
        let result = ToolResult {
            tool_name: "stub_tool".to_string(),
            result: serde_json::json!({"status": "completed", "task_id": task.task_id}),
            metadata: crate::tool_execution::ExecutionMetadata {
                execution_time_ms: 100,
                memory_used_mb: 10.0,
                success: true,
                error_message: None,
                resource_usage: crate::tool_execution::ResourceUsage {
                    cpu_time_ms: 50,
                    peak_memory_mb: 10.0,
                    io_operations: 0,
                    network_bytes: 0,
                },
            },
            timestamp: chrono::Utc::now(),
        };

        // Stub: communication hub result broadcasting
        // communication_hub.broadcast_result(&task.task_id, &result).await?;

        Ok((task.task_id, result))
    }

    /// Synthesize parallel results back into chain format
    async fn synthesize_parallel_results(
        &self,
        chain: &ToolChain,
        parallel_results: &HashMap<String, ToolResult>,
    ) -> Result<ExecutionResult, ParallelExecutionError> {
        let mut node_results = HashMap::new();
        let mut errors = Vec::new();
        let mut total_time = 0u64;

        // Map task results back to node indices
        for node_idx in chain.dag.node_indices() {
            let task_id = self.node_id_to_task_id(node_idx);

            if let Some(tool_result) = parallel_results.get(&task_id) {
                node_results.insert(node_idx, tool_result.result.clone());
                total_time = total_time.max(tool_result.metadata.execution_time_ms);
            } else {
                errors.push(format!("Missing result for task: {}", task_id));
            }
        }

        Ok(ExecutionResult {
            chain_hash: chain.plan_hash,
            success: errors.is_empty(),
            results: node_results,
            execution_time_ms: total_time,
            errors,
            cancelled_steps: Vec::new(),
        })
    }

    /// Estimate chain complexity
    fn estimate_chain_complexity(&self, chain: &ToolChain) -> f64 {
        let node_count = chain.dag.node_count() as f64;
        let edge_count = chain.dag.edge_count() as f64;
        let avg_cost = chain.estimated_cost / node_count.max(1.0);

        // Complexity based on structure and cost
        (node_count * 0.3) + (edge_count * 0.2) + (avg_cost.log10() * 0.5)
    }

    /// Check if chain can be parallelized
    fn can_chain_parallelize(&self, chain: &ToolChain) -> bool {
        // Check for cycles (already handled by DAG)
        // Check for high parallelization potential
        let node_count = chain.dag.node_count();
        let edge_count = chain.dag.edge_count();

        if node_count < 2 {
            return false; // Not enough nodes
        }

        // Calculate parallelism factor
        let avg_dependencies = if node_count > 0 {
            edge_count as f64 / node_count as f64
        } else {
            0.0
        };

        avg_dependencies < 1.5 // Low dependency ratio = high parallelism
    }

    /// Identify parallel execution sections
    fn identify_parallel_sections(&self, chain: &ToolChain) -> Vec<String> {
        let mut sections = Vec::new();

        // Find nodes with no dependencies (roots)
        let roots: Vec<_> = chain.dag.node_indices()
            .filter(|&idx| chain.dag.edges_directed(idx, petgraph::Direction::Incoming).count() == 0)
            .collect();

        if roots.len() > 1 {
            sections.push(format!("parallel_roots_{}", roots.len()));
        }

        // Find independent subgraphs
        // TODO: Implement proper independent subgraph detection
        //       Currently uses placeholder implementation; should detect truly independent subgraphs using graph algorithms.
        sections.push("independent_subgraphs".to_string());

        sections
    }

    /// Estimate resource requirements
    fn estimate_resource_requirements(&self, chain: &ToolChain) -> HashMap<String, u32> {
        let mut requirements = HashMap::new();

        requirements.insert("cpu_cores".to_string(), chain.dag.node_count().min(8) as u32);
        requirements.insert("memory_mb".to_string(), (chain.estimated_cost * 10.0) as u32);
        requirements.insert("network_bandwidth".to_string(), 100); // Mbps

        requirements
    }

    /// Estimate worker requirements
    fn estimate_worker_requirements(&self, chain: &ToolChain) -> usize {
        let node_count = chain.dag.node_count();

        if node_count <= 2 {
            1
        } else if node_count <= 4 {
            2
        } else {
            4
        }
    }

    /// Compute execution levels (topological levels)
    fn compute_execution_levels(&self, chain: &ToolChain) -> Result<HashMap<usize, Vec<petgraph::graph::NodeIndex>>, ParallelExecutionError> {
        use petgraph::visit::EdgeRef;
        use std::collections::HashSet;

        let mut levels = HashMap::new();
        let mut visited = HashSet::new();
        let mut current_level = 0;

        // Start with root nodes
        let mut current_nodes: Vec<_> = chain.dag.node_indices()
            .filter(|&idx| chain.dag.edges_directed(idx, petgraph::Direction::Incoming).count() == 0)
            .collect();

        while !current_nodes.is_empty() {
            levels.insert(current_level, current_nodes.clone());

            let mut next_level = Vec::new();

            for &node_idx in &current_nodes {
                visited.insert(node_idx);

                // Find nodes that depend on this node
                for neighbor in chain.dag.neighbors_directed(node_idx, petgraph::Direction::Outgoing) {
                    if !visited.contains(&neighbor) {
                        // Check if all dependencies of this neighbor are satisfied
                        let all_deps_satisfied = chain.dag.edges_directed(neighbor, petgraph::Direction::Incoming)
                            .all(|edge| visited.contains(&edge.source()));

                        if all_deps_satisfied && !next_level.contains(&neighbor) {
                            next_level.push(neighbor);
                        }
                    }
                }
            }

            current_nodes = next_level;
            current_level += 1;
        }

        Ok(levels)
    }

    /// Convert node index to task ID
    fn node_id_to_task_id(&self, node_idx: petgraph::graph::NodeIndex) -> String {
        format!("task_{}", node_idx.index())
    }

    /// Get node dependencies
    fn get_node_dependencies(&self, chain: &ToolChain, node_idx: petgraph::graph::NodeIndex) -> Vec<String> {
        chain.dag.edges_directed(node_idx, petgraph::Direction::Incoming)
            .map(|edge| self.node_id_to_task_id(edge.source()))
            .collect()
    }

    /// Estimate node duration
    fn estimate_node_duration(&self, node: &ToolNode) -> u64 {
        // Base on SLA and tool characteristics
        node.sla_ms as u64
    }

    /// Estimate node resources
    fn estimate_node_resources(&self, node: &ToolNode) -> HashMap<String, u32> {
        let mut resources = HashMap::new();

        resources.insert("cpu_percent".to_string(), 10); // 10% CPU
        resources.insert("memory_mb".to_string(), (node.cost_hint * 50.0) as u32); // Cost-based estimate

        resources
    }

    /// Calculate task priority
    fn calculate_task_priority(&self, task: &ParallelTask) -> u8 {
        // Higher priority for critical path tasks
        if task.execution_level == 0 {
            10 // Root tasks highest priority
        } else {
            5  // Other tasks medium priority
        }
    }
}

/// Parallel task representation
#[derive(Clone, Debug)]
pub struct ParallelTask {
    pub task_id: String,
    pub node_idx: petgraph::graph::NodeIndex,
    pub node: ToolNode,
    pub execution_level: usize,
    pub dependencies: Vec<String>,
    pub estimated_duration_ms: u64,
    pub resource_requirements: HashMap<String, u32>,
}

/// Worker task representation
#[derive(Clone, Debug, JsonSchema)]
pub struct WorkerTask {
    pub task_id: String,
    pub tool_id: String,
    pub parameters: serde_json::Value,
    pub timeout_ms: u64,
    pub priority: u8,
}

/// Parallel execution errors
#[derive(Debug, thiserror::Error)]
pub enum ParallelExecutionError {
    #[error("Sequential execution failed: {0}")]
    SequentialExecution(String),

    #[error("Parallel decomposition failed: {0}")]
    DecompositionError(String),

    #[error("Worker execution failed: {0}")]
    WorkerError(String),

    #[error("No available workers")]
    NoAvailableWorker,

    #[error("Task synthesis failed: {0}")]
    SynthesisError(String),

    #[error("Communication error: {0}")]
    CommunicationError(String),

    #[error("Timeout exceeded")]
    Timeout,
}
