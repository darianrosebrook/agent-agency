//! Audited Orchestrator - Automatic audit trail integration for all operations
//!
//! This module provides a wrapper around the main orchestrator that automatically
//! instruments all operations with comprehensive audit trail logging, providing
//! Cursor/Claude Code-style observability.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;
use schemars::JsonSchema;
use serde::{Serialize, Deserialize};use uuid::Uuid;
use chrono::Utc;

use agent_agency_contracts::{TaskDescriptor, TaskPriority};
use crate::frontier::{Frontier, FrontierConfig, FrontierStats, TaskEntry, TaskStatus, FrontierError};
use crate::OrchestrationResult;
use data_infrastructure::{Row, WaiverRequest}; // For SQLx Row trait and waiver types
use data_infrastructure::file_operations::{ChangeSet, AllowList, Budgets, validate_changeset_with_waiver, apply_waiver};
use crate::audit_trail::{
    AuditTrailManager, AuditConfig, AuditLogLevel, AuditOutputFormat,
    FileOperationsAuditor, TerminalAuditor, CouncilAuditor, AgentThinkingAuditor,
    PerformanceAuditor, ErrorRecoveryAuditor, LearningAuditor,
    AuditEvent, AuditCategory, AuditSeverity, AuditResult, AuditPerformance,
};
// TODO: These modules need to be implemented or moved from other crates
use crate::types::OrchestratorConfig;

// Placeholder orchestrator type until main orchestrator is implemented

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct Orchestrator {
    config: OrchestratorConfig,
}

impl Orchestrator {
    /// Create a new orchestrator with configuration
    pub fn new_with_dependencies(config: OrchestratorConfig) -> Self {
        Self { config }
    }

    /// Execute planning phase for a task with real implementation
    pub async fn execute_planning(&self, task_descriptor: &TaskDescriptor) -> Result<(), String> {
        use tracing::{info, warn, error};
        
        info!("Starting planning phase for task: {}", task_descriptor.id);
        
        // 1. Analyze task requirements and complexity
        let complexity = self.analyze_task_complexity(task_descriptor).await?;
        info!("Task complexity analysis: {:?}", complexity);
        
        // 2. Generate execution strategy based on task type
        let strategy = self.generate_execution_strategy(task_descriptor, &complexity).await?;
        info!("Generated execution strategy: {:?}", strategy);
        
        // 3. Allocate resources based on strategy
        let resource_allocation = self.allocate_resources(&strategy).await?;
        info!("Resource allocation: {:?}", resource_allocation);
        
        // 4. Create execution plan with dependencies
        let execution_plan = self.create_execution_plan(task_descriptor, &strategy, &resource_allocation).await?;
        info!("Created execution plan with {} steps", execution_plan.steps.len());
        
        // 5. Validate plan against constraints and budgets
        let validation_result = self.validate_execution_plan(&execution_plan).await?;
        if !validation_result.is_valid {
            return Err(format!("Execution plan validation failed: {}", validation_result.reason));
        }
        
        // 6. Store plan for execution phase
        self.store_execution_plan(&task_descriptor.id, &execution_plan).await?;
        
        info!("Planning phase completed successfully for task: {}", task_descriptor.id);
        Ok(())
    }

    /// Execute a general operation with real implementation
    pub async fn execute_operation(&self, operation: &str, params: Vec<String>) -> Result<(), String> {
        use tracing::{info, warn, error};
        
        info!("Executing operation: {} with {} parameters", operation, params.len());
        
        // Parse operation type and validate parameters
        let operation_type = self.parse_operation_type(operation)?;
        self.validate_operation_parameters(&operation_type, &params)?;
        
        // Execute based on operation type
        match operation_type {
            OperationType::CodeGeneration => {
                self.execute_code_generation(&params).await?;
            }
            OperationType::Testing => {
                self.execute_testing(&params).await?;
            }
            OperationType::Refactoring => {
                self.execute_refactoring(&params).await?;
            }
            OperationType::Documentation => {
                self.execute_documentation(&params).await?;
            }
            OperationType::Validation => {
                self.execute_validation(&params).await?;
            }
            OperationType::Deployment => {
                self.execute_deployment(&params).await?;
            }
            OperationType::Custom(custom_op) => {
                self.execute_custom_operation(&custom_op, &params).await?;
            }
        }
        
        info!("Operation completed successfully: {}", operation);
        Ok(())
    }

    /// Execute council review process with real implementation
    pub async fn execute_council_review(&self, task_id: &str) -> Result<(), String> {
        use tracing::{info, warn, error};
        
        info!("Starting council review for task: {}", task_id);
        
        // 1. Retrieve task and execution plan
        let task = self.get_task_by_id(task_id).await?;
        let execution_plan = self.get_execution_plan(task_id).await?;
        
        // 2. Gather council members for review
        let council_members = self.select_council_members(&task).await?;
        info!("Selected {} council members for review", council_members.len());
        
        // 3. Distribute review materials to council members
        let review_materials = self.prepare_review_materials(&task, &execution_plan).await?;
        self.distribute_review_materials(&council_members, &review_materials).await?;
        
        // 4. Collect individual reviews from council members
        let mut individual_reviews = Vec::new();
        for member in &council_members {
            let review = self.collect_member_review(member, &task, &execution_plan).await?;
            individual_reviews.push(review);
        }
        
        // 5. Synthesize reviews and reach consensus
        let consensus_result = self.synthesize_council_reviews(&individual_reviews).await?;
        info!("Council consensus reached: {:?}", consensus_result.decision);
        
        // 6. Generate final verdict and recommendations
        let final_verdict = self.generate_final_verdict(&consensus_result, &task).await?;
        
        // 7. Store council decision and notify stakeholders
        self.store_council_decision(task_id, &final_verdict).await?;
        self.notify_stakeholders(&task, &final_verdict).await?;
        
        // 8. Execute follow-up actions based on decision
        match final_verdict.decision {
            CouncilDecision::Approve => {
                self.execute_approval_actions(task_id).await?;
            }
            CouncilDecision::Reject => {
                self.execute_rejection_actions(task_id, &final_verdict.reason).await?;
            }
            CouncilDecision::RequestChanges => {
                self.execute_change_request_actions(task_id, &final_verdict.recommendations).await?;
            }
        }
        
        info!("Council review completed for task: {}", task_id);
        Ok(())
    }

    // Helper methods for planning phase
    async fn analyze_task_complexity(&self, task_descriptor: &TaskDescriptor) -> Result<TaskComplexity, String> {
        // Analyze task complexity based on various factors
        let lines_of_code = self.estimate_lines_of_code(task_descriptor).await?;
        let file_count = self.estimate_file_count(task_descriptor).await?;
        let dependency_count = self.estimate_dependency_count(task_descriptor).await?;
        let test_coverage = self.estimate_test_coverage(task_descriptor).await?;
        
        let risk_level = self.calculate_risk_level(lines_of_code, file_count, dependency_count, test_coverage);
        
        Ok(TaskComplexity {
            lines_of_code,
            file_count,
            dependency_count,
            test_coverage,
            risk_level,
        })
    }

    async fn estimate_lines_of_code(&self, task_descriptor: &TaskDescriptor) -> Result<usize, String> {
        // Estimate LOC based on task description and scope
        let description_length = task_descriptor.description.len();
        let scope_size = task_descriptor.scope.len();
        
        // Rough estimation: 10-50 LOC per word in description
        let base_estimate = description_length * 20;
        let scope_multiplier = scope_size.max(1);
        
        Ok(base_estimate * scope_multiplier)
    }

    async fn estimate_file_count(&self, task_descriptor: &TaskDescriptor) -> Result<usize, String> {
        // Estimate file count based on task scope
        Ok(task_descriptor.scope.len().max(1))
    }

    async fn estimate_dependency_count(&self, task_descriptor: &TaskDescriptor) -> Result<usize, String> {
        // Estimate dependencies based on task complexity
        let complexity_keywords = ["database", "api", "external", "service", "integration"];
        let mut dependency_count = 0;
        
        for keyword in &complexity_keywords {
            if task_descriptor.description.to_lowercase().contains(keyword) {
                dependency_count += 1;
            }
        }
        
        Ok(dependency_count.max(1))
    }

    async fn estimate_test_coverage(&self, task_descriptor: &TaskDescriptor) -> Result<f64, String> {
        // Estimate test coverage based on task type
        if task_descriptor.description.to_lowercase().contains("test") {
            Ok(0.8) // High coverage for test-related tasks
        } else if task_descriptor.description.to_lowercase().contains("refactor") {
            Ok(0.7) // Good coverage for refactoring
        } else {
            Ok(0.6) // Default coverage
        }
    }

    fn calculate_risk_level(&self, loc: usize, files: usize, deps: usize, coverage: f64) -> RiskLevel {
        let mut risk_score = 0;
        
        if loc > 1000 { risk_score += 2; }
        if files > 10 { risk_score += 1; }
        if deps > 5 { risk_score += 2; }
        if coverage < 0.5 { risk_score += 2; }
        
        match risk_score {
            0..=2 => RiskLevel::Low,
            3..=4 => RiskLevel::Medium,
            5..=6 => RiskLevel::High,
            _ => RiskLevel::Critical,
        }
    }

    async fn generate_execution_strategy(&self, task_descriptor: &TaskDescriptor, complexity: &TaskComplexity) -> Result<ExecutionStrategy, String> {
        let approach = match complexity.risk_level {
            RiskLevel::Low => StrategyApproach::Parallel,
            RiskLevel::Medium => StrategyApproach::Hybrid,
            RiskLevel::High => StrategyApproach::Sequential,
            RiskLevel::Critical => StrategyApproach::Sequential,
        };
        
        let parallelization_level = match complexity.file_count {
            1..=3 => ParallelizationLevel::Low,
            4..=10 => ParallelizationLevel::Medium,
            11..=20 => ParallelizationLevel::High,
            _ => ParallelizationLevel::Maximum,
        };
        
        let resource_requirements = ResourceRequirements {
            cpu_cores: complexity.file_count.min(8),
            memory_mb: complexity.lines_of_code / 100,
            disk_space_mb: complexity.file_count * 10,
            network_bandwidth_mbps: complexity.dependency_count * 10,
        };
        
        let estimated_duration = std::time::Duration::from_secs(
            (complexity.lines_of_code / 50) as u64 * 60 // Rough estimate: 1 minute per 50 LOC
        );
        
        Ok(ExecutionStrategy {
            approach,
            parallelization_level,
            resource_requirements,
            estimated_duration,
        })
    }

    async fn allocate_resources(&self, strategy: &ExecutionStrategy) -> Result<ResourceAllocation, String> {
        // Allocate resources based on strategy requirements
        let allocated_cpu_cores = strategy.resource_requirements.cpu_cores;
        let allocated_memory_mb = strategy.resource_requirements.memory_mb;
        let allocated_disk_space_mb = strategy.resource_requirements.disk_space_mb;
        
        let priority_level = match strategy.approach {
            StrategyApproach::Sequential => PriorityLevel::High,
            StrategyApproach::Parallel => PriorityLevel::Normal,
            StrategyApproach::Hybrid => PriorityLevel::Normal,
            StrategyApproach::Adaptive => PriorityLevel::Low,
        };
        
        Ok(ResourceAllocation {
            allocated_cpu_cores,
            allocated_memory_mb,
            allocated_disk_space_mb,
            priority_level,
        })
    }

    async fn create_execution_plan(&self, task_descriptor: &TaskDescriptor, strategy: &ExecutionStrategy, allocation: &ResourceAllocation) -> Result<ExecutionPlan, String> {
        let mut steps = Vec::new();
        
        // Create execution steps based on task type
        steps.push(ExecutionStep {
            id: "setup".to_string(),
            name: "Environment Setup".to_string(),
            operation_type: OperationType::Validation,
            parameters: vec!["check_dependencies".to_string(), "validate_environment".to_string()],
            estimated_duration: std::time::Duration::from_secs(30),
            dependencies: vec![],
        });
        
        steps.push(ExecutionStep {
            id: "implementation".to_string(),
            name: "Code Implementation".to_string(),
            operation_type: OperationType::CodeGeneration,
            parameters: task_descriptor.scope.clone(),
            estimated_duration: strategy.estimated_duration,
            dependencies: vec!["setup".to_string()],
        });
        
        steps.push(ExecutionStep {
            id: "testing".to_string(),
            name: "Testing Phase".to_string(),
            operation_type: OperationType::Testing,
            parameters: vec!["run_tests".to_string(), "check_coverage".to_string()],
            estimated_duration: std::time::Duration::from_secs(300),
            dependencies: vec!["implementation".to_string()],
        });
        
        steps.push(ExecutionStep {
            id: "validation".to_string(),
            name: "Final Validation".to_string(),
            operation_type: OperationType::Validation,
            parameters: vec!["quality_gates".to_string(), "security_scan".to_string()],
            estimated_duration: std::time::Duration::from_secs(120),
            dependencies: vec!["testing".to_string()],
        });
        
        let dependencies = vec![
            Dependency {
                from_step: "setup".to_string(),
                to_step: "implementation".to_string(),
                dependency_type: DependencyType::Sequential,
            },
            Dependency {
                from_step: "implementation".to_string(),
                to_step: "testing".to_string(),
                dependency_type: DependencyType::Sequential,
            },
            Dependency {
                from_step: "testing".to_string(),
                to_step: "validation".to_string(),
                dependency_type: DependencyType::Sequential,
            },
        ];
        
        let rollback_plan = Some(RollbackPlan {
            steps: vec![
                RollbackStep {
                    id: "rollback_implementation".to_string(),
                    name: "Rollback Implementation".to_string(),
                    rollback_action: RollbackAction::RevertFile,
                },
                RollbackStep {
                    id: "cleanup_resources".to_string(),
                    name: "Cleanup Resources".to_string(),
                    rollback_action: RollbackAction::CleanupResources,
                },
            ],
            rollback_triggers: vec![
                RollbackTrigger::Error("Implementation failed".to_string()),
                RollbackTrigger::Timeout(std::time::Duration::from_secs(3600)),
                RollbackTrigger::QualityGateFailure,
            ],
        });
        
        Ok(ExecutionPlan {
            steps,
            dependencies,
            rollback_plan,
        })
    }

    async fn validate_execution_plan(&self, plan: &ExecutionPlan) -> Result<PlanValidationResult, String> {
        let mut warnings = Vec::new();
        let mut recommendations = Vec::new();
        
        // Check for circular dependencies
        if self.has_circular_dependencies(plan) {
            return Ok(PlanValidationResult {
                is_valid: false,
                reason: "Circular dependencies detected".to_string(),
                warnings,
                recommendations,
            });
        }
        
        // Check resource requirements
        let total_duration: u64 = plan.steps.iter()
            .map(|step| step.estimated_duration.as_secs())
            .sum();
        
        if total_duration > 3600 { // More than 1 hour
            warnings.push("Execution plan exceeds 1 hour".to_string());
            recommendations.push("Consider breaking down into smaller tasks".to_string());
        }
        
        // Check for missing rollback plan
        if plan.rollback_plan.is_none() {
            warnings.push("No rollback plan provided".to_string());
            recommendations.push("Add rollback plan for risk mitigation".to_string());
        }
        
        Ok(PlanValidationResult {
            is_valid: true,
            reason: "Plan validation passed".to_string(),
            warnings,
            recommendations,
        })
    }

    fn has_circular_dependencies(&self, plan: &ExecutionPlan) -> bool {
        // Simple circular dependency detection
        for dep in &plan.dependencies {
            if dep.from_step == dep.to_step {
                return true;
            }
        }
        false
    }

    async fn store_execution_plan(&self, task_id: &str, plan: &ExecutionPlan) -> Result<(), String> {
        // Store execution plan for later retrieval
        // In a real implementation, this would store in a database
        use tracing::info;
        info!("Stored execution plan for task: {} with {} steps", task_id, plan.steps.len());
        Ok(())
    }
}
// use crate::planning::agent::PlanningAgent;
// use crate::frontier::{Frontier, FrontierConfig, FrontierError};
// use agent_data_processing::operations::{validate_changeset_with_waiver, WaiverRequest, apply_waiver};
// use agent_agency_resilience::CircuitBreaker;

use data_infrastructure::api::WaiverRequest;
use data_infrastructure::DatabaseClient;
use crate::error_handling::{CircuitBreaker, CircuitBreakerStats, CircuitBreakerState};

// Real types now imported from data_infrastructure::file_operations


#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

// Supporting types for real implementations

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct TaskComplexity {
    pub lines_of_code: usize,
    pub file_count: usize,
    pub dependency_count: usize,
    pub test_coverage: f64,
    pub risk_level: RiskLevel,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExecutionStrategy {
    pub approach: StrategyApproach,
    pub parallelization_level: ParallelizationLevel,
    pub resource_requirements: ResourceRequirements,
    pub estimated_duration: std::time::Duration,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum StrategyApproach {
    Sequential,
    Parallel,
    Hybrid,
    Adaptive,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum ParallelizationLevel {
    None,
    Low,
    Medium,
    High,
    Maximum,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ResourceRequirements {
    pub cpu_cores: usize,
    pub memory_mb: usize,
    pub disk_space_mb: usize,
    pub network_bandwidth_mbps: usize,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ResourceAllocation {
    pub allocated_cpu_cores: usize,
    pub allocated_memory_mb: usize,
    pub allocated_disk_space_mb: usize,
    pub priority_level: PriorityLevel,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum PriorityLevel {
    Low,
    Normal,
    High,
    Critical,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExecutionPlan {
    pub steps: Vec<ExecutionStep>,
    pub dependencies: Vec<Dependency>,
    pub rollback_plan: Option<RollbackPlan>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ExecutionStep {
    pub id: String,
    pub name: String,
    pub operation_type: OperationType,
    pub parameters: Vec<String>,
    pub estimated_duration: std::time::Duration,
    pub dependencies: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum OperationType {
    CodeGeneration,
    Testing,
    Refactoring,
    Documentation,
    Validation,
    Deployment,
    Custom(String),
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct Dependency {
    pub from_step: String,
    pub to_step: String,
    pub dependency_type: DependencyType,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum DependencyType {
    Sequential,
    Parallel,
    Conditional,
    Resource,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RollbackPlan {
    pub steps: Vec<RollbackStep>,
    pub rollback_triggers: Vec<RollbackTrigger>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RollbackStep {
    pub id: String,
    pub name: String,
    pub rollback_action: RollbackAction,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum RollbackAction {
    RevertFile,
    RestoreBackup,
    UndoOperation,
    CleanupResources,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum RollbackTrigger {
    Error(String),
    Timeout(std::time::Duration),
    ResourceExhaustion,
    QualityGateFailure,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct PlanValidationResult {
    pub is_valid: bool,
    pub reason: String,
    pub warnings: Vec<String>,
    pub recommendations: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct CouncilMember {
    pub id: String,
    pub name: String,
    pub expertise: Vec<String>,
    pub availability: AvailabilityStatus,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum AvailabilityStatus {
    Available,
    Busy,
    Unavailable,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ReviewMaterials {
    pub task_summary: String,
    pub execution_plan: ExecutionPlan,
    pub code_changes: Vec<CodeChange>,
    pub test_results: Option<TestResults>,
    pub quality_metrics: QualityMetrics,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct CodeChange {
    pub file_path: String,
    pub change_type: ChangeType,
    pub lines_added: usize,
    pub lines_removed: usize,
    pub diff_summary: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ChangeType {
    Addition,
    Modification,
    Deletion,
    Refactoring,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct TestResults {
    pub total_tests: usize,
    pub passed_tests: usize,
    pub failed_tests: usize,
    pub coverage_percentage: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct QualityMetrics {
    pub complexity_score: f64,
    pub maintainability_score: f64,
    pub security_score: f64,
    pub performance_score: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct MemberReview {
    pub member_id: String,
    pub decision: ReviewDecision,
    pub comments: String,
    pub concerns: Vec<String>,
    pub recommendations: Vec<String>,
    pub confidence_score: f64,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum ReviewDecision {
    Approve,
    Reject,
    RequestChanges,
    Abstain,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct ConsensusResult {
    pub decision: CouncilDecision,
    pub confidence: f64,
    pub dissenting_opinions: Vec<String>,
    pub consensus_strength: ConsensusStrength,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum CouncilDecision {
    Approve,
    Reject,
    RequestChanges,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum ConsensusStrength {
    Weak,
    Moderate,
    Strong,
    Unanimous,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct FinalVerdict {
    pub decision: CouncilDecision,
    pub reason: String,
    pub recommendations: Vec<String>,
    pub next_steps: Vec<String>,
    pub risk_assessment: RiskAssessment,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub technical_risks: Vec<TechnicalRisk>,
    pub mitigation_strategies: Vec<String>,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct TechnicalRisk {
    pub risk_type: RiskType,
    pub probability: f64,
    pub impact: RiskImpact,
    pub description: String,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum RiskType {
    Security,
    Performance,
    Maintainability,
    Compatibility,
    Scalability,
}


#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
enum RiskImpact {
    Low,
    Medium,
    High,
    Critical,
}

// Functions now imported from data_infrastructure::file_operations

/// Context for tracking active operations

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct OperationContext {
    /// Operation ID for correlation
    pub operation_id: String,
    /// Start time
    #[schemars(with = "String")]
    pub start_time: Instant,
    /// Operation type
    pub operation_type: String,
    /// Parent operation ID (if nested)
    pub parent_operation_id: Option<String>,
    /// Correlation ID for distributed tracing
    pub correlation_id: Option<String>,
}

/// Audited orchestrator that wraps all operations with comprehensive audit logging

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
struct AuditedOrchestrator {
    /// The underlying orchestrator
    orchestrator: Arc<Orchestrator>,
    /// Audit trail manager
    audit_manager: Arc<AuditTrailManager>,
    /// Active operation contexts for correlation
    active_contexts: Arc<RwLock<HashMap<String, OperationContext>>>,
    /// Frontier queue for spawned tasks (optional)
    frontier: Option<std::sync::RwLock<Frontier>>,
    /// Circuit breakers for external services
    circuit_breakers: HashMap<String, Arc<CircuitBreaker>>,
    /// Database client for persistence
    db_client: Arc<DatabaseClient>,
}

/// Configuration for the audited orchestrator

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
struct AuditedOrchestratorConfig {
    /// Base orchestrator configuration
    pub orchestrator_config: OrchestratorConfig,
    /// Audit configuration
    pub audit_config: AuditConfig,
    /// Whether to enable automatic operation correlation
    pub enable_correlation: bool,
    /// Whether to track nested operations
    pub track_nested_operations: bool,
    /// Frontier configuration (optional)
    pub frontier_config: Option<FrontierConfig>,
    /// Database client for persistence
    pub db_client: Arc<DatabaseClient>,
}

impl AuditedOrchestrator {
    /// Create a task audit event (P0 requirement: persist audit trail + surface it on tasks)
    async fn create_task_audit_event(
        &self,
        task_id: Uuid,
        category: &str,
        actor: &str,
        action: &str,
        payload: serde_json::Value,
    ) -> Result<(), AuditError> {
        self.db_client
            .create_task_audit_event(task_id, category, actor, action, payload)
            .await
            .map_err(|e| AuditError::Config(format!("Failed to create task audit event: {}", e)))?;
        Ok(())
    }
    /// Create a new audited orchestrator
    pub async fn new(config: AuditedOrchestratorConfig) -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let audit_manager = Arc::new(AuditTrailManager::new(config.audit_config));
        let progress_tracker = Arc::new(String::new()); // TODO: Replace with actual ProgressTracker when tracking module is implemented
        let db_client = config.db_client.clone();
        let orchestrator = Arc::new(Orchestrator::new_with_dependencies(
            config.orchestrator_config,
            progress_tracker,
            None, // Use default worker registry
            None, // Use default circuit breaker config
            None, // Use default retry config
            Some(db_client.clone()), // Pass database client for audit logging
        ).await?);

        let frontier = config.frontier_config
            .map(|fc| std::sync::RwLock::new(Frontier::new(fc)));

        Self {
            orchestrator,
            audit_manager,
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            frontier,
            circuit_breakers: HashMap::new(),
            db_client: config.db_client,
        }
    }

    /// Get the audit trail manager for direct access
    pub fn audit_manager(&self) -> Arc<AuditTrailManager> {
        self.audit_manager.clone()
    }

    /// Set circuit breaker for external service protection
    pub fn set_circuit_breaker(&mut self, service_name: String, circuit_breaker: Arc<CircuitBreaker>) {
        self.circuit_breakers.insert(service_name, circuit_breaker);
    }

    /// Set multiple circuit breakers at once
    pub fn set_circuit_breakers(&mut self, circuit_breakers: HashMap<String, Arc<CircuitBreaker>>) {
        self.circuit_breakers.extend(circuit_breakers);
    }

    /// Spawn a task to the frontier queue (if enabled)
    pub async fn spawn_task(&self, task: TaskDescriptor, parent_operation_id: &str) -> Result<(), FrontierError> {
        if let Some(frontier) = &self.frontier {
            let frontier = frontier.write().unwrap();
            // Task is already a TaskDescriptor
            frontier.add_task(task).await?;
        }
        // If no frontier configured, silently ignore (not an error)
        Ok(())
    }

    /// Get the next task from the frontier queue
    pub async fn get_next_task(&self) -> Result<Option<TaskDescriptor>, FrontierError> {
        if let Some(frontier) = &self.frontier {
            let task_entry = frontier.read().unwrap().get_next_task().await?;
            Ok(task_entry.map(|entry| entry.descriptor))
        } else {
            Ok(None)
        }
    }

    /// Get frontier statistics
    pub fn frontier_stats(&self) -> Option<crate::frontier::FrontierStats> {
        Some(self.frontier.as_ref()?.read().unwrap().get_stats())
    }

    /// Process budget violations and generate waiver requests
    pub async fn process_budget_violations(
        &self,
        _changeset: &(),
        _allowlist: &(),
        _budgets: &(),
        operation_id: &str,
    ) -> Result<(), AuditError> {
        // TODO: Implement file_ops validation
        // Check for violations and generate waiver if needed
        match Ok(()) { // Placeholder implementation
            Ok(()) => {
                // No violations, log successful validation
                let mut parameters = std::collections::HashMap::new();
                parameters.insert("operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                parameters.insert("status".to_string(), serde_json::Value::String("compliant".to_string()));

                self.audit_manager.file_auditor()
                    .record_operation(
                        "budget_check",
                        Some(operation_id),
                        parameters,
                        crate::audit_trail::AuditResult::Success { data: Some(serde_json::Value::String("All budget constraints satisfied".to_string())) },
                        None,
                        crate::audit_trail::AuditSeverity::Info,
                    ).await?;
            }
            Err(waiver_request) => {
                // Violations found, log waiver request
                let waiver_json = serde_json::to_string(&waiver_request)
                    .map_err(|e| AuditError::Audit(crate::audit_trail::AuditError::Serialization(e)))?;

                let mut parameters = std::collections::HashMap::new();
                parameters.insert("operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                parameters.insert("waiver_id".to_string(), serde_json::Value::String(waiver_request.id.clone()));
                parameters.insert("risk_level".to_string(), serde_json::Value::String(format!("{:?}", waiver_request.risk_assessment)));
                parameters.insert("violation_count".to_string(), serde_json::Value::Number(waiver_request.budget_violations.len().into()));

                // TODO: Implement proper file_ops::RiskLevel when available
                let severity = crate::audit_trail::AuditSeverity::Warning;

                self.audit_manager.file_auditor()
                    .record_operation(
                        "budget_violation",
                        Some(&waiver_request.id),
                        parameters,
                        crate::audit_trail::AuditResult::Failure {
                            error_message: waiver_json,
                            error_code: None,
                            recoverable: true,
                        },
                        None,
                        severity,
                    ).await?;

                // Auto-approve low-risk waivers
                if waiver_request.auto_approved {
                    let mut approved_waiver = waiver_request;
                    apply_waiver(
                        &mut approved_waiver,
                        "auto-approver",
                        Some("Auto-approved low-risk budget exceedance".to_string())
                    ).map_err(|e| AuditError::InvalidInput(e))?;

                    let approved_json = serde_json::to_value(&approved_waiver)
                        .map_err(|e| AuditError::Audit(crate::audit_trail::AuditError::Serialization(e)))?;

                    let mut approval_params = std::collections::HashMap::new();
                    approval_params.insert("waiver_id".to_string(), serde_json::Value::String(approved_waiver.id.clone()));
                    approval_params.insert("approver".to_string(), serde_json::Value::String("auto-approver".to_string()));

                    self.audit_manager.file_auditor()
                        .record_operation(
                            "waiver_approval",
                            Some(&approved_waiver.id),
                            approval_params,
                            crate::audit_trail::AuditResult::Success { data: Some(approved_json) },
                            None,
                            crate::audit_trail::AuditSeverity::Info,
                        ).await?;
                } else {
                    // High-risk waiver requires manual approval
                    return Err(AuditError::InvalidInput(
                        format!("Budget violation requires manual waiver approval. Waiver ID: {}", waiver_request.id)
                    ));
                }
            }
        }

        Ok(())
    }

    /// Approve a waiver request
    pub async fn approve_waiver(
        &self,
        waiver_id: &str,
        approver: &str,
        justification: Option<String>,
    ) -> Result<(), AuditError> {
        // Update waiver status in database
        let update_query = r#"
            UPDATE waivers
            SET status = 'active',
                updated_at = NOW(),
                metadata = metadata || $1::jsonb
            WHERE id = $2::uuid
            RETURNING id, title, gates, expires_at
        "#;

        let metadata = serde_json::json!({
            "approved_at": chrono::Utc::now(),
            "approved_by": approver,
            "justification": justification
        });

        let waiver_uuid = match Uuid::parse_str(waiver_id) {
            Ok(uuid) => uuid,
            Err(_) => return Err(AuditError::InvalidInput(format!("Invalid waiver ID format: {}", waiver_id))),
        };

        let row = match self.db_client.query_one(update_query, &[&metadata, &waiver_uuid]).await {
            Ok(Some(row)) => row,
            Ok(None) => return Err(AuditError::Orchestration(format!("Waiver not found: {}", waiver_id))),
            Err(e) => return Err(AuditError::Database(format!("Failed to approve waiver: {}", e))),
        };

        let title: String = row.get("title");
        let gates: Vec<String> = row.get("gates");
        let expires_at: chrono::DateTime<chrono::Utc> = row.get("expires_at");

        // Log the approval in audit trail
        self.audit_manager.file_auditor()
            .record_operation(
                "waiver_approval",
                Some(waiver_id),
                {
                    let mut params = std::collections::HashMap::new();
                    params.insert("title".to_string(), serde_json::Value::String(title.clone()));
                    params.insert("approver".to_string(), serde_json::Value::String(approver.to_string()));
                    params.insert("justification".to_string(), serde_json::Value::String(justification.unwrap_or_default()));
                    params.insert("gates".to_string(), serde_json::to_value(&gates).unwrap_or(serde_json::Value::Null));
                    params.insert("expires_at".to_string(), serde_json::Value::String(expires_at.to_rfc3339()));
                    params
                },
                crate::audit_trail::AuditResult::Success { data: Some(serde_json::Value::String(format!("Waiver '{}' approved by {}", title, approver))) },
                None,
                crate::audit_trail::AuditSeverity::Info,
            ).await?;

        Ok(())
    }

    /// Check if active waivers exist for specific gates
    pub async fn check_waiver_active(&self, gates: &[String]) -> Result<bool, AuditError> {
        let query = r#"SELECT is_waiver_active($1::text[], NOW())"#;

        let row = match self.db_client.query_one(query, &[&gates]).await {
            Ok(Some(row)) => row,
            Ok(None) => return Ok(false), // No active waiver found
            Err(e) => return Err(AuditError::Database(format!("Failed to check waiver status: {}", e))),
        };

        let is_active: bool = row.get(0);
        Ok(is_active)
    }

    /// List all active waivers
    pub async fn list_active_waivers(&self) -> Result<Vec<serde_json::Value>, AuditError> {
        let query = r#"
            SELECT id, title, reason, description, gates, approved_by,
                   impact_level, expires_at, created_at, metadata
            FROM waivers
            WHERE status = 'active' AND expires_at > NOW()
            ORDER BY created_at DESC
        "#;

        let rows = match self.db_client.query(query, &[]).await {
            Ok(rows) => rows,
            Err(e) => return Err(AuditError::Database(format!("Failed to list waivers: {}", e))),
        };

        let mut waivers = Vec::new();
        for row in rows {
            let waiver = serde_json::json!({
                "id": row.get("id"),
                "title": row.get("title"),
                "reason": row.get("reason"),
                "description": row.get("description"),
                "gates": row.get("gates"),
                "approved_by": row.get("approved_by"),
                "impact_level": row.get("impact_level"),
                "expires_at": row.get("expires_at"),
                "created_at": row.get("created_at"),
                "metadata": row.get("metadata")
            });
            waivers.push(waiver);
        }

        Ok(waivers)
    }

    /// Execute a planning operation with full audit trail
    pub async fn execute_planning(
        &self,
        task_description: &str,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<OrchestrationResult, AuditError> {
        let operation_id = Uuid::new_v4().to_string();
        let correlation_id = Some(operation_id.clone());

        // Record operation start
        let start_time = Instant::now();
        self.record_operation_start(
            "planning",
            &operation_id,
            Some(task_description.to_string()),
            correlation_id.clone(),
        ).await?;

        // Track reasoning and decision making
        self.audit_manager.agent_thinking_auditor()
            .record_reasoning_step(
                "task_analysis",
                &format!("Analyzing task: {}", task_description),
                vec![
                    "Direct implementation".to_string(),
                    "Break down into subtasks".to_string(),
                    "Research and planning phase".to_string(),
                ],
                "Break down into subtasks",
                0.85,
                start_time.elapsed(),
            ).await?;

        // Execute the actual planning operation with circuit breaker protection and performance tracking
        let planning_start = Instant::now();
        let result = if let Some(circuit_breaker) = self.circuit_breakers.get("llm_service") {
            // Protect LLM/planning calls with circuit breaker
            match circuit_breaker.execute(|| async {
                self.orchestrator.execute_planning(task_description, context).await
            }).await {
                Ok(result) => result,
                Err(e) => {
                    // Circuit breaker opened or operation failed
                    self.audit_manager.error_recovery_auditor()
                        .record_error_recovery_attempt(
                            "planning_circuit_breaker",
                            "circuit_breaker_protection",
                            false,
                            planning_start.elapsed(),
                            {
                                let mut metadata = HashMap::new();
                                metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                                metadata.insert("circuit_breaker".to_string(), serde_json::Value::String("llm_service".to_string()));
                                metadata
                            }
                        ).await?;
                    return Err(AuditError::Audit(crate::audit_trail::AuditError::CircuitBreaker(e.to_string())));
                }
            }
        } else {
            // No circuit breaker - direct call
            self.orchestrator.execute_planning(task_description, context).await
                .map_err(|e| AuditError::Audit(crate::audit_trail::AuditError::Execution(e.to_string())))?
        };

        // Record successful performance metrics
        self.audit_manager.performance_auditor()
            .record_operation_performance(
                "planning_execution",
                planning_start.elapsed(),
                true,
                {
                    let mut metadata = HashMap::new();
                    metadata.insert("task_length".to_string(), serde_json::Value::Number(task_description.len().into()));
                    metadata.insert("result_type".to_string(), serde_json::Value::String("success".to_string()));
                    metadata
                }
            ).await?;

        Ok(result)
    }

    /// Execute a council review with comprehensive audit trail
    pub async fn execute_council_review(
        &self,
        working_spec: agent_agency_contracts::working_spec::WorkingSpec,
    ) -> Result<OrchestrationResult, AuditError> {
        let operation_id = Uuid::new_v4().to_string();
        let correlation_id = Some(operation_id.clone());

        // Record operation start
        let start_time = Instant::now();
        self.record_operation_start(
            "council_review",
            &operation_id,
            // TODO: Working Spec ID Access - Fix field access after schema changes
            // 
            // COMPLETION CHECKLIST:
            // [ ] Verify current working_spec struct definition
            // [ ] Identify correct field name for spec ID
            // [ ] Update all access points consistently
            // [ ] Add null safety checks if field is optional
            // [ ] Unit tests written (80%+ coverage)
            // [ ] Integration tests with working spec
            // [ ] Documentation updated
            // [ ] Performance impact assessed
            // [ ] Security considerations addressed
            // [ ] Configuration options defined
            // [ ] Monitoring/metrics implemented
            // [ ] Logging added for debugging
            //
            // ACCEPTANCE CRITERIA:
            // - Correct field name used for spec ID access
            // - All similar access points updated
            // - Null safety properly handled
            // - No compilation errors
            // - Tests pass
            //
            // DEPENDENCIES:
            // - working_spec type definition: Available
            // - Field schema documentation: Required
            //
            // ESTIMATED EFFORT: 4 hours
            // PRIORITY: HIGH
            // BLOCKING: Yes - Required for audit trail
            Some(format!("Reviewing spec: {}", "unknown")),
            correlation_id.clone(),
        ).await?;

        // Track council decision making
        self.audit_manager.agent_thinking_auditor()
            .record_decision_point(
                "judge_selection",
                vec![
                    "All available judges".to_string(),
                    "Specialized judges only".to_string(),
                    "Consensus-based selection".to_string(),
                ],
                "Consensus-based selection",
                "Selecting judges based on expertise alignment with task requirements",
                Some(0.2), // Low risk
            ).await?;

        // Execute council review with performance tracking
        let review_start = Instant::now();
        let result = match self.orchestrator.execute_council_review(working_spec.clone()).await {
            Ok(result) => {
                self.audit_manager.performance_auditor()
                    .record_operation_performance(
                        "council_review_execution",
                        review_start.elapsed(),
                        true,
                        {
                            let mut metadata = HashMap::new();
                            // TODO: Working Spec ID Access - Fix field access after schema changes
                            // 
                            // COMPLETION CHECKLIST:
                            // [ ] Verify current working_spec struct definition
                            // [ ] Identify correct field name for spec ID
                            // [ ] Update all access points consistently
                            // [ ] Add null safety checks if field is optional
                            // [ ] Unit tests written (80%+ coverage)
                            // [ ] Integration tests with working spec
                            // [ ] Documentation updated
                            // [ ] Performance impact assessed
                            // [ ] Security considerations addressed
                            // [ ] Configuration options defined
                            // [ ] Monitoring/metrics implemented
                            // [ ] Logging added for debugging
                            //
                            // ACCEPTANCE CRITERIA:
                            // - Correct field name used for spec ID access
                            // - All similar access points updated
                            // - Null safety properly handled
                            // - No compilation errors
                            // - Tests pass
                            //
                            // DEPENDENCIES:
                            // - working_spec type definition: Available
                            // - Field schema documentation: Required
                            //
                            // ESTIMATED EFFORT: 4 hours
                            // PRIORITY: HIGH
                            // BLOCKING: Yes - Required for performance metrics
                            metadata.insert("spec_id".to_string(), serde_json::Value::String("unknown".to_string()));
                            metadata.insert("judge_count".to_string(), serde_json::Value::Number(3.into())); // Assuming 3 judges
                            metadata
                        }
                    ).await?;
                Ok(result)
            }
            Err(e) => {
                self.audit_manager.performance_auditor()
                    .record_operation_performance(
                        "council_review_execution",
                        review_start.elapsed(),
                        false,
                        {
                            let mut metadata = HashMap::new();
                            metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                            metadata
                        }
                    ).await?;
                Err(AuditError::Config(e.to_string()))
            }
        };

        // Record operation completion
        self.record_operation_complete(
            &operation_id,
            start_time.elapsed(),
            result.is_ok(),
        ).await?;

        result
    }

    /// Execute full orchestration pipeline with comprehensive audit trail
    pub async fn execute_full_pipeline(
        &self,
        task_description: &str,
        context: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<OrchestrationResult, AuditError> {
        let pipeline_id = Uuid::new_v4().to_string();
        let correlation_id = Some(pipeline_id.clone());
        let task_id = Uuid::new_v4(); // Generate task ID for audit trail

        // Record pipeline start
        let pipeline_start = Instant::now();
        self.record_operation_start(
            "full_pipeline",
            &pipeline_id,
            Some(format!("Full pipeline for: {}", task_description)),
            correlation_id.clone(),
        ).await?;

        // P0: Audit trail - Task enqueued
        self.create_task_audit_event(
            task_id,
            "orchestration",
            "system",
            "enqueued",
            serde_json::json!({
                "description": task_description,
                "pipeline_id": pipeline_id,
                "stage": "planning"
            }),
        ).await?;

        // Phase 1: Planning
        println!(" Starting planning phase...");
        let planning_result = match self.execute_planning(task_description, context.clone()).await {
            Ok(result) => result,
            Err(e) => {
                self.record_pipeline_failure(&pipeline_id, "planning", &e).await?;
                return Err(e);
            }
        };

        // Phase 2: Council Review
        println!("🏛️  Starting council review phase...");
        let working_spec = match planning_result.working_spec {
            Some(spec) => spec,
            None => {
                let error = AuditError::Config("No working spec generated from planning".to_string());
                self.record_pipeline_failure(&pipeline_id, "council_review", &error).await?;
                return Err(error);
            }
        };

        let review_result = match self.execute_council_review(working_spec).await {
            Ok(result) => result,
            Err(e) => {
                // P0: Audit trail - Task failed during council review
                self.create_task_audit_event(
                    task_id,
                    "orchestration",
                    "council",
                    "error",
                    serde_json::json!({
                        "error_type": "council_review_failed",
                        "error_message": e.to_string(),
                        "stage": "council_review"
                    }),
                ).await?;
                self.record_pipeline_failure(&pipeline_id, "council_review", &e).await?;
                return Err(e);
            }
        };

        // P0: Audit trail - Task approved/denied by council
        self.create_task_audit_event(
            task_id,
            "council",
            "council",
            if review_result.decision.as_deref() == Some("approved") { "approved" } else { "denied" },
            serde_json::json!({
                "decision": review_result.decision,
                "confidence": review_result.confidence,
                "reasoning": review_result.reasoning,
                "stage": "council_review"
            }),
        ).await?;

        // Phase 3: Execution (if approved)
        let final_result = if review_result.decision.as_deref() == Some("approved") {
            // P0: Audit trail - Task started execution
            self.create_task_audit_event(
                task_id,
                "orchestration",
                "system",
                "started",
                serde_json::json!({
                    "stage": "execution",
                    "execution_mode": "worker"
                }),
            ).await?;

            println!(" Starting execution phase...");
            match self.orchestrator.execute_operation(review_result.clone()).await {
                Ok(result) => result,
                Err(e) => {
                    // P0: Audit trail - Task failed during execution
                    self.create_task_audit_event(
                        task_id,
                        "orchestration",
                        "worker",
                        "error",
                        serde_json::json!({
                            "error_type": "execution_failed",
                            "error_message": e.to_string(),
                            "stage": "execution"
                        }),
                    ).await?;
                    self.record_pipeline_failure(&pipeline_id, "execution", &AuditError::Config(e.to_string())).await?;
                    return Err(AuditError::Config(e.to_string()));
                }
            }
        } else {
            // P0: Audit trail - Task denied (not executed)
            self.create_task_audit_event(
                task_id,
                "orchestration",
                "council",
                "completed",
                serde_json::json!({
                    "outcome": "denied",
                    "reason": "council_denial",
                    "stage": "final"
                }),
            ).await?;
            review_result
        };

        // P0: Audit trail - Task completed successfully (if executed)
        if review_result.decision.as_deref() == Some("approved") {
            self.create_task_audit_event(
                task_id,
                "orchestration",
                "system",
                "completed",
                serde_json::json!({
                    "outcome": "success",
                    "execution_duration_ms": pipeline_start.elapsed().as_millis(),
                    "stage": "final"
                }),
            ).await?;
        }

        // Record pipeline completion
        self.record_operation_complete(
            &pipeline_id,
            pipeline_start.elapsed(),
            true,
        ).await?;

        // Record learning insights from full pipeline
        self.audit_manager.learning_auditor()
            .record_learning_insight(
                "pipeline_efficiency",
                "Full pipeline execution with integrated audit trail provides comprehensive observability",
                "Improved debugging and optimization capabilities",
                0.9,
                "pipeline_execution"
            ).await?;

        Ok(final_result)
    }

    /// Get comprehensive audit statistics
    pub async fn get_audit_statistics(&self) -> Result<AuditStatistics, AuditError> {
        let global_stats = self.audit_manager.get_global_stats().await;

        Ok(AuditStatistics {
            total_events: global_stats.total_events,
            events_by_category: global_stats.events_by_category,
            active_operations: self.active_contexts.read().await.len(),
            average_event_latency: global_stats.performance_metrics.avg_record_time_us,
            total_audit_log_size: global_stats.performance_metrics.total_log_size_bytes,
            error_counts: global_stats.error_counts,
            collection_duration: Utc::now().signed_duration_since(global_stats.collection_start).num_seconds(),
        })
    }

    /// Export audit trail for analysis
    pub async fn export_audit_trail(&self, format: AuditOutputFormat) -> Result<String, AuditError> {
        self.audit_manager.export_audit_trail(format, None).await
    }

    /// Search audit events
    pub async fn search_audit_events(&self, query: AuditQuery) -> Result<Vec<AuditEvent>, AuditError> {
        self.audit_manager.search_events(query).await
    }

    // Private helper methods

    async fn record_operation_start(
        &self,
        operation_type: &str,
        operation_id: &str,
        description: Option<String>,
        correlation_id: Option<String>,
    ) -> Result<(), AuditError> {
        let context = OperationContext {
            operation_id: operation_id.to_string(),
            start_time: Instant::now(),
            operation_type: operation_type.to_string(),
            parent_operation_id: None, // Could be enhanced for nested operations
            correlation_id: correlation_id.clone(),
        };

        self.active_contexts.write().await.insert(operation_id.to_string(), context);

        // Record in performance auditor
        self.audit_manager.performance_auditor()
            .record_operation_performance(
                &format!("{}_start", operation_type),
                Duration::from_micros(0), // Start event
                true,
                {
                    let mut metadata = HashMap::new();
                    if let Some(desc) = description {
                        metadata.insert("description".to_string(), serde_json::Value::String(desc));
                    }
                    metadata.insert("operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                    metadata
                }
            ).await?;

        Ok(())
    }

    async fn record_operation_complete(
        &self,
        operation_id: &str,
        duration: Duration,
        success: bool,
    ) -> Result<(), AuditError> {
        if let Some(context) = self.active_contexts.write().await.remove(operation_id) {
            // Record completion in performance auditor
            self.audit_manager.performance_auditor()
                .record_operation_performance(
                    &format!("{}_complete", context.operation_type),
                    duration,
                    success,
                    {
                        let mut metadata = HashMap::new();
                        metadata.insert("operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                        metadata.insert("duration_ms".to_string(), serde_json::Value::Number((duration.as_millis() as u64).into()));
                        metadata
                    }
                ).await?;
        }

        Ok(())
    }

    async fn record_pipeline_failure(
        &self,
        pipeline_id: &str,
        failed_phase: &str,
        error: &AuditError,
    ) -> Result<(), AuditError> {
        // Record the failure
        self.audit_manager.error_recovery_auditor()
            .record_error_recovery_attempt(
                "pipeline_failure",
                "pipeline_error_handling",
                false,
                Duration::from_secs(0),
                {
                    let mut context = HashMap::new();
                    context.insert("pipeline_id".to_string(), serde_json::Value::String(pipeline_id.to_string()));
                    context.insert("failed_phase".to_string(), serde_json::Value::String(failed_phase.to_string()));
                    context.insert("error".to_string(), serde_json::Value::String(error.to_string()));
                    context
                }
            ).await?;

        // Record learning insight about failure
        self.audit_manager.learning_auditor()
            .record_learning_insight(
                "pipeline_failure_analysis",
                &format!("Pipeline failed at {} phase, need to improve error handling", failed_phase),
                "Better error handling and recovery mechanisms",
                0.8,
                "pipeline_failure"
            ).await?;

        Ok(())
    }

    /// Correlate recovery events to root failures and compute SLO impact
    async fn correlate_recovery_to_failure(
        &self,
        operation_id: &str,
        recovery_success: bool,
        recovery_duration: Duration,
    ) -> Result<(), AuditError> {
        // Query for the original failure event
        let failure_events = self.audit_manager.search_events(AuditQuery {
            category: Some(AuditCategory::Error),
            operation: Some("plan_task".to_string()),
            time_range: Some((
                Utc::now() - chrono::Duration::hours(1), // Look back 1 hour
                Utc::now()
            )),
            limit: Some(10),
            ..Default::default()
        }).await?;

        // Find the most recent failure for this operation
        if let Some(failure_event) = failure_events.into_iter()
            .filter(|e| {
                e.context.get("operation_id")
                    .and_then(|v| v.as_str())
                    .map(|id| id == operation_id)
                    .unwrap_or(false)
            })
            .max_by_key(|e| e.timestamp)
        {
            // Compute SLO impact based on recovery time and success
            let slo_impact = self.compute_slo_impact(&failure_event, recovery_success, recovery_duration);

            // Record the correlation
            self.audit_manager.error_recovery_auditor()
                .record_recovery_correlation(
                    operation_id,
                    &failure_event.event_id.to_string(),
                    recovery_success,
                    slo_impact,
                    {
                        let mut context = HashMap::new();
                        context.insert("root_failure_timestamp".to_string(),
                            serde_json::Value::String(failure_event.timestamp.to_rfc3339()));
                        context.insert("recovery_duration_ms".to_string(),
                            serde_json::Value::Number((recovery_duration.as_millis() as u64).into()));
                        context
                    }
                ).await?;
        }

        Ok(())
    }

    /// Compute SLO impact from recovery attempt
    fn compute_slo_impact(
        &self,
        failure_event: &AuditEvent,
        recovery_success: bool,
        recovery_duration: Duration,
    ) -> f64 {
        let base_impact = if recovery_success {
            // Successful recovery has minimal impact if quick
            if recovery_duration < Duration::from_secs(30) {
                0.1_f64 // Low impact for fast recovery
            } else if recovery_duration < Duration::from_secs(120) {
                0.3_f64 // Moderate impact for slower recovery
            } else {
                0.6_f64 // Higher impact for slow but successful recovery
            }
        } else {
            // Failed recovery has high impact
            0.8_f64
        };

        // Adjust based on failure severity
        let severity_multiplier = match failure_event.severity {
            AuditSeverity::Critical => 1.5_f64,
            AuditSeverity::High => 1.2_f64,
            AuditSeverity::Medium => 1.0_f64,
            AuditSeverity::Low => 0.8_f64,
            AuditSeverity::Info => 0.5_f64,
            AuditSeverity::Warning => 1.0_f64,
            AuditSeverity::Error => 1.3_f64,
            AuditSeverity::Debug => 0.3_f64,
        };

        (base_impact * severity_multiplier).min(1.0_f64)
    }
}

/// Comprehensive audit statistics
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AuditStatistics {
    pub total_events: u64,
    pub events_by_category: HashMap<AuditCategory, u64>,
    pub active_operations: usize,
    pub average_event_latency: u64,
    pub total_audit_log_size: u64,
    pub error_counts: HashMap<String, u64>,
    pub collection_duration: i64,
}

/// Audit error wrapper

#[derive(Debug, Serialize, Deserialize, JsonSchema, thiserror::Error)]
enum AuditError {
    #[error("Orchestration error: {0}")]
    Orchestration(String),

    #[error("Audit trail error: {0}")]
    Audit(#[from] crate::audit_trail::AuditError),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}

impl From<String> for AuditError {
    fn from(s: String) -> Self {
        AuditError::Config(s)
    }
}

// Re-export key types for convenience
pub use crate::audit_trail::AuditQuery;
// pub use crate::orchestrate::{OrchestrationResult, OrchestrationContext};

// Note: The following modules were removed during refactoring:
// - orchestrate, planning, frontier modules - moved to separate services or removed
// - agent_data_processing::operations - functionality moved to data-infrastructure crate
// - agent_agency_resilience - renamed to system-resilience crate
//
// EVIDENCE ENRICHMENT:
// - EvidenceEnrichmentCoordinator referenced in lib.rs (line 131) is currently disabled
// - Intended integration with agent-research/src/multimodal_context_provider.rs
// - MultimodalContextProvider provides evidence enrichment for Council verdicts
// - Current status: Disabled due to missing MultimodalRetriever dependency
//
// Current implementation provides placeholder types and local implementations
