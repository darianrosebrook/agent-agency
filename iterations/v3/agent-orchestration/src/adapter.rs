//! Adapter layer for legacy orchestration functionality
//!
//! This module provides compatibility between the old monolithic orchestration
//! system and the current modular agent-orchestration architecture.
//!
//! @author @darianrosebrook

use crate::types::{
    TaskScope, ChangeBudget, BlastRadius, OrchestratorConfig, TaskExecutionResult,
    ExecutionArtifacts, ExecutionStatus, TaskDescriptor, WorkingSpec, DiffStats,
    AcceptanceCriterion, TaskPriority
};
use crate::council::{Council, CouncilConfig, CouncilSession};
use crate::multimodal_orchestration::{MultimodalOrchestrator, ProcessingResult};
use crate::audit_trail::{AuditTrailManager, AuditConfig};
use anyhow::{Context, Result};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Legacy orchestrator adapter that bridges old and new architectures
#[derive(Debug)]
pub struct LegacyOrchestratorAdapter {
    /// Council for decision making
    council: Arc<Council>,
    /// Multimodal orchestrator for task execution
    orchestrator: Arc<MultimodalOrchestrator>,
    /// Audit trail manager
    audit_trail: Arc<AuditTrailManager>,
    /// Configuration
    config: OrchestratorConfig,
}

impl LegacyOrchestratorAdapter {
    /// Create a new legacy orchestrator adapter
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        let council_config = CouncilConfig::default();
        let council = Arc::new(Council::new(council_config).await?);
        
        let orchestrator_config = OrchestratorConfig::default();
        let orchestrator = Arc::new(MultimodalOrchestrator::new(orchestrator_config)?);
        
        let audit_config = AuditConfig::default();
        let audit_trail = Arc::new(AuditTrailManager::new(audit_config));

        Ok(Self {
            council,
            orchestrator,
            audit_trail,
            config,
        })
    }

    /// Main orchestration function that coordinates the entire task execution process
    /// This adapts the old `orchestrate_task` function to work with the new architecture
    pub async fn orchestrate_task(
        &self,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
        diff: &DiffStats,
        tests_added: bool,
        deterministic: bool,
    ) -> Result<TaskExecutionResult> {
        debug!(
            task_id = %desc.task_id,
            "Starting orchestration for task: {}",
            desc.task_id
        );

        // Step 1: Validate the task
        let validation_result = self.validate_orchestration_task(spec, desc, diff, tests_added, deterministic).await?;
        
        if let Some(short_circuit_result) = self.build_short_circuit_verdict(&validation_result) {
            warn!(
                task_id = %desc.task_id,
                "Validation produced short-circuit verdict: {:?}",
                short_circuit_result
            );
            return Ok(short_circuit_result);
        }

        // Step 2: Evaluate task with council
        let council_session = self.council.start_session().await?;
        let consensus_result = council_session.review_task(&self.to_orchestrated_task(desc)).await?;

        if !consensus_result.approved {
            return Err(anyhow::anyhow!("Council rejected task: {}", consensus_result.reason));
        }

        // Step 3: Execute task with orchestrator
        let artifacts = self.execute_task_with_orchestrator(spec, desc).await?;
        
        // Step 4: Review artifacts (simplified for now)
        let artifact_verdict = self.review_artifacts(&artifacts, spec, desc).await?;

        // Step 5: Combine verdicts and create final result
        let final_result = self.combine_verdicts(consensus_result, artifact_verdict, artifacts);

        // Step 6: Record audit trail
        self.audit_trail.record_execution(&final_result).await?;

        info!(
            task_id = %desc.task_id,
            "Orchestration completed for task: {}",
            desc.task_id
        );

        Ok(final_result)
    }

    /// Validate orchestration task
    async fn validate_orchestration_task(
        &self,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
        diff: &DiffStats,
        tests_added: bool,
        deterministic: bool,
    ) -> Result<ValidationResult> {
        debug!("Validating orchestration task: {}", desc.task_id);

        // Check change budget constraints
        if diff.files_changed > desc.change_budget.max_files {
            return Ok(ValidationResult::BudgetExceeded {
                files_changed: diff.files_changed,
                max_files: desc.change_budget.max_files,
            });
        }

        if diff.lines_added + diff.lines_modified > desc.change_budget.max_loc {
            return Ok(ValidationResult::BudgetExceeded {
                files_changed: diff.files_changed,
                max_files: desc.change_budget.max_files,
            });
        }

        // Check scope constraints
        if !self.validate_scope(&desc.scope_in, diff) {
            return Ok(ValidationResult::ScopeViolation);
        }

        // Check risk tier constraints
        if spec.risk_tier > 3 || spec.risk_tier < 1 {
            return Ok(ValidationResult::InvalidRiskTier);
        }

        Ok(ValidationResult::Valid)
    }

    /// Validate task scope
    fn validate_scope(&self, scope: &TaskScope, diff: &DiffStats) -> bool {
        // Simplified scope validation - in a real implementation,
        // this would check if changed files are within the scope
        !scope.in_scope.is_empty()
    }

    /// Build short-circuit verdict for validation failures
    fn build_short_circuit_verdict(&self, validation: &ValidationResult) -> Option<TaskExecutionResult> {
        match validation {
            ValidationResult::Valid => None,
            ValidationResult::BudgetExceeded { .. } => Some(TaskExecutionResult {
                working_spec: None,
                artifacts: ExecutionArtifacts {
                    execution_id: "short-circuit".to_string(),
                    worker_id: "validation".to_string(),
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some("Change budget exceeded".to_string()),
                },
                quality_report: None,
            }),
            ValidationResult::ScopeViolation => Some(TaskExecutionResult {
                working_spec: None,
                artifacts: ExecutionArtifacts {
                    execution_id: "short-circuit".to_string(),
                    worker_id: "validation".to_string(),
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some("Scope violation detected".to_string()),
                },
                quality_report: None,
            }),
            ValidationResult::InvalidRiskTier => Some(TaskExecutionResult {
                working_spec: None,
                artifacts: ExecutionArtifacts {
                    execution_id: "short-circuit".to_string(),
                    worker_id: "validation".to_string(),
                    status: ExecutionStatus::Failed,
                    output: None,
                    error: Some("Invalid risk tier".to_string()),
                },
                quality_report: None,
            }),
        }
    }

    /// Execute task with orchestrator
    async fn execute_task_with_orchestrator(
        &self,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
    ) -> Result<ExecutionArtifacts> {
        debug!("Executing task with orchestrator: {}", desc.task_id);

        // TODO: Convert to multimodal task format
        // let multimodal_task = crate::multimodal_orchestration::MultimodalTask {
        //     id: desc.task_id.clone(),
        //     description: desc.description.clone(),
        //     requirements: vec![], // Simplified for now
        //     priority: self.convert_priority(desc.priority),
        // };

        // TODO: Execute with multimodal orchestrator
        // let result = self.orchestrator.process_task(multimodal_task).await?;
        // Ok(ExecutionArtifacts {
        //     execution_id: uuid::Uuid::new_v4().to_string(),
        //     worker_id: "multimodal-orchestrator".to_string(),
        //     status: self.convert_status(result.status),
        //     output: Some(result.output),
        //     error: result.error,
        // })
        Ok(ExecutionArtifacts {
            execution_id: uuid::Uuid::new_v4().to_string(),
            worker_id: "multimodal-orchestrator".to_string(),
            status: ExecutionStatus::Failed,
            output: None,
            error: Some("Multimodal orchestration not yet implemented".to_string()),
        })
    }

    /// Review artifacts with judges
    async fn review_artifacts(
        &self,
        artifacts: &ExecutionArtifacts,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
    ) -> Result<ArtifactVerdict> {
        debug!("Reviewing artifacts for task: {}", desc.task_id);

        // Simplified artifact review - in a real implementation,
        // this would use the council's judge system
        let confidence = match artifacts.status {
            ExecutionStatus::Completed => 0.9,
            ExecutionStatus::Failed => 0.1,
            _ => 0.5,
        };

        Ok(ArtifactVerdict {
            approved: artifacts.status == ExecutionStatus::Completed,
            confidence,
            reasoning: format!("Artifact review for task {}", desc.task_id),
        })
    }

    /// Combine verdicts from council and artifact review
    fn combine_verdicts(
        &self,
        _council_result: (), // TODO: crate::council::CouncilDecision - type not yet defined
        artifact_verdict: ArtifactVerdict,
        artifacts: ExecutionArtifacts,
    ) -> TaskExecutionResult {
        let overall_approved = true && artifact_verdict.approved; // TODO: council_result.approved
        let overall_confidence = (0.8 + artifact_verdict.confidence) / 2.0; // TODO: council_result.confidence

        TaskExecutionResult {
            working_spec: Some(format!("Combined verdict for task")),
            artifacts,
            quality_report: Some(crate::types::QualityReport {
                score: overall_confidence,
                metrics: std::collections::HashMap::new(),
                recommendations: vec![],
            }),
        }
    }

    /// Convert task descriptor to orchestrated task format
    fn to_orchestrated_task(&self, desc: &TaskDescriptor) -> crate::OrchestratedTask {
        crate::OrchestratedTask {
            id: desc.task_id.clone(),
            description: desc.description.clone(),
            requirements: vec![], // Simplified for now
            priority: self.convert_priority(desc.priority),
        }
    }

    /// Convert task priority
    fn convert_priority(&self, priority: TaskPriority) -> crate::multimodal_orchestration::ProcessingPriority {
        match priority {
            TaskPriority::Low => crate::multimodal_orchestration::ProcessingPriority::Low,
            TaskPriority::Medium => crate::multimodal_orchestration::ProcessingPriority::Normal,
            TaskPriority::High => crate::multimodal_orchestration::ProcessingPriority::High,
            TaskPriority::Critical => crate::multimodal_orchestration::ProcessingPriority::Critical,
        }
    }

    /// Convert execution status
    fn convert_status(&self, status: crate::multimodal_orchestration::ProcessingStatus) -> ExecutionStatus {
        match status {
            crate::multimodal_orchestration::ProcessingStatus::Pending => ExecutionStatus::Pending,
            crate::multimodal_orchestration::ProcessingStatus::InProgress => ExecutionStatus::Running,
            crate::multimodal_orchestration::ProcessingStatus::Running => ExecutionStatus::Running,
            crate::multimodal_orchestration::ProcessingStatus::Skipped => ExecutionStatus::Cancelled,
            crate::multimodal_orchestration::ProcessingStatus::Cancelled => ExecutionStatus::Cancelled,
            crate::multimodal_orchestration::ProcessingStatus::Completed => ExecutionStatus::Completed,
            crate::multimodal_orchestration::ProcessingStatus::Failed => ExecutionStatus::Failed,
        }
    }
}

/// Validation result for orchestration tasks
#[derive(Debug, Clone)]
pub enum ValidationResult {
    Valid,
    BudgetExceeded {
        files_changed: u32,
        max_files: u32,
    },
    ScopeViolation,
    InvalidRiskTier,
}

/// Artifact verdict from review process
#[derive(Debug, Clone)]
pub struct ArtifactVerdict {
    pub approved: bool,
    pub confidence: f32,
    pub reasoning: String,
}
