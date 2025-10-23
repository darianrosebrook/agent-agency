//! Parallel Worker Integration
//!
//! Connects Tool Chain Executor with ParallelCoordinator for distributed
//! tool execution across multiple workers with load balancing and fault tolerance.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, Semaphore};
use tokio::sync::RwLock;
use tracing::{debug, info, warn, error};
use uuid::Uuid;

use crate::tool_chain_planner::{ToolChain, ToolNode, ToolEdge};
use crate::executor::{ChainExecutor, ExecutionResult};
use crate::tool_execution::{ToolExecutor, ToolResult};
use crate::tool_registry::ToolRegistry;

use parallel_workers::{
    ParallelCoordinator,
    ParallelCoordinatorConfig,
    DecompositionEngine,
    TaskAnalysis,
    Dependency,
    WorkerManager,
    CommunicationHub,
};

/// Parallel tool execution coordinator
pub struct ParallelToolCoordinator {
    chain_executor: Arc<ChainExecutor>,
    parallel_coordinator: Arc<ParallelCoordinator>,
    worker_manager: Arc<WorkerManager>,
    communication_hub: Arc<CommunicationHub>,
    execution_cache: Arc<RwLock<HashMap<String, ExecutionResult>>>,
    concurrency_limit: usize,
}

impl ParallelToolCoordinator {
    pub fn new(
        tool_executor: Arc<ToolExecutor>,
        tool_registry: Arc<ToolRegistry>,
        config: ParallelCoordinatorConfig,
    ) -> Self {
        let chain_executor = Arc::new(ChainExecutor::new(
            tool_executor.clone(),
            Arc::new(crate::schema_registry::JsonSchemaRegistry::new()),
            8, // concurrency limit
            30000, // default timeout
        ));

        let parallel_coordinator = Arc::new(ParallelCoordinator::new(config));
        let worker_manager = Arc::new(WorkerManager::new(8)); // 8 workers
        let communication_hub = Arc::new(CommunicationHub::new());

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
        cancel_token: tokio_util::sync::CancellationToken,
    ) -> Result<ExecutionResult, ParallelExecutionError> {
        info!("Executing tool chain with parallel workers: {}", chain.dag.node_count());

        // Analyze task for parallel decomposition
        let task_analysis = self.analyze_chain_for_parallelism(chain).await?;

        if !task_analysis.can_parallelize {
            info!("Chain cannot be parallelized, falling back to sequential execution");
            return self.chain_executor.execute(chain, cancel_token).await
                .map_err(ParallelExecutionError::SequentialExecution);
        }

        // Decompose into parallel tasks
        let parallel_tasks = self.decompose_chain_into_tasks(chain, &task_analysis).await?;

        // Execute with parallel coordinator
        let results = self.execute_parallel_tasks(parallel_tasks, cancel_token).await?;

        // Synthesize results back into chain format
        let execution_result = self.synthesize_parallel_results(chain, &results).await?;

        info!("Parallel execution completed: {} tasks, {}ms total",
              parallel_tasks.len(), execution_result.execution_time_ms);

        Ok(execution_result)
    }

    /// Analyze chain for parallel execution opportunities
    async fn analyze_chain_for_parallelism(
        &self,
        chain: &ToolChain,
    ) -> Result<TaskAnalysis, ParallelExecutionError> {
        let decomposition_engine = DecompositionEngine::new();

        // Convert chain to task analysis format
        let mut dependencies = Vec::new();

        // Build dependency graph
        for edge_idx in chain.dag.edge_indices() {
            let (source, target) = chain.dag.edge_endpoints(edge_idx).unwrap();
            let edge = chain.dag.edge_weight(edge_idx).unwrap();

            dependencies.push(Dependency {
                from_task: self.node_id_to_task_id(source),
                to_task: self.node_id_to_task_id(target),
                dependency_type: "data_flow".to_string(),
                metadata: HashMap::from([
                    ("from_port".to_string(), edge.from_port.clone()),
                    ("to_port".to_string(), edge.to_port.clone()),
                ]),
            });
        }

        let task_analysis = TaskAnalysis {
            task_id: format!("chain_{}", chain.plan_hash),
            complexity_score: self.estimate_chain_complexity(chain),
            estimated_duration_ms: chain.estimated_time_ms as u64,
            dependencies,
            can_parallelize: self.can_chain_parallelize(chain),
            parallelizable_sections: self.identify_parallel_sections(chain),
            resource_requirements: self.estimate_resource_requirements(chain),
            worker_requirements: self.estimate_worker_requirements(chain),
        };

        Ok(task_analysis)
    }

    /// Decompose chain into parallel tasks
    async fn decompose_chain_into_tasks(
        &self,
        chain: &ToolChain,
        analysis: &TaskAnalysis,
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

            let handle = tokio::spawn(async move {
                let _permit = match semaphore.acquire().await {
                    Ok(permit) => permit,
                    Err(_) => return,
                };

                if cancel_token.is_cancelled() {
                    return;
                }

                // Execute task with worker
                let result = self.execute_single_task_with_worker(
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

    /// Execute single task with worker
    async fn execute_single_task_with_worker(
        &self,
        task: ParallelTask,
        worker_manager: Arc<WorkerManager>,
        communication_hub: Arc<CommunicationHub>,
    ) -> Result<(String, ToolResult), ParallelExecutionError> {
        // Get available worker
        let worker = worker_manager.get_available_worker().await
            .ok_or(ParallelExecutionError::NoAvailableWorker)?;

        // Create worker task
        let worker_task = WorkerTask {
            task_id: task.task_id.clone(),
            tool_id: task.node.tool_id.clone(),
            parameters: serde_json::Value::Null, // Would be populated with actual inputs
            timeout_ms: task.estimated_duration_ms * 2,
            priority: self.calculate_task_priority(&task),
        };

        // Execute with worker
        let result = worker.execute_task(worker_task).await?;

        // Send result via communication hub
        communication_hub.broadcast_result(&task.task_id, &result).await?;

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
                total_time = total_time.max(tool_result.execution_time_ms.unwrap_or(0));
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
        // This is a simplified implementation
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
    fn estimate_worker_requirements(&self, chain: &ToolChain) -> Vec<String> {
        let node_count = chain.dag.node_count();

        if node_count <= 2 {
            vec!["single_worker".to_string()]
        } else if node_count <= 4 {
            vec!["dual_workers".to_string()]
        } else {
            vec!["multi_workers".to_string()]
        }
    }

    /// Compute execution levels (topological levels)
    fn compute_execution_levels(&self, chain: &ToolChain) -> Result<HashMap<usize, Vec<petgraph::graph::NodeIndex>>, ParallelExecutionError> {
        use petgraph::visit::Topo;
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
#[derive(Clone, Debug)]
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
    SequentialExecution(#[from] Box<dyn std::error::Error + Send + Sync>),

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
