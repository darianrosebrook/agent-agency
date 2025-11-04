//! Integration layer for autonomous agent systems
//!
//! Coordinates between multiple autonomous agents and external systems.

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use std::sync::Arc;
use tokio::sync::RwLock;
use async_trait::async_trait;

use crate::self_prompting_agent::prompting_types::{Task, SelfPromptingAgentError};
#[cfg(feature = "workers")]
use agent_workers::decomposition::{DecompositionEngine, TaskAnalysis, SubTask};
#[cfg(feature = "workers")]
use agent_workers::parallel_types::{ComplexTask, TaskId, TaskScope, Priority, QualityRequirements};
use chrono::Utc;
use system_observability::health_metrics::MetricsCollector;

// TODO: Implement real agent health metrics collection
// - [ ] Integrate with metrics collection system
// - [ ] Track agent health score from multiple indicators
// - [ ] Calculate success rate from execution history
// - [ ] Monitor agent performance over time
// - [ ] Add unit tests with mock metrics
// - [ ] Add integration tests with real health monitoring
/// Agent health metrics placeholder

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AgentHealthMetrics {
    health_score: f64,
    success_rate: f64,
    current_load: usize,
    max_load: usize,
    response_time_p95: u64,
}

/// Integrated autonomous agent coordinator
pub struct IntegratedAutonomousAgent {
    agents: Vec<Arc<dyn AutonomousAgent>>,
    state: Arc<RwLock<IntegrationState>>,
    performance_tracker: Option<Arc<MetricsCollector>>,
}

impl IntegratedAutonomousAgent {
    /// Create a new integrated agent
    pub fn new() -> Self {
        Self {
            agents: Vec::new(),
            state: Arc::new(RwLock::new(IntegrationState::default())),
            performance_tracker: None,
        }
    }
    
    /// Create with performance tracking
    pub fn with_performance_tracker(performance_tracker: Arc<MetricsCollector>) -> Self {
        Self {
            agents: Vec::new(),
            state: Arc::new(RwLock::new(IntegrationState::default())),
            performance_tracker: Some(performance_tracker),
        }
    }

    /// Register an autonomous agent
    pub fn register_agent(&mut self, agent: Arc<dyn AutonomousAgent>) {
        self.agents.push(agent);
    }

    /// Execute a task using integrated agents
    pub async fn execute_task(&self, task: Task) -> Result<TaskResult, SelfPromptingAgentError> {
        let mut state = self.state.write().await;

        // Select appropriate agent for the task
        let agent = self.select_agent(&task).await?;
        let agent_name = agent.name().to_string();
        state.active_agent = Some(agent_name.clone());

        // Record task start for performance tracking
        let start_time = std::time::Instant::now();

        // Execute with the selected agent
        let prompt_result = agent.execute_task(task.clone()).await
            .map_err(|e| SelfPromptingAgentError::Execution(format!("Agent execution failed: {}", e)))?;

        let execution_time_ms = start_time.elapsed().as_millis() as u64;
        let success = prompt_result.final_report.status == crate::self_prompting_agent::prompting_types::EvalStatus::Pass;

        // Record performance metrics
        if let Some(tracker) = &self.performance_tracker {
            if let Err(e) = tracker.record_agent_task(&agent_name, success, execution_time_ms).await {
                tracing::warn!("Failed to record agent performance: {}", e);
            }
        }

        state.completed_tasks += 1;
        state.last_task = Some(task.id.to_string());

        // Convert prompt_result to integration layer TaskResult
        Ok(TaskResult {
            task_id: prompt_result.task_id,
            agent_name,
            result: serde_json::json!({
                "status": if success { "success" } else { "failed" },
                "score": prompt_result.final_report.score,
                "execution_time_ms": execution_time_ms,
                "artifacts": prompt_result.artifacts.iter().map(|a| a.file_path.clone()).collect::<Vec<_>>(),
            }),
            execution_time_ms,
            artifacts: prompt_result.artifacts.iter().map(|a| a.file_path.clone()).collect(),
        })
    }

    /// Get integration status
    pub async fn status(&self) -> IntegrationStatus {
        let state = self.state.read().await;

        IntegrationStatus {
            registered_agents: self.agents.len(),
            active_agent: state.active_agent.clone(),
            completed_tasks: state.completed_tasks,
            failed_tasks: state.failed_tasks,
            last_task: state.last_task.clone(),
        }
    }

    /// Select the best agent for a task based on capabilities, performance, and load
    async fn select_agent(&self, task: &Task) -> Result<Arc<dyn AutonomousAgent>, SelfPromptingAgentError> {
        if self.agents.is_empty() {
            return Err(SelfPromptingAgentError::Execution("No agents registered".to_string()));
        }

        // Find agents that can handle this task
        let capable_agents: Vec<_> = self.agents.iter()
            .filter(|agent| agent.can_handle(task))
            .collect();

        if capable_agents.is_empty() {
            // No agent explicitly can handle it, fall back to first available agent
            tracing::warn!("No agent explicitly capable of handling task {}, using first available agent", task.id);
            return self.agents.first().cloned()
                .ok_or_else(|| SelfPromptingAgentError::Execution("No agents available".to_string()));
        }

        // If multiple agents can handle it, select based on:
        // 1. Performance history (success rate, response time)
        // 2. Current load (prefer agents with lower load)
        // 3. Health score (prefer healthier agents)
        // 4. Capability match (fallback to first capable if no metrics available)
        
        let selected = if let Some(tracker) = &self.performance_tracker {
            // Use performance-based selection
            self.select_agent_with_metrics(capable_agents, tracker).await?
        } else {
            // No performance tracker, use first capable agent
            capable_agents.first().unwrap().clone()
        };
        
        tracing::debug!(
            task_id = %task.id,
            agent_name = %selected.name(),
            capabilities = ?selected.capabilities(),
            "Selected agent for task"
        );

        Ok(selected)
    }
    
    /// Select agent using performance metrics and load balancing
    async fn select_agent_with_metrics(
        &self,
        agents: Vec<&Arc<dyn AutonomousAgent>>,
        tracker: &MetricsCollector,
    ) -> Result<Arc<dyn AutonomousAgent>, SelfPromptingAgentError> {
        let mut scored_agents: Vec<(Arc<dyn AutonomousAgent>, f64)> = Vec::new();
        
        for agent in agents {
            let agent_name = agent.name();
            // Get agent health metrics - using basic implementation
            // TODO: Integrate with system-observability crate for real metrics
            let metrics = Some(self.get_agent_health_metrics(agent_name).await);
            
            let score = if let Some(metrics) = metrics {
                // Calculate selection score based on multiple factors
                // Higher score = better choice
                let mut score = 0.0;
                
                // Health score (0-1, higher is better)
                score += metrics.health_score * 0.4;
                
                // Success rate (0-1, higher is better)
                score += metrics.success_rate * 0.3;
                
                // Load factor (lower load = higher score)
                let load_factor = if metrics.max_load > 0 {
                    1.0 - (metrics.current_load as f64 / metrics.max_load as f64) * 0.5
                } else {
                    0.5 // Default if max_load is 0
                };
                score += load_factor * 0.2;
                
                // Response time factor (faster = higher score, normalized)
                let response_time_factor = if metrics.response_time_p95 > 0 {
                    (1000.0 / metrics.response_time_p95 as f64).min(1.0) * 0.1
                } else {
                    0.05
                };
                score += response_time_factor;
                
                score
            } else {
                // No metrics available, use default score
                0.5
            };
            
            scored_agents.push((agent.clone(), score));
        }
        
        // Sort by score (descending) and return the best agent
        scored_agents.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        
        scored_agents.first()
            .map(|(agent, score)| {
                tracing::debug!(
                    agent_name = %agent.name(),
                    selection_score = score,
                    "Selected agent based on performance metrics"
                );
                agent.clone()
            })
            .ok_or_else(|| SelfPromptingAgentError::Execution("No agents available for selection".to_string()))
    }
}

/// Autonomous agent trait
#[async_trait]
pub trait AutonomousAgent: Send + Sync {
    /// Execute a task autonomously
    async fn execute_task(&self, task: Task) -> Result<crate::self_prompting_agent::prompting_types::TaskResult, SelfPromptingAgentError>;

    /// Get agent name
    fn name(&self) -> &str;

    /// Get agent capabilities
    fn capabilities(&self) -> Vec<String>;

    /// Check if agent can handle a task
    fn can_handle(&self, task: &Task) -> bool;
}

/// Task execution result for integration layer

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskResult {
    pub task_id: uuid::Uuid,
    pub agent_name: String,
    pub result: serde_json::Value,
    pub execution_time_ms: u64,
    pub artifacts: Vec<String>,
}

/// Integration state

#[derive(Debug, Serialize, Deserialize, JsonSchema, Default)]
pub struct IntegrationState {
    active_agent: Option<String>,
    completed_tasks: usize,
    failed_tasks: usize,
    last_task: Option<String>,
}

/// Integration status

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationStatus {
    pub registered_agents: usize,
    pub active_agent: Option<String>,
    pub completed_tasks: usize,
    pub failed_tasks: usize,
    pub last_task: Option<String>,
}

/// Multi-agent coordinator for complex tasks
#[cfg(feature = "workers")]
pub struct MultiAgentCoordinator {
    agents: Vec<Arc<dyn AutonomousAgent>>,
    decomposition_engine: DecompositionEngine,
}

#[cfg(feature = "workers")]
impl MultiAgentCoordinator {
    pub fn new() -> Self {
        Self { 
            agents: Vec::new(),
            decomposition_engine: DecompositionEngine::new(),
        }
    }

    /// Coordinate task execution across multiple agents
    #[cfg(feature = "workers")]
    pub async fn coordinate_task(&self, task: Task) -> Result<CoordinatedResult, SelfPromptingAgentError> {
        use std::time::Instant;
        let start_time = Instant::now();

        // Convert Task to ComplexTask for decomposition
        let complex_task = self.task_to_complex_task(&task)?;
        
        // Use DecompositionEngine to analyze and decompose the task
        let analysis = self.decomposition_engine.analyze(&complex_task).await
            .map_err(|e| SelfPromptingAgentError::Execution(format!("Task decomposition failed: {}", e)))?;
        
        // Create subtasks from decomposition analysis
        let subtasks = self.decomposition_engine.decompose(analysis)
            .map_err(|e| SelfPromptingAgentError::Execution(format!("Subtask creation failed: {}", e)))?;
        
        // Execute subtasks in parallel based on agent capabilities
        let mut task_results = Vec::new();
        
        for subtask in subtasks {
            // Find best agent for this subtask
            let agent = self.find_agent_for_subtask(&subtask).await?;
            
            // Convert SubTask back to Task for agent execution
            let agent_task = self.subtask_to_task(&subtask, &task)?;
            
            // Execute subtask with selected agent
            let prompt_result = agent.execute_task(agent_task).await
                .map_err(|e| SelfPromptingAgentError::Execution(format!("Subtask execution failed: {}", e)))?;
            
            // Convert to integration layer TaskResult
            let integration_result = TaskResult {
                task_id: prompt_result.task_id,
                agent_name: agent.name().to_string(),
                result: serde_json::json!({
                    "status": if prompt_result.final_report.status == crate::self_prompting_agent::prompting_types::EvalStatus::Pass { "success" } else { "failed" },
                    "score": prompt_result.final_report.score,
                    "execution_time_ms": prompt_result.execution_time_ms,
                    "artifacts": prompt_result.artifacts.iter().map(|a| a.file_path.clone()).collect::<Vec<_>>(),
                }),
                execution_time_ms: prompt_result.execution_time_ms,
                artifacts: prompt_result.artifacts.iter().map(|a| a.file_path.clone()).collect(),
            };
            
            task_results.push(integration_result);
        }
        
        let coordination_time_ms = start_time.elapsed().as_millis() as u64;
        
        // Merge results from all subtasks
        let final_result = if task_results.len() == 1 {
            serde_json::json!({
                "status": "success",
                "agent": task_results[0].agent_name.clone(),
                "execution_time_ms": task_results[0].execution_time_ms,
                "result": task_results[0].result.clone(),
            })
        } else {
            serde_json::json!({
                "status": "coordinated",
                "subtask_count": task_results.len(),
                "results": task_results.iter().map(|r| serde_json::json!({
                    "agent": r.agent_name,
                    "result": r.result,
                    "execution_time_ms": r.execution_time_ms,
                })).collect::<Vec<_>>(),
            })
        };
        
        Ok(CoordinatedResult {
            task_id: task.id,
            subtasks: task_results,
            final_result,
            coordination_time_ms,
        })
    }
    
    /// Convert Task to ComplexTask for decomposition engine
    #[cfg(feature = "workers")]
    fn task_to_complex_task(&self, task: &Task) -> Result<ComplexTask, SelfPromptingAgentError> {
        // Extract domains from task type and target files
        let domains = vec![format!("{:?}", task.task_type)];
        
        // Calculate complexity score based on task characteristics
        let complexity_score = self.calculate_complexity_score(task);
        
        // Determine priority based on constraints
        let priority = if task.constraints.contains_key("priority") {
            match task.constraints.get("priority").unwrap().as_str() {
                "critical" => Priority::Critical,
                "high" => Priority::High,
                "low" => Priority::Low,
                _ => Priority::Medium,
            }
        } else {
            Priority::Medium
        };
        
        // Create task scope from target files
        let scope = TaskScope {
            domains,
            files_affected: task.target_files.clone(),
            max_files: task.constraints.get("max_files")
                .and_then(|s| s.parse().ok()),
            max_loc: task.constraints.get("max_loc")
                .and_then(|s| s.parse().ok()),
        };
        
        // Create quality requirements from constraints
        let quality_requirements = QualityRequirements {
            min_coverage: task.constraints.get("min_coverage")
                .and_then(|s| s.parse().ok()),
            max_complexity: task.constraints.get("max_complexity")
                .and_then(|s| s.parse().ok()),
            required_tests: task.constraints.get("required_tests")
                .map(|s| s == "true")
                .unwrap_or(true),
            documentation_required: task.constraints.get("documentation_required")
                .map(|s| s == "true")
                .unwrap_or(false),
        };
        
        // Build metadata from constraints and refinement context
        let mut metadata = std::collections::HashMap::new();
        for (key, value) in &task.constraints {
            metadata.insert(key.clone(), serde_json::Value::String(value.clone()));
        }
        metadata.insert("refinement_context".to_string(), 
            serde_json::json!(task.refinement_context));
        
        Ok(ComplexTask {
            id: TaskId(task.id),
            title: format!("{:?}", task.task_type),
            description: task.description.clone(),
            complexity_score,
            priority,
            scope,
            quality_requirements,
            created_at: Utc::now(),
            deadline: None,
            metadata,
        })
    }
    
    /// Calculate complexity score for task
    fn calculate_complexity_score(&self, task: &Task) -> f64 {
        let mut score = 0.0;
        
        // Base complexity from task type
        score += match task.task_type {
            crate::self_prompting_agent::prompting_types::TaskType::CodeGeneration => 0.3,
            crate::self_prompting_agent::prompting_types::TaskType::CodeReview => 0.2,
            crate::self_prompting_agent::prompting_types::TaskType::CodeRefactor => 0.5,
            crate::self_prompting_agent::prompting_types::TaskType::Testing => 0.3,
            crate::self_prompting_agent::prompting_types::TaskType::Documentation => 0.1,
            crate::self_prompting_agent::prompting_types::TaskType::Research => 0.4,
            crate::self_prompting_agent::prompting_types::TaskType::Planning => 0.2,
        };
        
        // Add complexity for number of target files
        score += (task.target_files.len() as f64 / 10.0).min(0.3);
        
        // Add complexity for refinement context (multiple iterations)
        score += (task.refinement_context.len() as f64 / 5.0).min(0.2);
        
        score.min(1.0)
    }
    
    /// Find best agent for a subtask
    async fn find_agent_for_subtask(&self, subtask: &SubTask) -> Result<Arc<dyn AutonomousAgent>, SelfPromptingAgentError> {
        // Create a temporary Task from subtask to check capabilities
        let temp_task = Task {
            id: subtask.id.0,
            description: subtask.description.clone(),
            task_type: self.infer_task_type_from_subtask(subtask),
            target_files: subtask.scope.files_affected.clone(),
            constraints: std::collections::HashMap::new(),
            refinement_context: Vec::new(),
        };
        
        // Find agents that can handle this subtask
        let capable_agents: Vec<_> = self.agents.iter()
            .filter(|agent| agent.can_handle(&temp_task))
            .collect();
        
        if capable_agents.is_empty() {
            // Fallback to first available agent
            self.agents.first().cloned()
                .ok_or_else(|| SelfPromptingAgentError::Execution("No agents available".to_string()))
        } else {
            Ok(capable_agents.first().unwrap().clone())
        }
    }
    
    /// Infer task type from subtask description
    #[cfg(feature = "workers")]
    fn infer_task_type_from_subtask(&self, subtask: &SubTask) -> crate::self_prompting_agent::prompting_types::TaskType {
        let desc_lower = subtask.description.to_lowercase();
        
        if desc_lower.contains("test") || desc_lower.contains("test") {
            crate::self_prompting_agent::prompting_types::TaskType::Testing
        } else if desc_lower.contains("refactor") || desc_lower.contains("refactor") {
            crate::self_prompting_agent::prompting_types::TaskType::CodeRefactor
        } else if desc_lower.contains("review") || desc_lower.contains("review") {
            crate::self_prompting_agent::prompting_types::TaskType::CodeReview
        } else if desc_lower.contains("doc") || desc_lower.contains("documentation") {
            crate::self_prompting_agent::prompting_types::TaskType::Documentation
        } else if desc_lower.contains("plan") || desc_lower.contains("planning") {
            crate::self_prompting_agent::prompting_types::TaskType::Planning
        } else if desc_lower.contains("research") {
            crate::self_prompting_agent::prompting_types::TaskType::Research
        } else {
            crate::self_prompting_agent::prompting_types::TaskType::CodeGeneration
        }
    }
    
    /// Convert SubTask back to Task for agent execution
    #[cfg(feature = "workers")]
    fn subtask_to_task(&self, subtask: &SubTask, original_task: &Task) -> Result<Task, SelfPromptingAgentError> {
        Ok(Task {
            id: subtask.id.0,
            description: subtask.description.clone(),
            task_type: self.infer_task_type_from_subtask(subtask),
            target_files: subtask.scope.files_affected.clone(),
            constraints: original_task.constraints.clone(),
            refinement_context: original_task.refinement_context.clone(),
        })
    }
}

/// Coordinated execution result

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CoordinatedResult {
    pub task_id: uuid::Uuid,
    pub subtasks: Vec<TaskResult>,
    pub final_result: serde_json::Value,
    pub coordination_time_ms: u64,
}

/// Agent communication hub
pub struct AgentCommunicationHub {
    channels: std::collections::HashMap<String, tokio::sync::mpsc::UnboundedSender<Message>>,
}

impl AgentCommunicationHub {
    pub fn new() -> Self {
        Self {
            channels: std::collections::HashMap::new(),
        }
    }

    /// Send message to agent
    pub async fn send_message(&self, agent_name: &str, message: Message) -> Result<(), SelfPromptingAgentError> {
        if let Some(sender) = self.channels.get(agent_name) {
            sender.send(message).map_err(|_| SelfPromptingAgentError::Execution("Failed to send message".to_string()))?;
            Ok(())
        } else {
            Err(SelfPromptingAgentError::Execution(format!("Agent '{}' not found", agent_name)))
        }
    }

    /// Register agent channel
    pub fn register_agent(&mut self, agent_name: String, sender: tokio::sync::mpsc::UnboundedSender<Message>) {
        self.channels.insert(agent_name, sender);
    }

    /// Get agent health metrics - basic implementation
    /// TODO: Integrate with system-observability crate for real metrics
    pub async fn get_agent_health_metrics(&self, agent_name: &str) -> AgentHealthMetrics {
        // Basic implementation - return reasonable defaults
        // In a real implementation, this would query actual health metrics
        AgentHealthMetrics {
            agent_id: agent_name.to_string(),
            health_score: 0.85, // Assume generally healthy
            current_load: 50,    // Assume moderate load
            max_load: 100,
            success_rate: 0.92,  // Assume high success rate
            error_rate: 0.02,    // Low error rate
            response_time_p95: 250, // 250ms P95 response time
        }
    }
}

/// Inter-agent message

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Message {
    pub from: String,
    pub to: String,
    pub content: serde_json::Value,
    pub message_type: MessageType,
}

/// Message types

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum MessageType {
    TaskRequest,
    TaskResult,
    StatusUpdate,
    Coordination,
}
