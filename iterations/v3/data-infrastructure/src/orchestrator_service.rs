//! Orchestrator Service
//!
//! Modern orchestrator service wrapper for API integration.
//! Provides clean interface for task execution, chat, and monitoring.
//!
//! # CRITICAL: Observational API Design
//!
//! **This service is designed for OBSERVATION, not manipulation.**
//!
//! The API acts as a "doctor's MRI machine" - it observes what's happening inside
//! the orchestrator without directly controlling execution. This preserves research
//! integrity by ensuring the orchestrator maintains full autonomy over its execution
//! lifecycle.
//!
//! ## Design Principles
//!
//! 1. **Observation Only**: All methods observe orchestrator state, never manipulate it directly
//! 2. **Request-Based Control**: Control operations (pause/resume/cancel) are requests that
//!    are logged in chain-of-thought, but the orchestrator decides whether to honor them
//! 3. **Research Integrity**: No direct manipulation of execution state - orchestrator maintains
//!    full control over its own execution lifecycle
//! 4. **Agent Autonomy**: Agents use their own connections to task execution, not through the API
//!
//! ## Usage Pattern
//!
//! - **Submit tasks**: Request orchestrator to start a task (orchestrator handles execution)
//! - **Observe state**: Query task status, chain of thought, council decisions, worker actions
//! - **Request control**: Request pause/resume/cancel (orchestrator decides if safe)
//! - **Never manipulate**: Never directly change execution state - only observe and request
//!
//! ## Why This Matters
//!
//! Direct manipulation of orchestrator execution state would compromise research integrity.
//! By maintaining strict observation boundaries, we ensure that:
//! - Orchestrator decisions are autonomous and reproducible
//! - Research results are not contaminated by external manipulation
//! - The orchestrator's chain of thought accurately reflects its own reasoning
//! - Agents maintain their own execution connections independently
//!
//! @author @darianrosebrook

use anyhow::Result;
use chrono::Utc;
use serde_json::Value as JsonValue;
use std::sync::Arc;
use tracing::{error, info, warn};
use uuid::Uuid;

use agent_agency_contracts::{
    types::prelude::*, ExecutionArtifacts, ExecutionMode, TaskDescriptor, WorkingSpec,
};

use crate::simple_client::DatabaseClient;

/// Trait for task execution (allows dependency injection without circular dependencies)
#[async_trait::async_trait]
pub trait TaskExecutor: Send + Sync {
    async fn execute_task(
        &self,
        task_descriptor: &TaskDescriptor,
    ) -> Result<ExecutionArtifacts, anyhow::Error>;
}

/// Orchestrator service for API integration
#[derive(Clone)]
pub struct OrchestratorService {
    /// Database client for persistence
    _db_client: Arc<DatabaseClient>,

    /// Task executor (optional - can be injected when available)
    task_executor: Option<Arc<dyn TaskExecutor>>,

    /// Active task tracking
    active_tasks: Arc<tokio::sync::RwLock<std::collections::HashMap<Uuid, TaskExecutionState>>>,

    /// Pending task queue for when executor is unavailable
    pending_task_queue: Arc<tokio::sync::RwLock<Vec<QueuedTask>>>,

    /// Maximum queue size to prevent memory exhaustion
    max_queue_size: usize,
}

/// Queued task waiting for executor to become available
#[derive(Debug, Clone)]
struct QueuedTask {
    task_id: Uuid,
    task_descriptor: TaskDescriptor,
    queued_at: chrono::DateTime<Utc>,
    priority: i32, // Higher priority = execute first
}

/// Task execution state
#[derive(Debug, Clone)]
pub struct TaskExecutionState {
    pub task_id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub working_spec: Option<WorkingSpec>,
    pub artifacts: Option<ExecutionArtifacts>,
    pub started_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub completed_at: Option<chrono::DateTime<Utc>>,
    pub error_message: Option<String>,
    pub chain_of_thought: Vec<ChainOfThoughtEntry>,
    pub council_decisions: Vec<CouncilDecision>,
    pub worker_actions: Vec<WorkerAction>,
}

/// Lightweight task summary for listing endpoints
/// Avoids cloning large vectors (chain_of_thought, council_decisions, worker_actions)
#[derive(Debug, Clone)]
pub struct TaskSummary {
    pub task_id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub started_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
}

/// Task status
#[derive(Debug, Clone, PartialEq)]
pub enum TaskStatus {
    Pending,
    Planning,
    Executing,
    QualityCheck,
    Refining,
    Paused,
    Completed,
    Failed,
    Cancelled,
}

/// Chain of thought entry
#[derive(Debug, Clone)]
pub struct ChainOfThoughtEntry {
    pub timestamp: chrono::DateTime<Utc>,
    pub phase: String,
    pub reasoning: String,
    pub decision: String,
    pub context: JsonValue,
}

/// Council decision
#[derive(Debug, Clone)]
pub struct CouncilDecision {
    pub timestamp: chrono::DateTime<Utc>,
    pub judge: String,
    pub verdict: String,
    pub reasoning: String,
    pub confidence: f64,
}

/// Worker action
#[derive(Debug, Clone)]
pub struct WorkerAction {
    pub timestamp: chrono::DateTime<Utc>,
    pub worker_id: Uuid,
    pub action: String,
    pub result: String,
    pub artifacts: Vec<String>,
}

impl OrchestratorService {
    /// Create a new orchestrator service
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self {
            _db_client: db_client,
            task_executor: None,
            active_tasks: Arc::new(tokio::sync::RwLock::new(std::collections::HashMap::new())),
            pending_task_queue: Arc::new(tokio::sync::RwLock::new(Vec::new())),
            max_queue_size: 1000, // Default max queue size
        }
    }

    /// Initialize with task executor (when available)
    /// This will process any queued tasks when executor becomes available
    pub fn with_task_executor(mut self, executor: Arc<dyn TaskExecutor>) -> Self {
        self.task_executor = Some(executor.clone());
        
        // Process queued tasks in background
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(e) = service.process_queued_tasks(executor).await {
                error!("Failed to process queued tasks: {}", e);
            }
        });
        
        self
    }

    /// Process queued tasks when executor becomes available
    async fn process_queued_tasks(
        &self,
        executor: Arc<dyn TaskExecutor>,
    ) -> Result<()> {
        info!("Processing queued tasks...");
        
        loop {
            // Get next queued task
            let queued_task = {
                let mut queue = self.pending_task_queue.write().await;
                queue.pop()
            };
            
            if let Some(queued_task) = queued_task {
                info!("Processing queued task {} (priority: {}, queued at: {})", 
                    queued_task.task_id, 
                    queued_task.priority,
                    queued_task.queued_at
                );
                
                // Update task state to executing
                {
                    let mut tasks = self.active_tasks.write().await;
                    if let Some(task) = tasks.get_mut(&queued_task.task_id) {
                        task.status = TaskStatus::Executing;
                        task.error_message = None; // Clear error message
                        task.updated_at = Utc::now();
                        
                        // Record in chain of thought
                        task.chain_of_thought.push(ChainOfThoughtEntry {
                            timestamp: Utc::now(),
                            phase: "execution".to_string(),
                            reasoning: "Task executor became available, executing queued task".to_string(),
                            decision: "Dequeuing and executing task".to_string(),
                            context: serde_json::json!({
                                "queued_at": queued_task.queued_at,
                                "wait_time_seconds": (Utc::now() - queued_task.queued_at).num_seconds(),
                            }),
                        });
                    }
                }
                
                // Execute task
                match executor.execute_task(&queued_task.task_descriptor).await {
                    Ok(artifacts) => {
                        info!("Queued task {} completed successfully", queued_task.task_id);
                        
                        // Update task state with results
                        let mut tasks = self.active_tasks.write().await;
                        if let Some(task) = tasks.get_mut(&queued_task.task_id) {
                            task.status = TaskStatus::Completed;
                            task.artifacts = Some(artifacts.clone());
                            task.completed_at = Some(Utc::now());
                            task.updated_at = Utc::now();
                            
                            // Record completion in chain of thought
                            task.chain_of_thought.push(ChainOfThoughtEntry {
                                timestamp: Utc::now(),
                                phase: "completion".to_string(),
                                reasoning: "Queued task execution completed successfully".to_string(),
                                decision: "Task marked as completed".to_string(),
                                context: serde_json::json!({
                                    "working_spec_id": artifacts.working_spec_id,
                                    "iteration": artifacts.iteration,
                                }),
                            });
                        }
                    }
                    Err(e) => {
                        error!("Queued task {} execution failed: {}", queued_task.task_id, e);
                        
                        // Update task state with error
                        let mut tasks = self.active_tasks.write().await;
                        if let Some(task) = tasks.get_mut(&queued_task.task_id) {
                            task.status = TaskStatus::Failed;
                            task.error_message = Some(format!("Execution failed: {}", e));
                            task.completed_at = Some(Utc::now());
                            task.updated_at = Utc::now();
                            
                            // Record error in chain of thought
                            task.chain_of_thought.push(ChainOfThoughtEntry {
                                timestamp: Utc::now(),
                                phase: "error".to_string(),
                                reasoning: format!("Queued task execution failed: {}", e),
                                decision: "Task marked as failed".to_string(),
                                context: serde_json::json!({ "error": e.to_string() }),
                            });
                        }
                    }
                }
                
                // Small delay to avoid overwhelming the executor
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            } else {
                // No more queued tasks, exit
                break;
            }
        }
        
        info!("Finished processing queued tasks");
        Ok(())
    }

    /// Execute a task from description
    ///
    /// **OBSERVATIONAL API**: This submits a task request to the orchestrator.
    /// The orchestrator handles all execution independently. We only observe the results.
    /// Execution runs in the background via the orchestrator's own lifecycle.
    pub async fn execute_task(
        &self,
        description: String,
        execution_mode: Option<String>,
        context: Option<String>,
    ) -> Result<Uuid> {
        let task_id = Uuid::new_v4();
        info!(
            "Executing task {}: {}",
            task_id,
            description.chars().take(100).collect::<String>()
        );

        // Create initial task state
        let task_state = TaskExecutionState {
            task_id,
            description: description.clone(),
            status: TaskStatus::Pending,
            working_spec: None,
            artifacts: None,
            started_at: Utc::now(),
            updated_at: Utc::now(),
            completed_at: None,
            error_message: None,
            chain_of_thought: Vec::new(),
            council_decisions: Vec::new(),
            worker_actions: Vec::new(),
        };

        // Store task state
        {
            let mut tasks = self.active_tasks.write().await;
            tasks.insert(task_id, task_state.clone());
        }

        // Start execution in background
        let service = self.clone();
        tokio::spawn(async move {
            if let Err(e) = service
                .execute_task_internal(task_id, description, execution_mode, context)
                .await
            {
                error!("Task execution failed for {}: {:?}", task_id, e);

                // Update task state with error
                let mut tasks = service.active_tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.status = TaskStatus::Failed;
                    task.error_message = Some(format!("{:?}", e));
                    task.completed_at = Some(Utc::now());
                    task.updated_at = Utc::now();
                }
            }
        });

        Ok(task_id)
    }

    /// Internal task execution
    async fn execute_task_internal(
        &self,
        task_id: Uuid,
        description: String,
        execution_mode: Option<String>,
        _context: Option<String>,
    ) -> Result<()> {
        // Update status to planning
        {
            let mut tasks = self.active_tasks.write().await;
            if let Some(task) = tasks.get_mut(&task_id) {
                task.status = TaskStatus::Planning;
                task.updated_at = Utc::now();

                // Record chain of thought
                task.chain_of_thought.push(ChainOfThoughtEntry {
                    timestamp: Utc::now(),
                    phase: "planning".to_string(),
                    reasoning: format!("Analyzing task: {}", description),
                    decision: "Starting task planning phase".to_string(),
                    context: serde_json::json!({ "description": description }),
                });
            }
        }

        // Create task descriptor
        use agent_agency_contracts::planning_io::ChangeBudget;
        use agent_agency_contracts::task_request::ScopeRestrictions;

        let task_descriptor = TaskDescriptor {
            task_id,
            description: description.clone(),
            change_budget: ChangeBudget {
                max_files: 25,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            priority: TaskPriority::Medium,
            execution_mode: execution_mode
                .as_deref()
                .and_then(|m| match m {
                    "strict" => Some(ExecutionMode::Strict),
                    "auto" => Some(ExecutionMode::Auto),
                    "dry-run" => Some(ExecutionMode::DryRun),
                    _ => Some(ExecutionMode::Auto),
                })
                .unwrap_or(ExecutionMode::Auto),
            risk_tier: Some(RiskTier::Tier2),
            blast_radius: BlastRadius {
                modules: vec![],
                data_migration: false,
                external_deps: vec![],
            },
            scope_in: ScopeRestrictions {
                allowed_paths: vec![],
                blocked_paths: vec![],
            },
            scope_out: None,
            acceptance: None,
        };

        // Execute using task executor if available
        if let Some(ref executor) = self.task_executor {
            info!("Using task executor for task {}", task_id);

            // Update status to executing
            {
                let mut tasks = self.active_tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.status = TaskStatus::Executing;
                    task.updated_at = Utc::now();

                    // Record chain of thought
                    task.chain_of_thought.push(ChainOfThoughtEntry {
                        timestamp: Utc::now(),
                        phase: "execution".to_string(),
                        reasoning: "Starting task execution with executor".to_string(),
                        decision: "Delegating to task executor".to_string(),
                        context: serde_json::json!({ "executor": "available" }),
                    });
                }
            }

            match executor.execute_task(&task_descriptor).await {
                Ok(artifacts) => {
                    info!("Task {} completed successfully", task_id);

                    // Update task state with results
                    let mut tasks = self.active_tasks.write().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Completed;
                        task.artifacts = Some(artifacts.clone());
                        // Note: working_spec would be retrieved separately by working_spec_id if needed
                        task.completed_at = Some(Utc::now());
                        task.updated_at = Utc::now();

                        // Record completion in chain of thought
                        task.chain_of_thought.push(ChainOfThoughtEntry {
                            timestamp: Utc::now(),
                            phase: "completion".to_string(),
                            reasoning: "Task execution completed successfully".to_string(),
                            decision: "Task marked as completed".to_string(),
                            context: serde_json::json!({
                                "working_spec_id": artifacts.working_spec_id,
                                "iteration": artifacts.iteration,
                            }),
                        });
                    }

                    Ok(())
                }
                Err(e) => {
                    error!("Task execution failed: {}", e);

                    // Update task state with error
                    let mut tasks = self.active_tasks.write().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Failed;
                        task.error_message = Some(format!("Execution failed: {}", e));
                        task.completed_at = Some(Utc::now());
                        task.updated_at = Utc::now();

                        // Record error in chain of thought
                        task.chain_of_thought.push(ChainOfThoughtEntry {
                            timestamp: Utc::now(),
                            phase: "error".to_string(),
                            reasoning: format!("Task execution failed: {}", e),
                            decision: "Task marked as failed".to_string(),
                            context: serde_json::json!({ "error": e.to_string() }),
                        });
                    }

                    Err(anyhow::anyhow!("Task execution failed: {}", e))
                }
            }
        } else {
            warn!(
                "Task executor not available - task {} will be queued",
                task_id
            );

            // Queue task for later execution when executor becomes available
            {
                let mut queue = self.pending_task_queue.write().await;
                
                // Check queue size limits
                if queue.len() >= self.max_queue_size {
                    warn!("Task queue is full ({} tasks), rejecting task {}", self.max_queue_size, task_id);
                    
                    // Update task state with error
                    let mut tasks = self.active_tasks.write().await;
                    if let Some(task) = tasks.get_mut(&task_id) {
                        task.status = TaskStatus::Failed;
                        task.error_message = Some(format!("Task queue is full (max size: {})", self.max_queue_size));
                        task.updated_at = Utc::now();
                        
                        // Record in chain of thought
                        task.chain_of_thought.push(ChainOfThoughtEntry {
                            timestamp: Utc::now(),
                            phase: "error".to_string(),
                            reasoning: "Task queue is full, cannot queue task".to_string(),
                            decision: "Task rejected due to queue overflow".to_string(),
                            context: serde_json::json!({ 
                                "queue_size": queue.len(),
                                "max_queue_size": self.max_queue_size 
                            }),
                        });
                    }
                    
                    return Err(anyhow::anyhow!("Task queue is full, cannot queue task"));
                }
                
                // Determine task priority (higher number = higher priority)
                // Default to medium priority (0), can be enhanced with risk tier or other factors
                let priority = task_descriptor.priority as i32;
                
                // Add task to queue (sorted by priority, then by queued_at)
                let queued_task = QueuedTask {
                    task_id,
                    task_descriptor: task_descriptor.clone(),
                    queued_at: Utc::now(),
                    priority,
                };
                
                queue.push(queued_task);
                
                // Sort queue by priority (highest first), then by queued_at (oldest first)
                queue.sort_by(|a, b| {
                    b.priority.cmp(&a.priority) // Higher priority first
                        .then_with(|| a.queued_at.cmp(&b.queued_at)) // Older tasks first for same priority
                });
                
                info!("Task {} queued (queue size: {})", task_id, queue.len());
            }
            
            // Update task state to pending
            {
                let mut tasks = self.active_tasks.write().await;
                if let Some(task) = tasks.get_mut(&task_id) {
                    task.status = TaskStatus::Pending;
                    task.error_message = Some("Task executor not yet initialized. Task queued and will be executed when executor becomes available.".to_string());
                    task.updated_at = Utc::now();

                    // Record in chain of thought
                    task.chain_of_thought.push(ChainOfThoughtEntry {
                        timestamp: Utc::now(),
                        phase: "pending".to_string(),
                        reasoning: "Task executor not available".to_string(),
                        decision: "Task queued until executor is available".to_string(),
                        context: serde_json::json!({ 
                            "status": "waiting_for_executor",
                            "queued": true 
                        }),
                    });
                }
            }

            // Return success - task is queued and will be processed when executor is available
            Ok(())
        }
    }

    /// Get task status (observational only)
    ///
    /// **OBSERVATIONAL API**: This method only observes task state.
    /// It never manipulates or changes the execution state.
    pub async fn get_task_status(&self, task_id: Uuid) -> Result<Option<TaskExecutionState>> {
        let tasks = self.active_tasks.read().await;
        Ok(tasks.get(&task_id).cloned())
    }

    /// Get chain of thought for a task
    pub async fn get_chain_of_thought(&self, task_id: Uuid) -> Result<Vec<ChainOfThoughtEntry>> {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&task_id) {
            Ok(task.chain_of_thought.clone())
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }

    /// Get council decisions for a task
    pub async fn get_council_decisions(&self, task_id: Uuid) -> Result<Vec<CouncilDecision>> {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&task_id) {
            Ok(task.council_decisions.clone())
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }

    /// Get worker actions for a task
    pub async fn get_worker_actions(&self, task_id: Uuid) -> Result<Vec<WorkerAction>> {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&task_id) {
            Ok(task.worker_actions.clone())
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }

    /// Request pause of a task (orchestrator decides if it can pause)
    ///
    /// **OBSERVATIONAL API**: This is a request, not a direct control.
    /// The orchestrator maintains execution integrity and decides whether to honor the request.
    /// The request is logged in chain-of-thought for auditability.
    pub async fn request_pause_task(&self, task_id: Uuid) -> Result<()> {
        // Record the pause request in chain of thought
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.chain_of_thought.push(ChainOfThoughtEntry {
                timestamp: Utc::now(),
                phase: "pause_request".to_string(),
                reasoning: "API requested task pause".to_string(),
                decision:
                    "Request forwarded to orchestrator - orchestrator will decide if pause is safe"
                        .to_string(),
                context: serde_json::json!({ "requested_by": "api" }),
            });
            task.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
        // Note: Actual pause is handled by orchestrator's own execution management
        // We only observe the result, not control it directly
    }

    /// Request resume of a paused task (orchestrator decides if it can resume)
    ///
    /// **OBSERVATIONAL API**: This is a request, not a direct control.
    /// The orchestrator maintains execution integrity and decides whether to honor the request.
    /// The request is logged in chain-of-thought for auditability.
    pub async fn request_resume_task(&self, task_id: Uuid) -> Result<()> {
        // Record the resume request in chain of thought
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.chain_of_thought.push(ChainOfThoughtEntry {
                timestamp: Utc::now(),
                phase: "resume_request".to_string(),
                reasoning: "API requested task resume".to_string(),
                decision:
                    "Request forwarded to orchestrator - orchestrator will decide if resume is safe"
                        .to_string(),
                context: serde_json::json!({ "requested_by": "api" }),
            });
            task.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
        // Note: Actual resume is handled by orchestrator's own execution management
    }

    /// Request cancellation of a task (orchestrator decides if it can cancel safely)
    ///
    /// **OBSERVATIONAL API**: This is a request, not a direct control.
    /// The orchestrator maintains execution integrity and decides whether to honor the request.
    /// The request is logged in chain-of-thought for auditability.
    /// We only observe the result to maintain research integrity.
    pub async fn request_cancel_task(&self, task_id: Uuid) -> Result<()> {
        // Record the cancel request in chain of thought
        let mut tasks = self.active_tasks.write().await;
        if let Some(task) = tasks.get_mut(&task_id) {
            task.chain_of_thought.push(ChainOfThoughtEntry {
                timestamp: Utc::now(),
                phase: "cancel_request".to_string(),
                reasoning: "API requested task cancellation".to_string(),
                decision: "Request forwarded to orchestrator - orchestrator will decide if cancellation is safe".to_string(),
                context: serde_json::json!({ "requested_by": "api" }),
            });
            task.updated_at = Utc::now();
            Ok(())
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
        // Note: Actual cancellation is handled by orchestrator's own execution management
        // We only observe the result to maintain research integrity
    }

    /// List all active tasks
    pub async fn list_tasks(&self) -> Vec<TaskExecutionState> {
        let tasks = self.active_tasks.read().await;
        tasks.values().cloned().collect()
    }

    /// List task summaries (lightweight, for listing endpoints)
    /// Returns only essential fields without cloning large vectors
    pub async fn list_task_summaries(&self) -> Vec<TaskSummary> {
        let tasks = self.active_tasks.read().await;
        tasks
            .values()
            .map(|task| TaskSummary {
                task_id: task.task_id,
                description: task.description.clone(),
                status: task.status.clone(),
                started_at: task.started_at,
                updated_at: task.updated_at,
            })
            .collect()
    }

    /// Get task progress summary (observational only)
    pub async fn get_task_progress(&self, task_id: Uuid) -> Result<JsonValue> {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&task_id) {
            Ok(serde_json::json!({
                "task_id": task.task_id.to_string(),
                "status": format!("{:?}", task.status),
                "progress_percentage": match task.status {
                    TaskStatus::Pending => 0.0,
                    TaskStatus::Planning => 10.0,
                    TaskStatus::Executing => 50.0,
                    TaskStatus::QualityCheck => 80.0,
                    TaskStatus::Refining => 90.0,
                    TaskStatus::Completed => 100.0,
                    TaskStatus::Failed => 0.0,
                    TaskStatus::Cancelled => 0.0,
                    TaskStatus::Paused => {
                        // Estimate based on chain of thought length
                        (task.chain_of_thought.len() as f64 * 10.0).min(90.0)
                    }
                },
                "started_at": task.started_at.to_rfc3339(),
                "updated_at": task.updated_at.to_rfc3339(),
                "completed_at": task.completed_at.map(|d| d.to_rfc3339()),
                "chain_of_thought_entries": task.chain_of_thought.len(),
                "council_decisions": task.council_decisions.len(),
                "worker_actions": task.worker_actions.len(),
            }))
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }

    /// Get task logs (observational - aggregated from chain of thought)
    pub async fn get_task_logs(&self, task_id: Uuid) -> Result<Vec<JsonValue>> {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&task_id) {
            let logs: Vec<JsonValue> = task
                .chain_of_thought
                .iter()
                .map(|entry| {
                    serde_json::json!({
                        "timestamp": entry.timestamp.to_rfc3339(),
                        "level": "info",
                        "phase": entry.phase,
                        "message": format!("{} - {}", entry.reasoning, entry.decision),
                        "context": entry.context,
                    })
                })
                .collect();
            Ok(logs)
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }

    /// Get task events (observational - all events from chain of thought, council, workers)
    pub async fn get_task_events(&self, task_id: Uuid) -> Result<Vec<JsonValue>> {
        let tasks = self.active_tasks.read().await;
        if let Some(task) = tasks.get(&task_id) {
            let mut events = Vec::new();

            // Add chain of thought events
            for entry in &task.chain_of_thought {
                events.push(serde_json::json!({
                    "timestamp": entry.timestamp.to_rfc3339(),
                    "type": "chain_of_thought",
                    "phase": entry.phase,
                    "reasoning": entry.reasoning,
                    "decision": entry.decision,
                    "context": entry.context,
                }));
            }

            // Add council decision events
            for decision in &task.council_decisions {
                events.push(serde_json::json!({
                    "timestamp": decision.timestamp.to_rfc3339(),
                    "type": "council_decision",
                    "judge": decision.judge,
                    "verdict": decision.verdict,
                    "reasoning": decision.reasoning,
                    "confidence": decision.confidence,
                }));
            }

            // Add worker action events
            for action in &task.worker_actions {
                events.push(serde_json::json!({
                    "timestamp": action.timestamp.to_rfc3339(),
                    "type": "worker_action",
                    "worker_id": action.worker_id.to_string(),
                    "action": action.action,
                    "result": action.result,
                    "artifacts": action.artifacts,
                }));
            }

            // Sort by timestamp
            events.sort_by_key(|e| e["timestamp"].as_str().unwrap_or("").to_string());

            Ok(events)
        } else {
            Err(anyhow::anyhow!("Task {} not found", task_id))
        }
    }

    /// Get analytics summary (observational - aggregated from all tasks)
    pub async fn get_task_analytics(&self) -> JsonValue {
        let tasks = self.active_tasks.read().await;
        let task_list: Vec<_> = tasks.values().collect();

        let total_tasks = task_list.len();
        let completed = task_list
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Completed))
            .count();
        let failed = task_list
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Failed))
            .count();
        let in_progress = task_list
            .iter()
            .filter(|t| {
                matches!(
                    t.status,
                    TaskStatus::Planning
                        | TaskStatus::Executing
                        | TaskStatus::QualityCheck
                        | TaskStatus::Refining
                )
            })
            .count();
        let paused = task_list
            .iter()
            .filter(|t| matches!(t.status, TaskStatus::Paused))
            .count();

        let success_rate = if total_tasks > 0 {
            (completed as f64 / total_tasks as f64) * 100.0
        } else {
            0.0
        };

        serde_json::json!({
            "total_tasks": total_tasks,
            "completed": completed,
            "failed": failed,
            "in_progress": in_progress,
            "paused": paused,
            "success_rate": format!("{:.2}%", success_rate),
            "average_chain_of_thought_entries": if total_tasks > 0 {
                task_list.iter().map(|t| t.chain_of_thought.len()).sum::<usize>() as f64 / total_tasks as f64
            } else {
                0.0
            },
            "average_council_decisions": if total_tasks > 0 {
                task_list.iter().map(|t| t.council_decisions.len()).sum::<usize>() as f64 / total_tasks as f64
            } else {
                0.0
            },
            "average_worker_actions": if total_tasks > 0 {
                task_list.iter().map(|t| t.worker_actions.len()).sum::<usize>() as f64 / total_tasks as f64
            } else {
                0.0
            },
        })
    }
}
