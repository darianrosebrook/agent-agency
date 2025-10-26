//! Worker pool types and data structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

// Use shared types from contracts
use agent_agency_contracts::{WorkerHealthStatus, WorkerHealthMetrics, RiskTier, WorkerType, WorkerRegistration, TaskPriority};

/// Clock trait for time operations
pub trait Clock {
    fn now(&self) -> DateTime<Utc>;
}

/// IdGenerator trait for generating unique identifiers
pub trait IdGenerator {
    fn generate(&self) -> Uuid;
}

/// Default clock implementation using system time
#[derive(Debug, Clone)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

/// Default ID generator implementation using UUID v4
#[derive(Debug, Clone)]
pub struct UuidGenerator;

impl IdGenerator for UuidGenerator {
    fn generate(&self) -> Uuid {
        Uuid::new_v4()
    }
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
    pub health_status: WorkerHealthStatus,
    pub health_metrics: Option<WorkerHealthMetrics>,
    pub last_health_check: Option<DateTime<Utc>>,
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
    pub quality_score: f32,  // 0.0 to 1.0
    pub speed_score: f32,    // 0.0 to 1.0
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
    pub busy_workers: u32,
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


/// Task requirements for routing
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
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

impl std::fmt::Display for ExecutionStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionStatus::Completed => write!(f, "Completed"),
            ExecutionStatus::Failed => write!(f, "Failed"),
            ExecutionStatus::Timeout => write!(f, "Timeout"),
            ExecutionStatus::Cancelled => write!(f, "Cancelled"),
            ExecutionStatus::Partial => write!(f, "Partial"),
        }
    }
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
    pub constitutional_ref: Option<String>,
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

// WorkerHealthStatus and WorkerHealthMetrics are now imported from agent_agency_contracts

/// Worker metrics collection from /metrics endpoint
#[derive(Debug, Clone, Default)]
pub struct WorkerMetricsCollection {
    pub cpu_usage: Option<f64>,
    pub memory_usage: Option<f64>,
    pub active_tasks: Option<u32>,
    pub queue_depth: Option<u32>,
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
    pub selected_workers: Vec<WorkerAssignmentDetails>,
    pub routing_reasoning: String,
    pub estimated_completion_time: DateTime<Utc>,
    pub confidence_score: f32,
}

/// Worker assignment with reasoning (workers-specific)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerAssignmentDetails {
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
        let total_time = self.performance_metrics.average_execution_time_ms
            * (self.performance_metrics.total_tasks - 1) as f64;
        self.performance_metrics.average_execution_time_ms = (total_time
            + result.execution_time_ms as f64)
            / self.performance_metrics.total_tasks as f64;

        // Update average quality score
        if let Some(output) = &result.output {
            let total_quality = self.performance_metrics.average_quality_score
                * (self.performance_metrics.total_tasks - 1) as f32;
            self.performance_metrics.average_quality_score = (total_quality
                + output.self_assessment.quality_score)
                / self.performance_metrics.total_tasks as f32;
        }

        // Update average CAWS compliance
        let total_compliance = self.performance_metrics.average_caws_compliance
            * (self.performance_metrics.total_tasks - 1) as f32;
        self.performance_metrics.average_caws_compliance = (total_compliance
            + result.caws_compliance.compliance_score)
            / self.performance_metrics.total_tasks as f32;

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

/// Task execution context for workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub task_id: uuid::Uuid,
    pub worker_id: uuid::Uuid,
    pub start_time: chrono::DateTime<chrono::Utc>,
    pub timeout_ms: u64,
    pub retry_count: u32,
    pub max_retries: u32,
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}


/// Task scope definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskScope {
    pub domains: Vec<String>,
    pub files_affected: Vec<String>,
    pub max_loc: Option<usize>,
}

/// Task specification for workers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
    pub id: Uuid,
    pub title: String,
    pub description: String,
    pub requirements: TaskRequirements,
    pub context: TaskContext,
    pub created_at: DateTime<Utc>,
    pub deadline: Option<DateTime<Utc>>,
    pub risk_tier: RiskTier,
    pub scope: TaskScope,
}

/// Execution input for worker tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionInput {
    pub task_id: Uuid,
    pub prompt: String,
    pub context: String,
    pub requirements: String,
    pub caws_spec: Option<String>,
}

/// Raw execution result from worker
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RawExecutionResult {
    pub task_id: Uuid,
    pub worker_id: Uuid,
    pub raw_output: String,
    pub execution_time_ms: u64,
    pub tokens_used: Option<u32>,
    pub quality_score: f32,
    pub metadata: HashMap<String, serde_json::Value>,
    pub error: Option<String>,
}

/// CAWS specification for task execution
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CawsMetadata {
    pub created_at: DateTime<Utc>,
    pub created_by: String,
    pub description: String,
    pub tags: Vec<String>,
}

/// Quality gate definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityGate {
    pub name: String,
    pub required: bool,
    pub threshold: f32,
}

/// Compliance requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceRequirements {
    pub test_coverage_min: f32,
    pub mutation_score_min: f32,
    pub performance_budget_ms: Option<u64>,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    pub name: String,
    pub description: String,
    pub severity: ViolationSeverity,
}

/// Performance benchmarks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceBenchmarks {
    pub response_time_ms: u64,
    pub throughput_rps: u32,
    pub memory_usage_mb: u32,
}

/// Security requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        }
    }
}
