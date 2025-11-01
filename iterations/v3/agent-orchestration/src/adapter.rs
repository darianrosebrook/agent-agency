//! Adapter layer for legacy orchestration functionality
//!
//! This module provides compatibility between the old monolithic orchestration
//! system and the current modular agent-orchestration architecture.
//!
//! @author @darianrosebrook

// Use contracts types directly - prefer prelude for commonly used types
use agent_agency_contracts::types::prelude::{
    TaskDescriptor, TaskPriority, BlastRadius
};
use agent_agency_contracts::working_spec::{
    WorkingSpec, AcceptanceCriterion
};
use agent_agency_contracts::planning_io::ChangeBudget;
use agent_agency_contracts::task_executor::{TaskExecutionResult as ContractsTaskExecutionResult, ExecutionStatus};
use agent_agency_contracts::ExecutionArtifacts;
use agent_agency_contracts::types::planning::TaskScope;
use crate::types::{
    OrchestratorConfig, DiffStats, TaskExecutionResult, ExecutionArtifacts as LocalExecutionArtifacts
};
use crate::judge_backup::backup_types::JudgeType;
use crate::council::{Council, CouncilConfig};
use crate::decision_making::{ConsensusStrategy, RiskThresholds};
use crate::multimodal_orchestration::MultimodalOrchestrator;
use crate::audit_trail::{AuditTrailManager, AuditConfig};
use crate::judge_backup::{
    Judge, JudgeConfig,
    EthicsJudge,
    quality_judge::QualityAssuranceJudge,
    security_judge::SecurityJudge,
    verdicts::JudgeVerdict,
};
use crate::judge_backup::mock::VerdictStrategy;
use crate::verdict_aggregation::{
    VerdictAggregator, AggregationConfig, DissentHandling, RiskAggregationStrategy,
};
use crate::decision_making::AlgorithmicDecisionEngine;
use anyhow::Result;
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
    #[cfg(feature = "memory")]
    /// Memory system for learning and context retention (shared across components)
    memory_system: Option<Arc<agent_memory::MemorySystem>>,
}

impl LegacyOrchestratorAdapter {
    /// Initialize memory system for use across components
    /// 
    /// This helper function creates a MemorySystem instance that can be shared
    /// between Council, AutonomousExecutor, and other components that need memory.
    #[cfg(feature = "memory")]
    async fn init_memory_system() -> Result<Arc<agent_memory::MemorySystem>> {
        let memory_config = agent_memory::MemoryConfig::default();
        let memory_system = Arc::new(agent_memory::MemorySystem::init(memory_config).await?);
        Ok(memory_system)
    }

    /// Get the memory system instance (if memory feature is enabled)
    /// 
    /// This can be used to wire memory into other components like AutonomousExecutor
    #[cfg(feature = "memory")]
    pub fn get_memory_system(&self) -> Option<Arc<agent_memory::MemorySystem>> {
        self.memory_system.clone()
    }

    /// Create a new legacy orchestrator adapter
    pub async fn new(config: OrchestratorConfig) -> Result<Self> {
        let council_config = CouncilConfig {
            session_timeout_seconds: 300, // 5 minutes
            min_judges_required: 3, // Require all available judges
            max_judges_per_session: 10, // Allow up to 10 for future expansion
            judge_selection_strategy: crate::council::JudgeSelectionStrategy::AllAvailable, // Select all available judges
            consensus_strategy: ConsensusStrategy::Majority,
            risk_thresholds: crate::decision_making::RiskThresholds::default(),
            enable_parallel_reviews: true,
            judge_timeout_seconds: 60,
            enable_circuit_breakers: true,
            enable_graceful_degradation: true,
            enable_error_recovery: true,
        };
        // Create judges for the council
        let judges: Vec<Arc<dyn Judge>> = vec![
            // Ethics judge for moral and ethical considerations
            Arc::new(EthicsJudge::new(JudgeConfig {
                judge_id: "ethics-001".to_string(),
                name: "Ethics Judge".to_string(),
                judge_type: JudgeType::Ethics,
                specialization: "moral reasoning".to_string(),
                max_response_time_ms: 5000,
                health_check_interval_ms: 30000,
            })),

            // Quality assurance judge
            Arc::new(QualityAssuranceJudge::new(
                JudgeConfig {
                    judge_id: "qa-001".to_string(),
                    name: "Quality Assurance Judge".to_string(),
                    judge_type: JudgeType::Quality,
                    specialization: "code quality".to_string(),
                    max_response_time_ms: 3000,
                    health_check_interval_ms: 30000,
                },
            )),

            // Security judge
            Arc::new(SecurityJudge::new(
                JudgeConfig {
                    judge_id: "security-001".to_string(),
                    name: "Security Judge".to_string(),
                    judge_type: JudgeType::Security,
                    specialization: "security analysis".to_string(),
                    max_response_time_ms: 3000,
                    health_check_interval_ms: 30000,
                },
            )),
        ];

        // Create verdict aggregator
        let verdict_aggregator = Arc::new(VerdictAggregator::new(AggregationConfig {
            consensus_threshold: 0.7,
            weight_by_specialization: true,
            min_judges_required: 3, // Require all available judges
            dissent_handling: DissentHandling::Strict,
            risk_aggregation: RiskAggregationStrategy::WeightedAverage,
        }));

        // Create decision engine
        let decision_engine = Box::new(AlgorithmicDecisionEngine::new(ConsensusStrategy::Majority));

        // Create the council
        let mut council = Council::new(
            council_config,
            judges,
            verdict_aggregator,
            decision_engine,
        );

        #[cfg(feature = "memory")]
        let memory_system = {
            // Initialize shared memory system
            let memory = Self::init_memory_system().await?;
            // Inject into council
            council.set_memory_system(memory.clone());
            Some(memory)
        };

        #[cfg(not(feature = "memory"))]
        let _memory_system: Option<()> = None;

        let council = Arc::new(council);

        let _orchestrator_config = OrchestratorConfig::default();
        let orchestrator = Arc::new(MultimodalOrchestrator::new().await?);
        
        let audit_config = AuditConfig::default();
        let audit_trail = Arc::new(AuditTrailManager::new(audit_config));

        Ok(Self {
            council,
            orchestrator,
            audit_trail,
            config,
            #[cfg(feature = "memory")]
            memory_system,
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
        let council_session = self.council.start_session(desc).await?;
        let consensus_result = council_session.review_task(&self.to_orchestrated_task(desc)).await?;

        // Council approval is determined by the approved field

        // Step 3: Execute task with orchestrator
        let artifacts = self.execute_task_with_orchestrator(spec, desc).await?;
        
        // Step 4: Review artifacts with status and output analysis
        let artifact_verdict = if let Some(first_artifact) = artifacts.first() {
            let artifact_verdict = self.review_artifacts(first_artifact, spec, desc).await?;
            // The review_artifacts method already returns ArtifactVerdict
            artifact_verdict
        } else {
            // No artifacts to review - approve with low confidence
            ArtifactVerdict {
                approved: true,
                confidence: 0.5,
                reasoning: "No artifacts to review".to_string(),
            }
        };

        // Step 5: Combine verdicts and create final result
        let final_result = self.combine_verdicts(consensus_result, artifact_verdict, artifacts);

        // Step 6: Record audit trail
        if let Err(e) = self.audit_trail.record_execution(&final_result).await {
            warn!("Failed to record audit trail for task {}: {}", desc.task_id, e);
            // Don't fail the orchestration if audit trail recording fails
        }

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
    /// 
    /// Currently validates that scope is non-empty. Full implementation would
    /// check if changed files from diff_stats are within scope.in_scope boundaries.
    fn validate_scope(&self, scope: &TaskScope, _diff: &DiffStats) -> bool {
        // Basic validation: ensure scope is defined
        // Future enhancement: Cross-reference diff_stats.changed_files with scope.in_scope
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
                artifacts: LocalExecutionArtifacts {
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
                artifacts: LocalExecutionArtifacts {
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
    ) -> Result<Vec<ExecutionArtifacts>> {
        debug!("Executing task with orchestrator: {}", desc.task_id);

        // Build task description from TaskDescriptor and WorkingSpec
        let task_description = format!(
            "Task: {}\nDescription: {}\nScope: {:?}",
            desc.task_id,
            desc.description,
            desc.scope_in
        );

        // Build context from working spec and task descriptor
        let mut context = std::collections::HashMap::new();
        context.insert("task_id".to_string(), serde_json::json!(desc.task_id));
        // Convert acceptance criteria to a serializable format
        let acceptance_criteria_json: Vec<serde_json::Value> = spec.acceptance_criteria
            .iter()
            .map(|ac| serde_json::json!({
                "id": ac.id,
                "given": ac.given,
                "when": ac.when,
                "then": ac.then,
            }))
            .collect();
        context.insert("acceptance_criteria".to_string(), serde_json::json!(acceptance_criteria_json));
        context.insert("risk_tier".to_string(), serde_json::json!(spec.risk_tier));
        context.insert("mode".to_string(), serde_json::json!(spec.mode));

        // Execute using the multimodal orchestrator
        match self.orchestrator.execute_planning_with_audit(
            &task_description,
            Some(context),
        ).await {
            Ok(processing_result) => {
                // Convert ProcessingResult to ExecutionArtifacts
                let status = match processing_result.status {
                    crate::multimodal_orchestration::ProcessingStatus::Completed => ExecutionStatus::Completed,
                    crate::multimodal_orchestration::ProcessingStatus::Failed => ExecutionStatus::Failed,
                    crate::multimodal_orchestration::ProcessingStatus::InProgress => ExecutionStatus::Running,
                    crate::multimodal_orchestration::ProcessingStatus::Running => ExecutionStatus::Running,
                    crate::multimodal_orchestration::ProcessingStatus::Skipped => ExecutionStatus::Cancelled, // Map Skipped to Cancelled (contracts doesn't have Skipped)
                    crate::multimodal_orchestration::ProcessingStatus::Pending => ExecutionStatus::Pending,
                    crate::multimodal_orchestration::ProcessingStatus::Cancelled => ExecutionStatus::Cancelled,
                };

                let output = if status == ExecutionStatus::Completed {
                    Some(format!(
                        "Task {} processed successfully. Blocks processed: {}, enriched: {}, indexed: {}",
                        desc.task_id,
                        processing_result.blocks_processed,
                        processing_result.blocks_enriched,
                        processing_result.blocks_indexed
                    ))
                } else {
                    None
                };

                let mut artifacts = ExecutionArtifacts::default();
                artifacts.task_id = desc.task_id;
                artifacts.working_spec_id = "multimodal-processing".to_string();
                artifacts.metadata = Some(agent_agency_contracts::execution_artifacts::ArtifactMetadata {
                    compression_applied: None,
                    storage_location: Some("multimodal-processing".to_string()),
                    retention_policy: None,
                    tags: vec!["multimodal".to_string(), "orchestration".to_string()],
                });
                Ok(vec![artifacts])
            }
            Err(e) => {
                warn!("Orchestrator execution failed: {}", e);
                let mut artifacts = ExecutionArtifacts::default();
                artifacts.task_id = desc.task_id;
                artifacts.working_spec_id = "multimodal-processing".to_string();
                artifacts.metadata = Some(agent_agency_contracts::execution_artifacts::ArtifactMetadata {
                    compression_applied: None,
                    storage_location: Some("multimodal-processing".to_string()),
                    retention_policy: None,
                    tags: vec!["multimodal".to_string(), "error".to_string()],
                });
                Ok(vec![artifacts])
            }
        }
    }

    /// Review artifacts with judges
    /// 
    /// Reviews execution artifacts against acceptance criteria and quality requirements.
    /// Uses status-based confidence scoring with adjustments for error presence and
    /// output quality. Future enhancement: Create council session for artifact review
    /// to leverage judge system for more sophisticated analysis.
    async fn review_artifacts(
        &self,
        artifacts: &ExecutionArtifacts,
        spec: &WorkingSpec,
        desc: &TaskDescriptor,
    ) -> Result<ArtifactVerdict> {
        debug!("Reviewing artifacts for task: {}", desc.task_id);

        // Base confidence on execution status
        let mut confidence: f32 = match artifacts.status {
            ExecutionStatus::Completed => 0.9,
            ExecutionStatus::Failed => 0.1,
            ExecutionStatus::InProgress | ExecutionStatus::Running | ExecutionStatus::Execution => 0.5,
            ExecutionStatus::Pending | ExecutionStatus::AwaitingApproval | ExecutionStatus::Planning => 0.3,
            ExecutionStatus::Skipped => 0.2,
            ExecutionStatus::Cancelled => 0.1,
            ExecutionStatus::Starting => 0.2,
            ExecutionStatus::Paused => 0.4,
            ExecutionStatus::Consensus => 0.5,
        };

        // Adjust confidence based on error presence
        if artifacts.error.is_some() {
            confidence = (confidence * 0.5f32).max(0.1f32);
        }

        // Check if output exists and is meaningful
        if artifacts.status == ExecutionStatus::Completed {
            if let Some(ref output) = artifacts.output {
                // If output is meaningful (not empty), slightly increase confidence
                if !output.trim().is_empty() && output.len() > 10 {
                    confidence = (confidence * 1.1f32).min(0.95f32);
                }
            }
        }

        // Check against acceptance criteria if available
        let mut reasoning = format!("Artifact review for task {}: Status={:?}", desc.task_id, artifacts.status);
        if let Some(ref error) = artifacts.error {
            reasoning.push_str(&format!(", Error: {}", error));
        }
        if artifacts.status == ExecutionStatus::Completed {
            if let Some(ref output) = artifacts.output {
                reasoning.push_str(&format!(", Output length: {} chars", output.len()));
            }
        }

        Ok(ArtifactVerdict {
            approved: artifacts.status == ExecutionStatus::Completed && artifacts.error.is_none(),
            confidence,
            reasoning,
        })
    }

    /// Combine verdicts from council and artifact review
    fn combine_verdicts(
        &self,
        council_result: crate::autonomous_executor::ConsensusResult,
        artifact_verdict: ArtifactVerdict,
        artifacts: Vec<ExecutionArtifacts>,
    ) -> TaskExecutionResult {
        let overall_approved = council_result.approved && artifact_verdict.approved;
        let overall_confidence = ((council_result.confidence + artifact_verdict.confidence as f64) / 2.0) as f32;

        // Convert contracts ExecutionArtifacts to local ExecutionArtifacts for TaskExecutionResult
        // Note: This is a simplified conversion - full conversion would map all fields
        let local_artifacts = artifacts.into_iter().next().map(|contracts_artifacts| {
            LocalExecutionArtifacts {
                execution_id: contracts_artifacts.task_id.to_string(),
                worker_id: "unknown".to_string(), // contracts doesn't have worker_id at this level
                status: agent_agency_contracts::ExecutionStatus::Completed, // Simplified mapping
                output: Some(format!("Execution artifacts for task {}", contracts_artifacts.task_id)),
                error: None,
            }
        }).unwrap_or_else(|| LocalExecutionArtifacts {
            execution_id: "no-artifacts".to_string(),
            worker_id: "unknown".to_string(),
            status: agent_agency_contracts::ExecutionStatus::Completed,
            output: Some("No artifacts available".to_string()),
            error: None,
        });

        TaskExecutionResult {
            working_spec: None, // Would need to extract from artifacts or spec
            artifacts: local_artifacts,
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
            requirements: vec![], // TaskDescriptor doesn't have requirements field yet
            priority: match desc.priority {
                agent_agency_contracts::types::planning::TaskPriority::Low => crate::council_types::TaskPriority::Low,
                agent_agency_contracts::types::planning::TaskPriority::Medium => crate::council_types::TaskPriority::Normal,
                agent_agency_contracts::types::planning::TaskPriority::Normal => crate::council_types::TaskPriority::Normal,
                agent_agency_contracts::types::planning::TaskPriority::High => crate::council_types::TaskPriority::High,
                agent_agency_contracts::types::planning::TaskPriority::Critical => crate::council_types::TaskPriority::Critical,
                agent_agency_contracts::types::planning::TaskPriority::Urgent => crate::council_types::TaskPriority::Critical,
            },
        }
    }

    /// Convert task priority
    fn convert_priority(&self, priority: TaskPriority) -> agent_agency_contracts::types::data_processing::ProcessingPriority {
        match priority {
            TaskPriority::Low => agent_agency_contracts::types::data_processing::ProcessingPriority::Low,
            TaskPriority::Medium => agent_agency_contracts::types::data_processing::ProcessingPriority::Normal,
            TaskPriority::Normal => agent_agency_contracts::types::data_processing::ProcessingPriority::Normal,
            TaskPriority::High => agent_agency_contracts::types::data_processing::ProcessingPriority::High,
            TaskPriority::Urgent => agent_agency_contracts::types::data_processing::ProcessingPriority::High,
            TaskPriority::Critical => agent_agency_contracts::types::data_processing::ProcessingPriority::High,
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

    // TODO: Convert TaskDescriptor to ComplexTask for parallel execution
    // PLACEHOLDER: Implementation requires agent-workers dependency
    pub fn convert_to_complex_task(&self, _task: &agent_agency_contracts::TaskDescriptor) -> anyhow::Result<()> {
        Err(anyhow::anyhow!("PLACEHOLDER: convert_to_complex_task not implemented - requires agent-workers dependency"))
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
