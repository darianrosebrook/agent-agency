//! Worker pool types and data structures

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Worker types in the pool
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerType {
    Generalist,
    Specialist(String), // Specialization area
}

/// Worker status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkerStatus {
    Available,
    Busy,
    Unavailable,
    Maintenance,
    Error(String),
}

/// Worker in the pool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Worker {
    pub id: Uuid,
    pub name: String,
    pub worker_type: WorkerType,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: WorkerCapabilities,
    pub status: WorkerStatus,
    pub performance_metrics: WorkerPerformanceMetrics,
    pub created_at: DateTime<Utc>,
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerCapabilities {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub domains: Vec<String>,
    pub max_context_length: u32,
    pub max_output_length: u32,
    pub supported_formats: Vec<String>,
    pub caws_awareness: f32, // 0.0 to 1.0
    pub quality_score: f32, // 0.0 to 1.0
    pub speed_score: f32, // 0.0 to 1.0
}

/// Worker performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceMetrics {
    pub total_tasks: u64,
    pub completed_tasks: u64,
    pub failed_tasks: u64,
    pub average_execution_time_ms: f64,
    pub average_quality_score: f32,
    pub average_caws_compliance: f32,
    pub uptime_percentage: f32,
    pub last_task_at: Option<DateTime<Utc>>,
    pub current_load: f32, // 0.0 to 1.0
}

/// Task assignment to worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskAssignment {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub assigned_at: DateTime<Utc>,
    pub estimated_completion: DateTime<Utc>,
    pub priority: TaskPriority,
    pub requirements: TaskRequirements,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TaskPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Task requirements for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRequirements {
    pub required_languages: Vec<String>,
    pub required_frameworks: Vec<String>,
    pub required_domains: Vec<String>,
    pub min_quality_score: f32,
    pub min_caws_awareness: f32,
    pub max_execution_time_ms: Option<u64>,
    pub preferred_worker_type: Option<WorkerType>,
    pub context_length_estimate: u32,
}

/// Task execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskExecutionResult {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub status: ExecutionStatus,
    pub output: Option<WorkerOutput>,
    pub error_message: Option<String>,
    pub execution_time_ms: u64,
    pub tokens_used: Option<u32>,
    pub quality_metrics: QualityMetrics,
    pub caws_compliance: CawsComplianceResult,
    pub started_at: DateTime<Utc>,
    pub completed_at: DateTime<Utc>,
}

/// Execution status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionStatus {
    Completed,
    Failed,
    Timeout,
    Cancelled,
    Partial, // Partially completed but needs more work
}

/// Worker output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerOutput {
    pub content: String,
    pub files_modified: Vec<FileModification>,
    pub rationale: String,
    pub self_assessment: SelfAssessment,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// File modification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModification {
    pub path: String,
    pub operation: FileOperation,
    pub content: Option<String>,
    pub diff: Option<String>,
    pub size_bytes: u64,
}

/// File operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileOperation {
    Create,
    Modify,
    Delete,
    Move { from: String, to: String },
}

/// Self-assessment by worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfAssessment {
    pub caws_compliance: f32,
    pub quality_score: f32,
    pub confidence: f32,
    pub concerns: Vec<String>,
    pub improvements: Vec<String>,
    pub estimated_effort: Option<String>,
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityMetrics {
    pub completeness_score: f32,
    pub correctness_score: f32,
    pub maintainability_score: f32,
    pub readability_score: f32,
    pub test_coverage: Option<f32>,
    pub performance_impact: Option<f32>,
}

/// CAWS compliance result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsComplianceResult {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<CawsViolation>,
    pub budget_adherence: BudgetAdherence,
    pub provenance_complete: bool,
}

/// CAWS violation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsViolation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
}

/// Violation severity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Budget adherence tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetAdherence {
    pub files_used: u32,
    pub files_limit: u32,
    pub loc_used: u32,
    pub loc_limit: u32,
    pub time_used_ms: u64,
    pub time_limit_ms: Option<u64>,
    pub within_budget: bool,
}

/// Worker pool statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPoolStats {
    pub total_workers: u32,
    pub available_workers: u32,
    pub busy_workers: u32,
    pub unavailable_workers: u32,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub average_quality_score: f32,
    pub average_caws_compliance: f32,
    pub pool_uptime_seconds: u64,
    pub last_updated: DateTime<Utc>,
}

/// Worker health check result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerHealthCheck {
    pub worker_id: Uuid,
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    pub checked_at: DateTime<Utc>,
}

/// Task routing result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskRoutingResult {
    pub task_id: Uuid,
    pub selected_workers: Vec<WorkerAssignment>,
    pub routing_reasoning: String,
    pub estimated_completion_time: DateTime<Utc>,
    pub confidence_score: f32,
}

/// Worker assignment with reasoning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAssignment {
    pub worker_id: Uuid,
    pub worker_name: String,
    pub capability_match_score: f32,
    pub estimated_execution_time_ms: u64,
    pub reasoning: String,
    pub load_factor: f32,
}

/// Worker pool events for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkerPoolEvent {
    WorkerRegistered { worker: Worker },
    WorkerDeregistered { worker_id: Uuid },
    WorkerStatusChanged { worker_id: Uuid, old_status: WorkerStatus, new_status: WorkerStatus },
    TaskAssigned { task_id: Uuid, worker_id: Uuid },
    TaskCompleted { task_id: Uuid, worker_id: Uuid, result: TaskExecutionResult },
    TaskFailed { task_id: Uuid, worker_id: Uuid, error: String },
    HealthCheckFailed { worker_id: Uuid, error: String },
    PerformanceThresholdExceeded { worker_id: Uuid, metric: String, value: f64, threshold: f64 },
}

/// Worker configuration for registration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerRegistration {
    pub name: String,
    pub worker_type: WorkerType,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: WorkerCapabilities,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker update request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerUpdate {
    pub capabilities: Option<WorkerCapabilities>,
    pub status: Option<WorkerStatus>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Worker {
    /// Create a new worker
    pub fn new(
        name: String,
        worker_type: WorkerType,
        model_name: String,
        endpoint: String,
        capabilities: WorkerCapabilities,
    ) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            worker_type,
            model_name,
            endpoint,
            capabilities,
            status: WorkerStatus::Available,
            performance_metrics: WorkerPerformanceMetrics::default(),
            created_at: Utc::now(),
            last_heartbeat: Utc::now(),
            metadata: HashMap::new(),
        }
    }

    /// Check if worker can handle a task
    pub fn can_handle_task(&self, requirements: &TaskRequirements) -> bool {
        // Check required languages
        for required_lang in &requirements.required_languages {
            if !self.capabilities.languages.contains(required_lang) {
                return false;
            }
        }

        // Check required frameworks
        for required_framework in &requirements.required_frameworks {
            if !self.capabilities.frameworks.contains(required_framework) {
                return false;
            }
        }

        // Check required domains
        for required_domain in &requirements.required_domains {
            if !self.capabilities.domains.contains(required_domain) {
                return false;
            }
        }

        // Check minimum scores
        if self.capabilities.quality_score < requirements.min_quality_score {
            return false;
        }

        if self.capabilities.caws_awareness < requirements.min_caws_awareness {
            return false;
        }

        // Check context length
        if requirements.context_length_estimate > self.capabilities.max_context_length {
            return false;
        }

        // Check worker type preference
        if let Some(preferred_type) = &requirements.preferred_worker_type {
            if &self.worker_type != preferred_type {
                return false;
            }
        }

        // Check if worker is available
        matches!(self.status, WorkerStatus::Available)
    }

    /// Calculate capability match score for a task
    pub fn calculate_capability_score(&self, requirements: &TaskRequirements) -> f32 {
        let mut score = 0.0;
        let mut factors = 0.0;

        // Language matching (30% weight)
        let language_score = if requirements.required_languages.is_empty() {
            1.0
        } else {
            let matches = requirements.required_languages.iter()
                .filter(|lang| self.capabilities.languages.contains(lang))
                .count();
            matches as f32 / requirements.required_languages.len() as f32
        };
        score += language_score * 0.3;
        factors += 0.3;

        // Framework matching (25% weight)
        let framework_score = if requirements.required_frameworks.is_empty() {
            1.0
        } else {
            let matches = requirements.required_frameworks.iter()
                .filter(|framework| self.capabilities.frameworks.contains(framework))
                .count();
            matches as f32 / requirements.required_frameworks.len() as f32
        };
        score += framework_score * 0.25;
        factors += 0.25;

        // Domain matching (20% weight)
        let domain_score = if requirements.required_domains.is_empty() {
            1.0
        } else {
            let matches = requirements.required_domains.iter()
                .filter(|domain| self.capabilities.domains.contains(domain))
                .count();
            matches as f32 / requirements.required_domains.len() as f32
        };
        score += domain_score * 0.2;
        factors += 0.2;

        // Quality score (15% weight)
        score += self.capabilities.quality_score * 0.15;
        factors += 0.15;

        // CAWS awareness (10% weight)
        score += self.capabilities.caws_awareness * 0.1;
        factors += 0.1;

        if factors > 0.0 {
            score / factors
        } else {
            0.0
        }
    }

    /// Update performance metrics after task completion
    pub fn update_performance_metrics(&mut self, result: &TaskExecutionResult) {
        self.performance_metrics.total_tasks += 1;

        match result.status {
            ExecutionStatus::Completed => {
                self.performance_metrics.completed_tasks += 1;
            }
            ExecutionStatus::Failed | ExecutionStatus::Timeout | ExecutionStatus::Cancelled => {
                self.performance_metrics.failed_tasks += 1;
            }
            ExecutionStatus::Partial => {
                self.performance_metrics.completed_tasks += 1; // Count as completed
            }
        }

        // Update average execution time
        let total_time = self.performance_metrics.average_execution_time_ms * (self.performance_metrics.total_tasks - 1) as f64;
        self.performance_metrics.average_execution_time_ms = (total_time + result.execution_time_ms as f64) / self.performance_metrics.total_tasks as f64;

        // Update average quality score
        if let Some(output) = &result.output {
            let total_quality = self.performance_metrics.average_quality_score * (self.performance_metrics.total_tasks - 1) as f32;
            self.performance_metrics.average_quality_score = (total_quality + output.self_assessment.quality_score) / self.performance_metrics.total_tasks as f32;
        }

        // Update average CAWS compliance
        let total_compliance = self.performance_metrics.average_caws_compliance * (self.performance_metrics.total_tasks - 1) as f32;
        self.performance_metrics.average_caws_compliance = (total_compliance + result.caws_compliance.compliance_score) / self.performance_metrics.total_tasks as f32;

        self.performance_metrics.last_task_at = Some(result.completed_at);
    }
}

impl Default for WorkerPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            average_execution_time_ms: 0.0,
            average_quality_score: 0.0,
            average_caws_compliance: 0.0,
            uptime_percentage: 100.0,
            last_task_at: None,
            current_load: 0.0,
        }
    }
}

impl Default for QualityMetrics {
    fn default() -> Self {
        Self {
            completeness_score: 0.0,
            correctness_score: 0.0,
            maintainability_score: 0.0,
            readability_score: 0.0,
            test_coverage: None,
            performance_impact: None,
        }
    }
}

impl Default for CawsComplianceResult {
    fn default() -> Self {
        Self {
            is_compliant: true,
            compliance_score: 1.0,
            violations: vec![],
            budget_adherence: BudgetAdherence::default(),
            provenance_complete: true,
        }
    }
}

impl Default for BudgetAdherence {
    fn default() -> Self {
        Self {
            files_used: 0,
            files_limit: 0,
            loc_used: 0,
            loc_limit: 0,
            time_used_ms: 0,
            time_limit_ms: None,
            within_budget: true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worker_creation() {
        let capabilities = WorkerCapabilities {
            languages: vec!["rust".to_string(), "typescript".to_string()],
            frameworks: vec!["tokio".to_string()],
            domains: vec!["backend".to_string()],
            max_context_length: 8000,
            max_output_length: 4000,
            supported_formats: vec!["json".to_string()],
            caws_awareness: 0.9,
            quality_score: 0.85,
            speed_score: 0.8,
        };

        let worker = Worker::new(
            "test-worker".to_string(),
            WorkerType::Generalist,
            "llama3.3:7b".to_string(),
            "http://localhost:11434".to_string(),
            capabilities,
        );

        assert_eq!(worker.name, "test-worker");
        assert_eq!(worker.status, WorkerStatus::Available);
        assert_eq!(worker.performance_metrics.total_tasks, 0);
    }

    #[test]
    fn test_worker_capability_matching() {
        let capabilities = WorkerCapabilities {
            languages: vec!["rust".to_string(), "typescript".to_string()],
            frameworks: vec!["tokio".to_string(), "react".to_string()],
            domains: vec!["backend".to_string(), "frontend".to_string()],
            max_context_length: 8000,
            max_output_length: 4000,
            supported_formats: vec!["json".to_string()],
            caws_awareness: 0.9,
            quality_score: 0.85,
            speed_score: 0.8,
        };

        let worker = Worker::new(
            "test-worker".to_string(),
            WorkerType::Generalist,
            "llama3.3:7b".to_string(),
            "http://localhost:11434".to_string(),
            capabilities,
        );

        let requirements = TaskRequirements {
            required_languages: vec!["rust".to_string()],
            required_frameworks: vec!["tokio".to_string()],
            required_domains: vec!["backend".to_string()],
            min_quality_score: 0.8,
            min_caws_awareness: 0.8,
            max_execution_time_ms: Some(30000),
            preferred_worker_type: Some(WorkerType::Generalist),
            context_length_estimate: 4000,
        };

        assert!(worker.can_handle_task(&requirements));
        
        let capability_score = worker.calculate_capability_score(&requirements);
        assert!(capability_score > 0.8); // Should be high for good match
    }

    #[test]
    fn test_performance_metrics_update() {
        let mut worker = Worker::new(
            "test-worker".to_string(),
            WorkerType::Generalist,
            "llama3.3:7b".to_string(),
            "http://localhost:11434".to_string(),
            WorkerCapabilities::default(),
        );

        let result = TaskExecutionResult {
            task_id: Uuid::new_v4(),
            worker_id: worker.id,
            status: ExecutionStatus::Completed,
            output: Some(WorkerOutput {
                content: "test output".to_string(),
                files_modified: vec![],
                rationale: "test rationale".to_string(),
                self_assessment: SelfAssessment {
                    caws_compliance: 0.9,
                    quality_score: 0.85,
                    confidence: 0.8,
                    concerns: vec![],
                    improvements: vec![],
                    estimated_effort: None,
                },
                metadata: HashMap::new(),
            }),
            error_message: None,
            execution_time_ms: 2000,
            tokens_used: Some(1500),
            quality_metrics: QualityMetrics::default(),
            caws_compliance: CawsComplianceResult {
                compliance_score: 0.9,
                ..Default::default()
            },
            started_at: Utc::now(),
            completed_at: Utc::now(),
        };

        worker.update_performance_metrics(&result);

        assert_eq!(worker.performance_metrics.total_tasks, 1);
        assert_eq!(worker.performance_metrics.completed_tasks, 1);
        assert_eq!(worker.performance_metrics.average_execution_time_ms, 2000.0);
        assert_eq!(worker.performance_metrics.average_quality_score, 0.85);
        assert_eq!(worker.performance_metrics.average_caws_compliance, 0.9);
    }
}

impl Default for WorkerCapabilities {
    fn default() -> Self {
        Self {
            languages: vec![],
            frameworks: vec![],
            domains: vec![],
            max_context_length: 4000,
            max_output_length: 2000,
            supported_formats: vec!["text".to_string()],
            caws_awareness: 0.5,
            quality_score: 0.5,
            speed_score: 0.5,
        }
    }
}

/// Task execution context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub start_time: DateTime<Utc>,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: HashMap<String, serde_json::Value>,
}
