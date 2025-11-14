//! Worker pool types and data structures

use chrono::{DateTime, Utc};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// All core types are now defined in this module

/// Task status enumeration
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Task priority levels
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

impl Default for Priority {
    fn default() -> Self {
        Self::Medium
    }
}

/// Task scope - what the task affects
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskScope {
    pub domains: Vec<String>,
    pub files_affected: Vec<String>,
    pub files: Vec<String>,
    pub directories: Vec<String>,
    pub patterns: Vec<String>,
    pub max_files: Option<usize>,
    pub max_loc: Option<usize>,
}

/// Quality requirements for task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityRequirements {
    pub min_coverage: Option<f64>,
    pub max_complexity: Option<f64>,
    pub required_tests: bool,
    pub documentation_required: bool,
}

impl Default for QualityRequirements {
    fn default() -> Self {
        Self {
            min_coverage: Some(0.8),
            max_complexity: Some(10.0),
            required_tests: true,
            documentation_required: false,
        }
    }
}

// Use shared types from contracts
use agent_agency_contracts::{
    task_executor::{ExecutionStatus, TaskExecutionResult},
    RiskTier, TaskPriority as ContractTaskPriority, WorkerHealthMetrics, WorkerHealthStatus,
    WorkerRegistration, WorkerType,
};

// Define our own TaskPriority to avoid conflicts
pub type TaskPriority = ContractTaskPriority;

/// Task ID wrapper around UUID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct TaskId(#[schemars(with = "String")] pub Uuid);

/// Subtask ID wrapper around UUID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct SubTaskId(#[schemars(with = "String")] pub Uuid);

/// Worker ID wrapper around UUID
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, JsonSchema)]
pub struct WorkerId(#[schemars(with = "String")] pub Uuid);

impl std::fmt::Display for WorkerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl TaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl SubTaskId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl WorkerId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
}

impl Default for TaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for SubTaskId {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for WorkerId {
    fn default() -> Self {
        Self::new()
    }
}

/// Clock trait for time operations
pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

/// IdGenerator trait for generating unique identifiers
pub trait IdGenerator {
    fn generate(&self) -> Uuid;
}

/// Default clock implementation using system time

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Default ID generator implementation using UUID v4

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
}

/// Worker status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WorkerStatus {
    Available,
    Busy,
    Unavailable,
    Maintenance,
    Error(String),
}

/// Worker in the pool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Worker {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub worker_type: WorkerType,
    pub model_name: String,
    pub endpoint: String,
    pub capabilities: WorkerCapabilities,
    pub status: WorkerStatus,
    pub performance_metrics: WorkerPerformanceMetrics,
    pub health_status: WorkerHealthStatus,
    pub health_metrics: Option<WorkerHealthMetrics>,
    pub last_health_check: Option<DateTime<Utc>>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub last_heartbeat: DateTime<Utc>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Worker capabilities
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerCapabilities {
    pub languages: Vec<String>,
    pub frameworks: Vec<String>,
    pub domains: Vec<String>,
    pub max_context_length: u32,
    pub max_output_length: u32,
    pub supported_formats: Vec<String>,
    pub caws_awareness: f32, // 0.0 to 1.0
    pub quality_score: f32,  // 0.0 to 1.0
    pub speed_score: f32,    // 0.0 to 1.0
}

/// Worker performance metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
    pub busy_workers: u32,
}

/// Task assignment to worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskAssignment {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    #[schemars(with = "String")]
    pub assigned_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub estimated_completion: DateTime<Utc>,
    pub priority: TaskPriority,
    pub requirements: TaskRequirements,
}

/// Task requirements for routing
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
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

/// Worker output
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerOutput {
    pub content: String,
    pub files_modified: Vec<FileModification>,
    pub rationale: String,
    pub self_assessment: SelfAssessment,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// File modification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FileModification {
    pub path: String,
    pub operation: FileOperation,
    pub content: Option<String>,
    pub diff: Option<String>,
    pub size_bytes: u64,
}

/// File operation types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum FileOperation {
    Create,
    Modify,
    Delete,
    Move { from: String, to: String },
}

/// Self-assessment by worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SelfAssessment {
    pub caws_compliance: f32,
    pub quality_score: f32,
    pub confidence: f32,
    pub concerns: Vec<String>,
    pub improvements: Vec<String>,
    pub estimated_effort: Option<String>,
}

/// Metadata keys used to enrich `TaskExecutionResult`.
pub const META_EXECUTION_STATUS: &str = "execution_status";
pub const META_QUALITY_METRICS: &str = "quality_metrics";
pub const META_CAWS_COMPLIANCE: &str = "caws_compliance";
pub const META_WORKER_OUTPUT: &str = "worker_output";
pub const META_TOKENS_USED: &str = "tokens_used";

/// Extract the execution status from a contract result, defaulting by success flag.
pub fn get_execution_status(result: &TaskExecutionResult) -> ExecutionStatus {
    if let Some(value) = result
        .metadata
        .get(META_EXECUTION_STATUS)
        .and_then(|v| v.as_str())
    {
        match value {
            "Pending" => ExecutionStatus::Pending,
            "Running" => ExecutionStatus::Running,
            "Completed" => ExecutionStatus::Completed,
            "Failed" => ExecutionStatus::Failed,
            "Cancelled" | "Canceled" => ExecutionStatus::Cancelled,
            "Timeout" => ExecutionStatus::Timeout,
            "Partial" => ExecutionStatus::Failed,
            _ => {
                if result.success {
                    ExecutionStatus::Completed
                } else {
                    ExecutionStatus::Failed
                }
            }
        }
    } else if result.success {
        ExecutionStatus::Completed
    } else {
        ExecutionStatus::Failed
    }
}

/// Attempt to deserialize worker output from result metadata or output string.
pub fn get_worker_output(result: &TaskExecutionResult) -> Option<WorkerOutput> {
    if let Some(value) = result.metadata.get(META_WORKER_OUTPUT) {
        if let Ok(parsed) = serde_json::from_value::<WorkerOutput>(value.clone()) {
            return Some(parsed);
        }
    }

    if result.output.trim().is_empty() {
        return None;
    }

    serde_json::from_str::<WorkerOutput>(&result.output).ok()
}

/// Retrieve quality metrics if present.
pub fn get_quality_metrics(result: &TaskExecutionResult) -> Option<QualityMetrics> {
    result
        .metadata
        .get(META_QUALITY_METRICS)
        .and_then(|value| serde_json::from_value::<QualityMetrics>(value.clone()).ok())
}

/// Retrieve CAWS compliance summary if present.
pub fn get_caws_compliance(result: &TaskExecutionResult) -> Option<CawsComplianceResult> {
    result
        .metadata
        .get(META_CAWS_COMPLIANCE)
        .and_then(|value| serde_json::from_value::<CawsComplianceResult>(value.clone()).ok())
}

/// Retrieve token usage information if available.
pub fn get_tokens_used(result: &TaskExecutionResult) -> Option<u32> {
    result.metadata.get(META_TOKENS_USED).and_then(|value| {
        if let Some(num) = value.as_u64() {
            Some(num as u32)
        } else {
            value.as_str().and_then(|s| s.parse::<u32>().ok())
        }
    })
}

/// Quality metrics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityMetrics {
    pub completeness_score: f32,
    pub correctness_score: f32,
    pub maintainability_score: f32,
    pub readability_score: f32,
    pub test_coverage: Option<f32>,
    pub performance_impact: Option<f32>,
}

/// CAWS compliance result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsComplianceResult {
    pub is_compliant: bool,
    pub compliance_score: f32,
    pub violations: Vec<CawsViolation>,
    pub budget_adherence: BudgetAdherence,
    pub provenance_complete: bool,
}

/// CAWS violation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsViolation {
    pub rule: String,
    pub severity: ViolationSeverity,
    pub description: String,
    pub location: Option<String>,
    pub suggestion: Option<String>,
    pub constitutional_ref: Option<String>,
}

/// Violation severity
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema)]
pub enum ViolationSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Budget adherence tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerPoolStats {
    pub total_workers: u32,
    pub available_workers: u32,
    pub busy_workers: u32,
    pub unavailable_workers: u32,
    pub active_workers: u32,
    pub idle_workers: u32,
    pub unhealthy_workers: u32,
    pub tasks_in_progress: u32,
    pub total_tasks_completed: u64,
    pub total_tasks_failed: u64,
    pub average_execution_time_ms: f64,
    pub average_quality_score: f32,
    pub average_caws_compliance: f32,
    pub average_queue_time_ms: f64,
    pub pool_uptime_seconds: u64,
    #[schemars(with = "String")]
    pub last_updated: DateTime<Utc>,
}

// WorkerHealthStatus and WorkerHealthMetrics are now imported from agent_agency_contracts

/// Worker metrics collection from /metrics endpoint

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
struct WorkerMetricsCollection {
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub active_tasks: Option<u32>,
    pub queue_depth: Option<u32>,
}

/// Worker health check result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerHealthCheck {
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    pub is_healthy: bool,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
    #[schemars(with = "String")]
    pub checked_at: DateTime<Utc>,
}

/// Task routing result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskRoutingResult {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub selected_workers: Vec<WorkerAssignmentDetails>,
    pub routing_reasoning: String,
    #[schemars(with = "String")]
    pub estimated_completion_time: DateTime<Utc>,
    pub confidence_score: f32,
}

/// Worker assignment with reasoning (workers-specific)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerAssignmentDetails {
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    pub worker_name: String,
    pub capability_match_score: f32,
    pub estimated_execution_time_ms: u64,
    pub reasoning: String,
    pub load_factor: f32,
}

/// Worker pool events for monitoring
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum WorkerPoolEvent {
    WorkerRegistered {
        worker: Worker,
    },
    WorkerDeregistered {
        worker_id: Uuid,
    },
    WorkerStatusChanged {
        worker_id: Uuid,
        old_status: WorkerStatus,
        new_status: WorkerStatus,
    },
    TaskAssigned {
        task_id: Uuid,
        worker_id: Uuid,
    },
    TaskCompleted {
        task_id: Uuid,
        worker_id: Uuid,
        result: TaskExecutionResult,
    },
    TaskFailed {
        task_id: Uuid,
        worker_id: Uuid,
        error: String,
    },
    WorkerHealthChecked {
        worker_id: Uuid,
        is_healthy: bool,
        response_time_ms: u64,
        #[schemars(with = "String")]
        checked_at: DateTime<Utc>,
    },
    HealthCheckFailed {
        worker_id: Uuid,
        error: String,
    },
    PerformanceThresholdExceeded {
        worker_id: Uuid,
        metric: String,
        value: f64,
        threshold: f64,
    },
}

/// Worker update request
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
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
            health_status: WorkerHealthStatus::Healthy,
            health_metrics: None,
            last_health_check: None,
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
            let matches = requirements
                .required_languages
                .iter()
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
            let matches = requirements
                .required_frameworks
                .iter()
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
            let matches = requirements
                .required_domains
                .iter()
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

        match get_execution_status(result) {
            ExecutionStatus::Completed => self.performance_metrics.completed_tasks += 1,
            ExecutionStatus::Failed | ExecutionStatus::Timeout | ExecutionStatus::Cancelled => {
                self.performance_metrics.failed_tasks += 1;
            }
            _ => {}
        }

        // Update average execution time
        let total_time = self.performance_metrics.average_execution_time_ms
            * (self.performance_metrics.total_tasks - 1) as f64;
        self.performance_metrics.average_execution_time_ms =
            (total_time + result.duration_ms as f64) / self.performance_metrics.total_tasks as f64;

        // Update average quality score
        if let Some(output) = get_worker_output(result) {
            let total_quality = self.performance_metrics.average_quality_score
                * (self.performance_metrics.total_tasks - 1) as f32;
            self.performance_metrics.average_quality_score = (total_quality
                + output.self_assessment.quality_score)
                / self.performance_metrics.total_tasks as f32;
        }

        // Update average CAWS compliance
        if let Some(compliance) = get_caws_compliance(result) {
            let total_compliance = self.performance_metrics.average_caws_compliance
                * (self.performance_metrics.total_tasks - 1) as f32;
            self.performance_metrics.average_caws_compliance = (total_compliance
                + compliance.compliance_score)
                / self.performance_metrics.total_tasks as f32;
        }

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
            busy_workers: 0,
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

        let now = Utc::now();
        let worker_output = WorkerOutput {
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
        };

        let mut metadata = HashMap::new();
        metadata.insert(
            META_EXECUTION_STATUS.to_string(),
            serde_json::json!("Completed"),
        );
        metadata.insert(
            META_QUALITY_METRICS.to_string(),
            serde_json::to_value(QualityMetrics::default()).unwrap(),
        );
        metadata.insert(
            META_CAWS_COMPLIANCE.to_string(),
            serde_json::to_value(CawsComplianceResult {
                compliance_score: 0.9,
                ..Default::default()
            })
            .unwrap(),
        );
        metadata.insert(
            META_WORKER_OUTPUT.to_string(),
            serde_json::to_value(&worker_output).unwrap(),
        );
        metadata.insert(META_TOKENS_USED.to_string(), serde_json::json!(1500));

        let result = TaskExecutionResult {
            execution_id: Uuid::new_v4(),
            task_id: Uuid::new_v4(),
            success: true,
            output: serde_json::to_string(&worker_output).unwrap(),
            errors: vec![],
            metadata,
            started_at: now,
            completed_at: now,
            duration_ms: 2000,
            worker_id: Some(worker.id),
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

/// Task execution context for workers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskContext {
    pub task_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
    /// Tool ID for tool execution (optional)
    #[serde(default)]
    pub tool_id: Option<String>,
    /// Parameters for tool execution (optional)
    #[serde(default)]
    pub parameters: std::collections::HashMap<String, serde_json::Value>,
}

/// Task specification for workers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskSpec {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: TaskRequirements,
    pub context: TaskContext,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub risk_tier: RiskTier,
    pub scope: TaskScope,
}

/// Execution input for worker tasks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExecutionInput {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    pub prompt: String,
    pub context: String,
    pub requirements: String,
    pub caws_spec: Option<String>,
}

/// Raw execution result from worker
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RawExecutionResult {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    pub raw_output: String,
    pub execution_time_ms: u64,
    pub tokens_used: Option<u32>,
    pub quality_score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

/// CAWS specification for task execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsSpec {
    pub version: String,
    pub metadata: CawsMetadata,
    pub quality_gates: Vec<QualityGate>,
    pub compliance: ComplianceRequirements,
    pub validation_rules: Vec<ValidationRule>,
    pub benchmarks: Option<PerformanceBenchmarks>,
    pub security: SecurityRequirements,
}

/// CAWS metadata
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CawsMetadata {
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Quality gate definition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct QualityGate {
    pub name: String,
    pub required: bool,
    pub threshold: f32,
}

/// Compliance requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ComplianceRequirements {
    pub test_coverage_min: f32,
    pub mutation_score_min: f32,
    pub performance_budget_ms: Option<u64>,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationRule {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub severity: ViolationSeverity,
    pub rule_type: ValidationRuleType,
    pub file_patterns: Vec<String>,
    pub config: HashMap<String, serde_json::Value>,
}

/// Performance benchmarks
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PerformanceBenchmarks {
    pub response_time_ms: u64,
    pub throughput_rps: u32,
    pub memory_usage_mb: u32,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SecurityRequirements {
    pub authentication_required: bool,
    pub authorization_required: bool,
    pub encryption_required: bool,
}

impl Default for SecurityRequirements {
    fn default() -> Self {
        Self {
            authentication_required: true,
            authorization_required: true,
            encryption_required: true,
        }
    }
}

// Missing types that are referenced throughout the codebase

/// Worker communication messages
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum WorkerMessage {
    Started {
        worker_id: Uuid,
        subtask_id: Uuid,
        #[schemars(with = "String")]
        timestamp: DateTime<Utc>,
    },
    Progress {
        worker_id: Uuid,
        subtask_id: Uuid,
        progress_percentage: f32,
        message: String,
        #[schemars(with = "String")]
        timestamp: DateTime<Utc>,
    },
    Blocked {
        worker_id: Uuid,
        subtask_id: Uuid,
        reason: String,
        #[schemars(with = "String")]
        timestamp: DateTime<Utc>,
    },
    Completed {
        worker_id: Uuid,
        subtask_id: Uuid,
        result: WorkerOutput,
        #[schemars(with = "String")]
        timestamp: DateTime<Utc>,
    },
    Failed {
        worker_id: Uuid,
        subtask_id: Uuid,
        error: String,
        #[schemars(with = "String")]
        timestamp: DateTime<Utc>,
    },
}

/// Worker progress state during execution
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WorkerProgressStatus {
    Pending,
    Running,
    Completed,
    Failed,
    Blocked,
}

impl WorkerProgressStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            WorkerProgressStatus::Pending => "pending",
            WorkerProgressStatus::Running => "running",
            WorkerProgressStatus::Completed => "completed",
            WorkerProgressStatus::Failed => "failed",
            WorkerProgressStatus::Blocked => "blocked",
        }
    }
}

impl std::fmt::Display for WorkerProgressStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Worker progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkerProgress {
    pub worker_id: WorkerId,
    pub subtask_id: SubTaskId,
    pub progress_percentage: f32,
    pub status: WorkerProgressStatus,
    pub current_step: String,
    pub estimated_completion: Option<DateTime<Utc>>,
    #[schemars(with = "String")]
    pub last_updated: DateTime<Utc>,
    // Additional fields for tracker compatibility
    pub completed: u32,
    pub total: u32,
    pub task_weight: f32,
}

/// Overall progress tracking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Progress {
    pub total_tasks: u32,
    pub completed_tasks: u32,
    pub failed_tasks: u32,
    pub in_progress_tasks: u32,
    pub overall_percentage: f32,
    pub estimated_completion: Option<DateTime<Utc>>,
    #[schemars(with = "String")]
    pub last_updated: DateTime<Utc>,
}

/// Validation result for quality gates
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ValidationResult {
    Pass {
        /// Normalized score for the gate (0.0 - 1.0)
        score: f32,
        /// Human-readable details about the validation outcome
        details: String,
    },
    Fail {
        /// Normalized score for the gate (0.0 - 1.0)
        score: f32,
        /// Human-readable details about the failure
        details: String,
        /// Actionable suggestions for remediation
        suggestions: Vec<String>,
    },
    Warning {
        /// Normalized score for the gate (0.0 - 1.0)
        score: f32,
        /// Human-readable details about the warning
        details: String,
        /// Non-blocking recommendations to improve quality
        suggestions: Vec<String>,
    },
}

/// Validation context
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ValidationContext {
    #[schemars(with = "String")]
    pub task_id: Uuid,
    #[schemars(with = "String")]
    pub worker_id: Uuid,
    pub validation_type: String,
    pub requirements: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
    // Parallel execution validation fields
    #[serde(default)]
    pub package_name: Option<String>,
    #[serde(default)]
    pub workspace_root: Option<std::path::PathBuf>,
    #[serde(default)]
    pub execution_time: Option<std::time::Duration>,
    #[serde(default)]
    pub results: Option<Vec<crate::parallel_types::WorkerResult>>,
}

/// Artifact types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ArtifactType {
    SourceCode,
    Documentation,
    Test,
    Configuration,
    Data,
    Binary,
    Other(String),
}

/// Artifact representation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Artifact {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub path: String,
    pub artifact_type: ArtifactType,
    pub content: String,
    pub metadata: HashMap<String, serde_json::Value>,
    #[schemars(with = "String")]
    pub created_at: DateTime<Utc>,
    #[schemars(with = "String")]
    pub modified_at: DateTime<Utc>,
}

/// Worker health status (different from WorkerHealthStatus)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum WorkerHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Offline,
}

/// Severity levels for various operations
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum SeverityLevel {
    Low,
    Medium,
    High,
    Critical,
}

// WorkerSpecialty is now imported from parallel_types

/// Task definition for execution
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TaskDefinition {
    #[schemars(with = "String")]
    pub id: Uuid,
    pub name: String,
    pub description: String,
    pub required_tools: Vec<String>,
    pub parameters: HashMap<String, serde_json::Value>,
    pub timeout_seconds: Option<u32>,
    pub priority: TaskPriority,
    pub deadline: Option<DateTime<Utc>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Task status for tracking execution
/// Execution outcome for learning system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ExecutionOutcome {
    Success,
    Failure,
    Timeout,
    Cancelled,
}

/// Learning mode for adaptive systems
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum LearningMode {
    Learn,
    Apply,
    Optimize,
    Disabled,
}

/// Tool identifier
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ToolId {
    pub name: String,
    pub version: String,
}

/// Validation rule types
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum ValidationRuleType {
    Custom,
    Builtin,
    External,
    Acceptance,
    Invariant,
}

impl Default for TaskDefinition {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            description: String::new(),
            required_tools: Vec::new(),
            parameters: HashMap::new(),
            timeout_seconds: None,
            priority: TaskPriority::Medium,
            deadline: None,
            metadata: HashMap::new(),
        }
    }
}

// Duplicate Default impl removed - using the one defined earlier

impl Default for ToolId {
    fn default() -> Self {
        Self {
            name: String::new(),
            version: "1.0.0".to_string(),
        }
    }
}

impl Default for WorkerProgress {
    fn default() -> Self {
        Self {
            worker_id: WorkerId(Uuid::new_v4()),
            subtask_id: SubTaskId(Uuid::new_v4()),
            progress_percentage: 0.0,
            status: WorkerProgressStatus::Pending,
            current_step: String::new(),
            estimated_completion: None,
            last_updated: Utc::now(),
            completed: 0,
            total: 0,
            task_weight: 1.0,
        }
    }
}

impl Default for Progress {
    fn default() -> Self {
        Self {
            total_tasks: 0,
            completed_tasks: 0,
            failed_tasks: 0,
            in_progress_tasks: 0,
            overall_percentage: 0.0,
            estimated_completion: None,
            last_updated: Utc::now(),
        }
    }
}

impl Default for ValidationContext {
    fn default() -> Self {
        Self {
            task_id: Uuid::new_v4(),
            worker_id: Uuid::new_v4(),
            validation_type: String::new(),
            requirements: HashMap::new(),
            metadata: HashMap::new(),
            package_name: None,
            workspace_root: None,
            execution_time: None,
            results: None,
        }
    }
}

impl Default for Artifact {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            name: String::new(),
            path: String::new(),
            artifact_type: ArtifactType::Other("unknown".to_string()),
            content: String::new(),
            metadata: HashMap::new(),
            created_at: Utc::now(),
            modified_at: Utc::now(),
        }
    }
}

impl Default for WorkerAssignmentDetails {
    fn default() -> Self {
        Self {
            worker_id: Uuid::new_v4(),
            worker_name: "unknown".to_string(),
            capability_match_score: 0.0,
            estimated_execution_time_ms: 0,
            reasoning: "default assignment".to_string(),
            load_factor: 1.0,
        }
    }
}

impl Default for TaskSpec {
    fn default() -> Self {
        Self {
            id: Uuid::new_v4(),
            title: String::new(),
            description: String::new(),
            requirements: TaskRequirements::default(),
            context: TaskContext::default(),
            created_at: Utc::now(),
            deadline: None,
            risk_tier: RiskTier::Tier3,
            scope: TaskScope {
                domains: vec![],
                files_affected: vec![],
                files: vec![],
                directories: vec![],
                patterns: vec![],
                max_files: None,
                max_loc: None,
            },
        }
    }
}

impl Default for ExecutionInput {
    fn default() -> Self {
        Self {
            task_id: Uuid::new_v4(),
            prompt: String::new(),
            context: String::new(),
            requirements: String::new(),
            caws_spec: None,
        }
    }
}

impl Default for RawExecutionResult {
    fn default() -> Self {
        Self {
            task_id: Uuid::new_v4(),
            worker_id: Uuid::new_v4(),
            raw_output: String::new(),
            execution_time_ms: 0,
            tokens_used: None,
            quality_score: 0.0,
            metadata: HashMap::new(),
            error: None,
        }
    }
}

impl Default for CawsSpec {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            metadata: CawsMetadata::default(),
            quality_gates: Vec::new(),
            compliance: ComplianceRequirements::default(),
            validation_rules: Vec::new(),
            benchmarks: None,
            security: SecurityRequirements::default(),
        }
    }
}

impl Default for CawsMetadata {
    fn default() -> Self {
        Self {
            created_at: Utc::now(),
            created_by: String::new(),
            description: String::new(),
            tags: Vec::new(),
        }
    }
}

impl Default for ComplianceRequirements {
    fn default() -> Self {
        Self {
            test_coverage_min: 0.8,
            mutation_score_min: 0.7,
            performance_budget_ms: None,
        }
    }
}

impl Default for TaskContext {
    fn default() -> Self {
        Self {
            task_id: Uuid::new_v4(),
            worker_id: Uuid::new_v4(),
            start_time: Utc::now(),
            timeout_ms: 30000,
            retry_count: 0,
            max_retries: 3,
            metadata: HashMap::new(),
            tool_id: None,
            parameters: HashMap::new(),
        }
    }
}
