//! Parallel Task Execution
//!
//! Provides parallel task decomposition and coordination capabilities
//! consolidated from the parallel-workers/ crate.

use schemars::JsonSchema;
use crate::worker_types::*;
use crate::parallel_types::{TaskDependency, ParallelExecutionPlan, CoordinationStrategy, DependencyType, SubTask, TaskResult, WorkerId, SubTaskId, SubTaskStatus, Priority as ParallelPriority};
use crate::decomposition::TaskDecomposer;
use crate::execution::ToolExecutor;
use crate::worker_types::TaskContext;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{info, warn};

/// Configuration for parallel execution

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ParallelExecutionConfig {
    pub max_parallel_tasks: usize,
    pub decomposition_depth: usize,
    pub enable_dependency_tracking: bool,
    pub coordination_timeout_seconds: u64,
}

impl Default for ParallelExecutionConfig {
    fn default() -> Self {
        Self {
            max_parallel_tasks: 10,
            decomposition_depth: 3,
            enable_dependency_tracking: true,
            coordination_timeout_seconds: 300,
        }
    }
}

/// Parallel execution coordinator
pub struct ParallelCoordinator {
    config: ParallelExecutionConfig,
    decomposer: Arc<TaskDecomposer>,
    tool_executor: Arc<ToolExecutor>,
    active_executions: Arc<RwLock<HashMap<TaskId, ParallelExecutionPlan>>>,
}

impl ParallelCoordinator {
    /// Create a new parallel coordinator
    pub fn new() -> Self {
        Self::with_config(ParallelExecutionConfig::default())
    }

    /// Create coordinator with custom configuration
    pub fn with_config(config: ParallelExecutionConfig) -> Self {
        Self {
            config: config.clone(),
            decomposer: Arc::new(TaskDecomposer::new()),
            tool_executor: Arc::new(ToolExecutor::new()),
            active_executions: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Decompose a complex task into parallel subtasks
    pub async fn decompose_task(&self, task: &TaskDefinition) -> Result<ParallelExecutionPlan, ParallelError> {
        info!("Decomposing task {}: {}", task.id, task.name);

        // Analyze task complexity
        let analysis = self.decomposer.analyze_task(task).await?;

        // Create subtasks based on analysis
        let subtasks = self.create_subtasks(&analysis, task).await?;

        // Determine dependencies
        let dependencies = self.calculate_dependencies(&subtasks).await?;

        // Choose coordination strategy
        let strategy = self.select_coordination_strategy(&analysis);

        let plan = ParallelExecutionPlan {
            main_task: task.clone(),
            subtasks,
            dependencies,
            coordination_strategy: strategy,
        };

        // Store the execution plan
        let mut executions = self.active_executions.write().await;
        executions.insert(task.id, plan.clone());

        Ok(plan)
    }

    /// Execute a parallel execution plan
    pub async fn execute_parallel(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        info!("Executing parallel plan for task {}", plan.main_task.id);

        match plan.coordination_strategy {
            CoordinationStrategy::FullyParallel => {
                self.execute_fully_parallel(plan).await
            }
            CoordinationStrategy::SequentialDependencies => {
                self.execute_with_dependencies(plan).await
            }
            CoordinationStrategy::Adaptive => {
                self.execute_adaptive(plan).await
            }
        }
    }

    /// Execute all subtasks in parallel without dependencies
    async fn execute_fully_parallel(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        let mut handles = Vec::new();

        for subtask in &plan.subtasks {
            let tool_executor = Arc::clone(&self.tool_executor);
            let subtask = subtask.clone();

            let handle = tokio::spawn(async move {
                let tool_id = subtask.metadata.get("pattern_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "general-purpose".to_string());
                
                let context = TaskContext {
                    task_id: subtask.id.0,
                    worker_id: WorkerId::new().0,
                    start_time: chrono::Utc::now(),
                    timeout_ms: subtask.estimated_duration.as_millis() as u64,
                    retry_count: 0,
                    max_retries: 3,
                    metadata: HashMap::new(),
                    tool_id: Some(tool_id),
                    parameters: HashMap::new(), // Parameters would come from subtask metadata if needed
                };

                tool_executor.execute_tool(context).await
            });

            handles.push(handle);
        }

        // Wait for all subtasks to complete
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(result) => results.push(result?),
                Err(e) => {
                    warn!("Subtask execution failed: {}", e);
                    return Err(ParallelError::SubtaskFailed(e.to_string()));
                }
            }
        }

        Ok(results)
    }

    /// Execute subtasks respecting dependencies
    async fn execute_with_dependencies(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        let mut results = Vec::new();
        let mut completed_tasks = std::collections::HashSet::new();

        // Simple topological sort for dependencies
        let mut remaining_tasks: Vec<_> = plan.subtasks.iter().cloned().collect();

        while !remaining_tasks.is_empty() {
            // Find tasks with satisfied dependencies
            let mut executable_tasks = Vec::new();

            for task in &remaining_tasks {
                let dependencies_satisfied = plan.dependencies
                    .iter()
                    .filter(|dep| dep.dependent_task == task.id)
                    .all(|dep| completed_tasks.contains(&dep.dependency_task));

                if dependencies_satisfied {
                    executable_tasks.push(task.clone());
                }
            }

            if executable_tasks.is_empty() {
                return Err(ParallelError::CircularDependency);
            }

            // Execute executable tasks in parallel
            let mut handles = Vec::new();
            for task in executable_tasks {
                let tool_executor = Arc::clone(&self.tool_executor);
                let task = task.clone();

                let handle = tokio::spawn(async move {
                    let tool_id = task.metadata.get("pattern_type")
                        .and_then(|v| v.as_str())
                        .map(|s| s.to_string())
                        .unwrap_or_else(|| "general-purpose".to_string());
                    
                    let context = TaskContext {
                        task_id: task.id.0,
                        worker_id: WorkerId::new().0,
                        start_time: chrono::Utc::now(),
                        timeout_ms: 60000,
                        retry_count: 0,
                        max_retries: 3,
                        metadata: HashMap::new(),
                        tool_id: Some(tool_id),
                        parameters: HashMap::new(), // Parameters would come from subtask metadata if needed
                    };

                    tool_executor.execute_tool(context).await
                });

                handles.push(handle);
            }

            // Wait for this batch to complete
            for handle in handles {
                let result = handle.await??;
                results.push(result);
            }

            // Mark tasks as completed
            for task in &remaining_tasks {
                completed_tasks.insert(task.id);
            }

            // Remove completed tasks
            remaining_tasks.retain(|task| !completed_tasks.contains(&task.id));
        }

        Ok(results)
    }

    /// Execute with adaptive coordination based on results
    async fn execute_adaptive(&self, plan: ParallelExecutionPlan) -> Result<Vec<TaskResult>, ParallelError> {
        // Start with parallel execution, then adapt based on results
        let mut results = self.execute_fully_parallel(plan.clone()).await?;

        // Analyze results and potentially re-execute failed tasks
        let failed_tasks: Vec<_> = results.iter()
            .filter(|r| !r.success)
            .collect();

        if !failed_tasks.is_empty() {
            warn!("{} subtasks failed, attempting recovery", failed_tasks.len());

            // Retry failed tasks sequentially
            for failed_result in failed_tasks {
                let subtask = plan.subtasks.iter()
                    .find(|t| t.id == failed_result.task_id)
                    .ok_or(ParallelError::SubtaskNotFound)?;

                let tool_id = subtask.metadata.get("pattern_type")
                    .and_then(|v| v.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| "general-purpose".to_string());
                
                let context = TaskContext {
                    task_id: subtask.id.0,
                    worker_id: WorkerId::new().0,
                    start_time: chrono::Utc::now(),
                    timeout_ms: 120000, // Longer timeout for retry
                    retry_count: 1,
                    max_retries: 3,
                    metadata: HashMap::new(),
                    tool_id: Some(tool_id),
                    parameters: HashMap::new(), // Parameters would come from subtask metadata if needed
                };

                let retry_result = self.tool_executor.execute_tool(context).await?;
                results.push(retry_result);
            }
        }

        Ok(results)
    }

    /// Synthesize results from multiple subtasks
    pub async fn synthesize_results(&self, results: Vec<TaskResult>) -> Result<TaskResult, ParallelError> {
        // Combine outputs from all subtasks
        let success = results.iter().all(|r| matches!(r.status, TaskStatus::Completed));

        let combined_output = if success {
            let outputs: Vec<_> = results.iter()
                .filter_map(|r| r.output.as_ref())
                .collect();

            Some(serde_json::json!({
                "subtask_results": outputs,
                "total_subtasks": results.len(),
                "successful_subtasks": results.iter().filter(|r| r.success).count()
            }))
        } else {
            None
        };

        let total_execution_time: u64 = results.iter().map(|r| r.execution_time_ms).sum();

        Ok(TaskResult {
            task_id: TaskId::new_v4(), // Would be the main task ID
            status: if success { TaskStatus::Completed } else { TaskStatus::Failed },
            output: combined_output,
            error_message: if success { None } else { Some("Some subtasks failed".to_string()) },
            execution_time_ms: total_execution_time,
            tool_used: "parallel-coordinator".to_string(),
            quality_score: None,
        })
    }

    /// Create subtasks from task analysis
    async fn create_subtasks(&self, analysis: &crate::decomposition::TaskAnalysis, main_task: &TaskDefinition) -> Result<Vec<SubTask>, ParallelError> {
        let mut subtasks = Vec::new();
        let task_id = crate::parallel_types::TaskId(main_task.id);

        // Create subtasks based on analysis patterns
        for (idx, pattern) in analysis.patterns.iter().enumerate() {
            let (title, description) = self.extract_pattern_info(pattern);
            let complexity = self.calculate_pattern_complexity(pattern);
            let priority = self.convert_priority(&main_task.priority);
            
            let subtask = SubTask {
                id: SubTaskId::new(),
                parent_task_id: task_id.clone(),
                title,
                description,
                complexity,
                dependencies: vec![],
                assigned_worker: None,
                status: SubTaskStatus::Pending,
                priority,
                estimated_duration: std::time::Duration::from_secs(60 * (idx + 1) as u64),
                metadata: {
                    let mut meta = HashMap::new();
                    meta.insert("pattern_index".to_string(), serde_json::json!(idx));
                    meta.insert("pattern_type".to_string(), serde_json::json!(self.get_pattern_type_name(pattern)));
                    meta
                },
            };
            subtasks.push(subtask);
        }

        Ok(subtasks)
    }
    
    /// Extract title and description from TaskPattern enum
    fn extract_pattern_info(&self, pattern: &crate::decomposition::TaskPattern) -> (String, String) {
        match pattern {
            crate::decomposition::TaskPattern::CompilationErrors { error_groups } => {
                let count: usize = error_groups.iter().map(|eg| eg.error_count).sum();
                (
                    format!("Fix {} compilation errors", count),
                    format!("Resolve {} compilation errors across {} files", 
                        count, error_groups.len())
                )
            }
            crate::decomposition::TaskPattern::RefactoringOperations { operations } => {
                (
                    format!("Refactor {} operations", operations.len()),
                    format!("Perform {} refactoring operations", operations.len())
                )
            }
            crate::decomposition::TaskPattern::TestingGaps { missing_tests } => {
                (
                    format!("Add {} missing tests", missing_tests.len()),
                    format!("Implement {} missing test cases", missing_tests.len())
                )
            }
            crate::decomposition::TaskPattern::DocumentationNeeds { files_needing_docs } => {
                (
                    format!("Document {} files", files_needing_docs.len()),
                    format!("Add documentation for {} files", files_needing_docs.len())
                )
            }
        }
    }
    
    /// Calculate complexity score for a pattern
    fn calculate_pattern_complexity(&self, pattern: &crate::decomposition::TaskPattern) -> f64 {
        match pattern {
            crate::decomposition::TaskPattern::CompilationErrors { error_groups } => 
                error_groups.len() as f64 * 2.0,
            crate::decomposition::TaskPattern::RefactoringOperations { operations } => 
                operations.len() as f64 * 3.0,
            crate::decomposition::TaskPattern::TestingGaps { missing_tests } => 
                missing_tests.len() as f64 * 1.5,
            crate::decomposition::TaskPattern::DocumentationNeeds { files_needing_docs } => 
                files_needing_docs.len() as f64 * 0.5,
        }
    }
    
    /// Convert TaskPriority to ParallelPriority
    fn convert_priority(&self, priority: &TaskPriority) -> ParallelPriority {
        match priority {
            TaskPriority::Low => ParallelPriority::Low,
            TaskPriority::Medium => ParallelPriority::Medium,
            TaskPriority::High => ParallelPriority::High,
            TaskPriority::Critical => ParallelPriority::Critical,
        }
    }
    
    /// Get pattern type name as string
    fn get_pattern_type_name(&self, pattern: &crate::decomposition::TaskPattern) -> String {
        match pattern {
            crate::decomposition::TaskPattern::CompilationErrors { .. } => "CompilationErrors".to_string(),
            crate::decomposition::TaskPattern::RefactoringOperations { .. } => "RefactoringOperations".to_string(),
            crate::decomposition::TaskPattern::TestingGaps { .. } => "TestingGaps".to_string(),
            crate::decomposition::TaskPattern::DocumentationNeeds { .. } => "DocumentationNeeds".to_string(),
        }
    }

    /// Calculate dependencies between subtasks by analyzing:
    /// - Explicit dependency lists
    /// - File references in metadata/description
    /// - Input/output relationships
    /// - Resource dependencies
    /// - Tool-based dependencies
    async fn calculate_dependencies(&self, subtasks: &[SubTask]) -> Result<Vec<TaskDependency>, ParallelError> {
        use std::collections::{HashMap, HashSet};
        
        let mut dependencies = Vec::new();
        let mut file_to_tasks: HashMap<String, HashSet<usize>> = HashMap::new();

        // Build file reference map from metadata and descriptions
        for (idx, subtask) in subtasks.iter().enumerate() {
            let mut files = HashSet::new();
            
            // Extract files from description
            if let Some(files_in_desc) = self.extract_file_references(&subtask.description) {
                files.extend(files_in_desc);
            }
            
            // Extract files from metadata
            if let Some(files_metadata) = subtask.metadata.get("files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<HashSet<_>>())
            {
                files.extend(files_metadata);
            }
            
            // Extract input/output files from metadata
            if let Some(input_files) = subtask.metadata.get("input_files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<HashSet<_>>())
            {
                files.extend(input_files);
            }
            
            if let Some(output_files) = subtask.metadata.get("output_files")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect::<HashSet<_>>())
            {
                files.extend(output_files);
            }
            
            // Map files to tasks
            for file in files {
                file_to_tasks.entry(file).or_insert_with(HashSet::new).insert(idx);
            }
        }

        // Analyze dependencies
        for (i, subtask) in subtasks.iter().enumerate() {
            // Check explicit dependencies first
            for dep_id in &subtask.dependencies {
                if let Some(j) = subtasks.iter().position(|s| s.id == *dep_id) {
                    if j < i {
                        dependencies.push(TaskDependency {
                            dependent_task: subtask.id,
                            dependency_task: subtasks[j].id,
                            dependency_type: DependencyType::Sequential,
                        });
                    }
                }
            }
            
            // Check file-based dependencies
            let task1_files = self.get_task_files(subtask);
            for (j, subtask2) in subtasks.iter().enumerate() {
                if j >= i {
                    continue;
                }
                
                let task2_files = self.get_task_files(subtask2);
                
                // Data dependency: task1 reads files that task2 writes
                let data_dependency = !task1_files.is_empty() && 
                    !task2_files.is_empty() &&
                    task1_files.iter().any(|f| task2_files.contains(f));
                
                // Resource dependency: same files accessed
                let resource_dependency = !task1_files.is_empty() &&
                    !task2_files.is_empty() &&
                    task1_files.iter().any(|f| task2_files.contains(f));
                
                // Use has_dependency for specialty/tool-based dependencies
                if self.has_dependency(subtask, subtask2) || data_dependency || resource_dependency {
                    let dep_type = if data_dependency {
                        DependencyType::Data
                    } else if resource_dependency {
                        DependencyType::Resource
                    } else {
                        DependencyType::Sequential
                    };
                    
                    dependencies.push(TaskDependency {
                        dependent_task: subtask.id,
                        dependency_task: subtask2.id,
                        dependency_type: dep_type,
                    });
                }
            }
        }

        Ok(dependencies)
    }
    
    /// Extract file references from description text
    fn extract_file_references(&self, description: &str) -> Option<HashSet<String>> {
        use regex::Regex;
        
        // Match common file path patterns
        let file_pattern = Regex::new(r#"(?:^|\s)(?:\./)?([a-zA-Z0-9_\-./]+\.(?:rs|ts|tsx|js|jsx|py|go|java|cpp|h|hpp|md|json|yaml|yml|toml|sh|sql|css|html))(?:\s|$)"#)
            .ok()?;
        
        let mut files = HashSet::new();
        for cap in file_pattern.captures_iter(description) {
            if let Some(file) = cap.get(1) {
                files.insert(file.as_str().to_string());
            }
        }
        
        if files.is_empty() {
            None
        } else {
            Some(files)
        }
    }
    
    /// Get all files referenced by a task (from metadata and description)
    fn get_task_files(&self, subtask: &SubTask) -> HashSet<String> {
        let mut files = HashSet::new();
        
        // Extract from description
        if let Some(desc_files) = self.extract_file_references(&subtask.description) {
            files.extend(desc_files);
        }
        
        // Extract from metadata
        if let Some(files_array) = subtask.metadata.get("files")
            .and_then(|v| v.as_array())
        {
            for v in files_array {
                if let Some(f) = v.as_str() {
                    files.insert(f.to_string());
                }
            }
        }
        
        // Extract input/output files
        if let Some(input_files) = subtask.metadata.get("input_files")
            .and_then(|v| v.as_array())
        {
            for v in input_files {
                if let Some(f) = v.as_str() {
                    files.insert(f.to_string());
                }
            }
        }
        
        if let Some(output_files) = subtask.metadata.get("output_files")
            .and_then(|v| v.as_array())
        {
            for v in output_files {
                if let Some(f) = v.as_str() {
                    files.insert(f.to_string());
                }
            }
        }
        
        files
    }

    /// Select coordination strategy based on task analysis
    fn select_coordination_strategy(&self, analysis: &crate::decomposition::TaskAnalysis) -> CoordinationStrategy {
        // Analyze task characteristics to determine optimal coordination strategy
        
        // Factor 1: Parallelization score - higher score favors parallel execution
        let parallelization_score = analysis.subtask_scores.parallelization_score;
        
        // Factor 2: Task complexity - high complexity may need sequential dependencies
        let complexity_score = analysis.complexity_score;
        
        // Factor 3: Number of patterns - more patterns may benefit from parallel execution
        let pattern_count = analysis.patterns.len();
        
        // Factor 4: Recommended workers - if few workers, sequential may be better
        let recommended_workers = analysis.recommended_workers;
        
        // Decision logic:
        // - High parallelization score (>0.7) + many workers (>4) -> FullyParallel
        // - Medium parallelization (0.4-0.7) + dependencies -> SequentialDependencies
        // - Low parallelization (<0.4) or high complexity -> SequentialDependencies
        // - Otherwise -> Adaptive
        
        if parallelization_score > 0.7 && recommended_workers > 4 && pattern_count > 2 {
            CoordinationStrategy::FullyParallel
        } else if parallelization_score < 0.4 || complexity_score > 0.8 {
            CoordinationStrategy::SequentialDependencies
        } else if parallelization_score >= 0.4 && parallelization_score <= 0.7 {
            CoordinationStrategy::SequentialDependencies
        } else {
            CoordinationStrategy::Adaptive
        }
    }

    /// Select appropriate tool for a task pattern
    async fn select_tool_for_pattern(&self, pattern: &crate::decomposition::TaskPattern) -> Result<String, ParallelError> {
        // Map TaskPattern enum variants to tool identifiers
        let tool_id = match pattern {
            crate::decomposition::TaskPattern::CompilationErrors { .. } => "compilation-fixer".to_string(),
            crate::decomposition::TaskPattern::RefactoringOperations { .. } => "refactoring-assistant".to_string(),
            crate::decomposition::TaskPattern::TestingGaps { .. } => "test-generator".to_string(),
            crate::decomposition::TaskPattern::DocumentationNeeds { .. } => "documentation-generator".to_string(),
        };
        Ok(tool_id)
    }

    /// Check if two subtasks have a dependency
    fn has_dependency(&self, task1: &SubTask, task2: &SubTask) -> bool {
        // Analyze task relationships to detect dependencies
        
        // Dependency 1: Explicit dependency list
        // Check if task1 explicitly depends on task2
        if task1.dependencies.contains(&task2.id) {
            return true;
        }
        
        // Dependency 2: Description-based dependencies
        // Some tasks naturally depend on others (e.g., testing depends on compilation)
        let desc1 = task1.description.to_lowercase();
        let desc2 = task2.description.to_lowercase();
        
        let specialty_dependency = 
            (desc1.contains("test") && desc2.contains("compile")) ||
            (desc1.contains("refactor") && desc2.contains("compile")) ||
            (desc1.contains("document") && desc2.contains("refactor")) ||
            (desc1.contains("document") && desc2.contains("compile"));
        
        // Dependency 3: Tool-based dependencies (check metadata for tool info)
        // Some tools depend on outputs from other tools
        let tool_dependency = {
            let tool1 = task1.metadata.get("pattern_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let tool2 = task2.metadata.get("pattern_type")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            
            (tool1.contains("test") && tool2.contains("compile")) ||
            (tool1.contains("document") && tool2.contains("refactor")) ||
            (tool1.contains("document") && tool2.contains("compile"))
        };
        
        specialty_dependency || tool_dependency
    }
}

/// Errors from parallel execution

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
enum ParallelError {
    #[error("Task decomposition failed: {0}")]
    DecompositionFailed(String),

    #[error("Circular dependency detected")]
    CircularDependency,

    #[error("Subtask not found")]
    SubtaskNotFound,

    #[error("Subtask execution failed: {0}")]
    SubtaskFailed(String),

    #[error("Result synthesis failed: {0}")]
    SynthesisFailed(String),
}
