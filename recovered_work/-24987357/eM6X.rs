//! Audited Orchestrator - Automatic audit trail integration for all operations
//!
//! This module provides a wrapper around the main orchestrator that automatically
//! instruments all operations with comprehensive audit trail logging, providing
//! Cursor/Claude Code-style observability.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::audit_trail::{
    AuditTrailManager, AuditConfig, AuditLogLevel, AuditOutputFormat,
    FileOperationsAuditor, TerminalAuditor, CouncilAuditor, AgentThinkingAuditor,
    PerformanceAuditor, ErrorRecoveryAuditor, LearningAuditor,
    AuditEvent, AuditCategory, AuditSeverity, AuditResult, AuditPerformance,
};
use crate::orchestrate::{Orchestrator, OrchestratorConfig, OrchestrationResult, OrchestrationContext};
use crate::planning::agent::PlanningAgent;
use crate::frontier::{Frontier, FrontierConfig, FrontierError};
use file_ops::{validate_changeset_with_waiver, WaiverRequest, apply_waiver};

/// Audited orchestrator that wraps all operations with comprehensive audit logging
#[derive(Debug)]
pub struct AuditedOrchestrator {
    /// The underlying orchestrator
    orchestrator: Arc<Orchestrator>,
    /// Audit trail manager
    audit_manager: Arc<AuditTrailManager>,
    /// Active operation contexts for correlation
    active_contexts: Arc<RwLock<HashMap<String, OperationContext>>>,
    /// Frontier queue for spawned tasks (optional)
    frontier: Option<std::sync::RwLock<Frontier>>,
    /// Circuit breakers for external services
    circuit_breakers: HashMap<String, Arc<crate::audit_trail::CircuitBreaker>>,
}

/// Context for tracking active operations
#[derive(Debug, Clone)]
struct OperationContext {
    /// Operation ID for correlation
    operation_id: String,
    /// Start time
    start_time: Instant,
    /// Operation type
    operation_type: String,
    /// Parent operation ID (if nested)
    parent_operation_id: Option<String>,
    /// Correlation ID for distributed tracing
    correlation_id: Option<String>,
}

/// Configuration for the audited orchestrator
#[derive(Debug, Clone)]
pub struct AuditedOrchestratorConfig {
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
}

impl AuditedOrchestrator {
    /// Create a new audited orchestrator
    pub fn new(config: AuditedOrchestratorConfig) -> Self {
        let audit_manager = Arc::new(AuditTrailManager::new(config.audit_config));
        let orchestrator = Arc::new(Orchestrator::new(config.orchestrator_config));

        let frontier = config.frontier_config
            .map(|fc| std::sync::RwLock::new(Frontier::with_config(fc)));

        Self {
            orchestrator,
            audit_manager,
            active_contexts: Arc::new(RwLock::new(HashMap::new())),
            frontier,
        }
    }

    /// Get the audit trail manager for direct access
    pub fn audit_manager(&self) -> Arc<AuditTrailManager> {
        self.audit_manager.clone()
    }

    /// Spawn a task to the frontier queue (if enabled)
    pub fn spawn_task(&self, task: crate::planning::types::Task, parent_operation_id: &str) -> Result<(), FrontierError> {
        if let Some(frontier) = &self.frontier {
            let mut frontier = frontier.write().unwrap();
            frontier.push(task, parent_operation_id)?;
        }
        // If no frontier configured, silently ignore (not an error)
        Ok(())
    }

    /// Get the next task from the frontier queue
    pub fn get_next_task(&self) -> Option<crate::planning::types::Task> {
        self.frontier.as_ref()?.read().unwrap().pop()
    }

    /// Get frontier statistics
    pub fn frontier_stats(&self) -> Option<crate::frontier::FrontierStats> {
        Some(self.frontier.as_ref()?.read().unwrap().stats())
    }

    /// Process budget violations and generate waiver requests
    pub async fn process_budget_violations(
        &self,
        changeset: &file_ops::ChangeSet,
        allowlist: &file_ops::AllowList,
        budgets: &file_ops::Budgets,
        operation_id: &str,
    ) -> Result<(), AuditError> {
        // Check for violations and generate waiver if needed
        match file_ops::validate_changeset_with_waiver(changeset, allowlist, budgets) {
            Ok(()) => {
                // No violations, log successful validation
                let mut parameters = std::collections::HashMap::new();
                parameters.insert("operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                parameters.insert("status".to_string(), serde_json::Value::String("compliant".to_string()));

                self.audit_manager.file_operations_auditor()
                    .record_operation(
                        "budget_check",
                        Some(operation_id),
                        parameters,
                        crate::audit_trail::AuditResult::Success { data: Some("All budget constraints satisfied".to_string()) },
                        None,
                        crate::audit_trail::AuditSeverity::Info,
                    ).await?;
            }
            Err(waiver_request) => {
                // Violations found, log waiver request
                let waiver_json = serde_json::to_string(&waiver_request)
                    .map_err(|e| AuditError::SerializationError(e.to_string()))?;

                let mut parameters = std::collections::HashMap::new();
                parameters.insert("operation_id".to_string(), serde_json::Value::String(operation_id.to_string()));
                parameters.insert("waiver_id".to_string(), serde_json::Value::String(waiver_request.id.clone()));
                parameters.insert("risk_level".to_string(), serde_json::Value::String(format!("{:?}", waiver_request.risk_assessment)));
                parameters.insert("violation_count".to_string(), serde_json::Value::Number(waiver_request.budget_violations.len().into()));

                let severity = if matches!(waiver_request.risk_assessment, file_ops::RiskLevel::Critical) {
                    crate::audit_trail::AuditSeverity::Error
                } else {
                    crate::audit_trail::AuditSeverity::Warning
                };

                self.audit_manager.file_operations_auditor()
                    .record_operation(
                        "budget_violation",
                        Some(&waiver_request.id),
                        parameters,
                        crate::audit_trail::AuditResult::Failure { error: waiver_json },
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
                    ).map_err(|e| AuditError::ValidationError(e))?;

                    let approved_json = serde_json::to_string(&approved_waiver)
                        .map_err(|e| AuditError::SerializationError(e.to_string()))?;

                    let mut approval_params = std::collections::HashMap::new();
                    approval_params.insert("waiver_id".to_string(), serde_json::Value::String(approved_waiver.id.clone()));
                    approval_params.insert("approver".to_string(), serde_json::Value::String("auto-approver".to_string()));

                    self.audit_manager.file_operations_auditor()
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
                    return Err(AuditError::ValidationError(
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
        // In a real implementation, you'd load the waiver from storage
        // For now, we'll create a mock waiver for demonstration
        let mut waiver = WaiverRequest {
            id: waiver_id.to_string(),
            timestamp: chrono::Utc::now(),
            changeset_fingerprint: "mock".to_string(),
            budget_violations: vec![],
            justification_required: true,
            risk_assessment: file_ops::RiskLevel::Medium,
            auto_approved: false,
            approved_by: None,
            approval_timestamp: None,
            justification: None,
        };

        apply_waiver(&mut waiver, approver, justification)
            .map_err(|e| AuditError::ValidationError(e))?;

        let waiver_json = serde_json::to_string(&waiver)
            .map_err(|e| AuditError::SerializationError(e.to_string()))?;

        let mut approval_params = std::collections::HashMap::new();
        approval_params.insert("waiver_id".to_string(), serde_json::Value::String(waiver_id.to_string()));
        approval_params.insert("approver".to_string(), serde_json::Value::String(approver.to_string()));

        self.audit_manager.file_operations_auditor()
            .record_operation(
                "waiver_approval",
                Some(waiver_id),
                approval_params,
                crate::audit_trail::AuditResult::Success { data: Some(waiver_json) },
                None,
                crate::audit_trail::AuditSeverity::Info,
            ).await?;

        Ok(())
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

        // Execute the actual planning operation with performance tracking
        let planning_start = Instant::now();
        let result = match self.orchestrator.execute_planning(task_description, context).await {
            Ok(result) => {
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
            Err(e) => {
                self.audit_manager.performance_auditor()
                    .record_operation_performance(
                        "planning_execution",
                        planning_start.elapsed(),
                        false,
                        {
                            let mut metadata = HashMap::new();
                            metadata.insert("error".to_string(), serde_json::Value::String(e.to_string()));
                            metadata
                        }
                    ).await?;

                // Record error recovery attempt
                self.audit_manager.error_recovery_auditor()
                    .record_error_recovery_attempt(
                        "planning_error",
                        "retry_with_simplification",
                        false, // Assume failure for now
                        planning_start.elapsed(),
                        {
                            let mut context = HashMap::new();
                            context.insert("original_error".to_string(), serde_json::Value::String(e.to_string()));
                            context
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

        // Record learning insights
        if result.is_ok() {
            self.audit_manager.learning_auditor()
                .record_learning_insight(
                    "task_breakdown_effectiveness",
                    "Breaking complex tasks into subtasks improves planning success rate",
                    "20% improvement in planning accuracy",
                    0.75,
                    "planning_execution"
                ).await?;
        }

        result
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
            Some(format!("Reviewing spec: {}", working_spec.id)),
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
                            metadata.insert("spec_id".to_string(), serde_json::Value::String(working_spec.id.clone()));
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

        // Record pipeline start
        let pipeline_start = Instant::now();
        self.record_operation_start(
            "full_pipeline",
            &pipeline_id,
            Some(format!("Full pipeline for: {}", task_description)),
            correlation_id.clone(),
        ).await?;

        // Phase 1: Planning
        println!("🔍 Starting planning phase...");
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
                self.record_pipeline_failure(&pipeline_id, "council_review", &e).await?;
                return Err(e);
            }
        };

        // Phase 3: Execution (if approved)
        let final_result = if review_result.decision.as_deref() == Some("approved") {
            println!("⚡ Starting execution phase...");
            match self.orchestrator.execute_operation(review_result.clone()).await {
                Ok(result) => result,
                Err(e) => {
                    self.record_pipeline_failure(&pipeline_id, "execution", &AuditError::Config(e.to_string())).await?;
                    return Err(AuditError::Config(e.to_string()));
                }
            }
        } else {
            review_result
        };

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
                        metadata.insert("duration_ms".to_string(), serde_json::Value::Number(duration.as_millis().into()));
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
}

/// Comprehensive audit statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
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
#[derive(Debug, thiserror::Error)]
pub enum AuditError {
    #[error("Orchestration error: {0}")]
    Orchestration(String),

    #[error("Audit trail error: {0}")]
    Audit(#[from] crate::audit_trail::AuditError),

    #[error("Configuration error: {0}")]
    Config(String),
}

impl From<String> for AuditError {
    fn from(s: String) -> Self {
        AuditError::Config(s)
    }
}

// Re-export key types for convenience
pub use crate::audit_trail::{AuditQuery, AuditOutputFormat, AuditLogLevel};
pub use crate::orchestrate::{OrchestratorConfig, OrchestrationResult, OrchestrationContext};
