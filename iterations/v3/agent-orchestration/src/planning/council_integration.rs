//! Council Integration Trait
//!
//! Unified trait for all council interactions across the orchestration system.
//! Consolidates council review, presentation, and decision-making interfaces.

use anyhow::Result;
use std::sync::Arc;
use uuid::Uuid;

use agent_agency_contracts::execution_artifacts::ExecutionArtifacts;
use agent_agency_contracts::final_verdict::FinalVerdictContract;
use agent_agency_contracts::WorkingSpec;

use crate::council::{Council, CouncilConfig};
use crate::planning::plan_types::ExecutionPlan;

/// Unified council integration trait
#[async_trait::async_trait]
pub trait CouncilIntegration: Send + Sync {
    /// Review execution plan (CAWS Examination stage)
    async fn review_plan(
        &self,
        execution_plan: &ExecutionPlan,
        working_spec: &WorkingSpec,
        spec_id: Option<&str>,
        project_root: Option<&std::path::Path>,
    ) -> Result<PlanReviewResult>;

    /// Present completed work to council (CAWS Pleading stage)
    async fn present_work(
        &self,
        artifacts: &[ExecutionArtifacts],
        milestone_id: &str,
        worker_id: Uuid,
    ) -> Result<WorkPresentationResult>;

    /// Get council verdict for work
    async fn get_verdict(
        &self,
        artifacts: &ExecutionArtifacts,
        working_spec: &WorkingSpec,
    ) -> Result<FinalVerdictContract>;

    /// Check if council approval is required
    fn requires_approval(&self, risk_tier: u8) -> bool;
}

/// Result of plan review
#[derive(Debug, Clone)]
pub struct PlanReviewResult {
    pub approved: bool,
    pub needs_refinement: bool,
    pub refinement_reason: String,
    pub council_feedback: Option<String>,
}

/// Result of work presentation
#[derive(Debug, Clone)]
pub struct WorkPresentationResult {
    pub approved: bool,
    pub needs_refinement: bool,
    pub refinement_reason: String,
    pub verdict: Option<FinalVerdictContract>,
}

/// Council integration implementation using Council
pub struct CouncilIntegrationImpl {
    council: Arc<Council>,
    #[allow(dead_code)] // Reserved for future use
    config: CouncilConfig,
}

impl CouncilIntegrationImpl {
    /// Create new council integration
    pub fn new(council: Arc<Council>, config: CouncilConfig) -> Self {
        Self { council, config }
    }
}

#[async_trait::async_trait]
impl CouncilIntegration for CouncilIntegrationImpl {
    async fn review_plan(
        &self,
        _execution_plan: &ExecutionPlan,
        working_spec: &WorkingSpec,
        spec_id: Option<&str>,
        project_root: Option<&std::path::Path>,
    ) -> Result<PlanReviewResult> {
        // Detect complexity mode for review context
        let complexity_mode = if let Some(root) = project_root {
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(root).ok()
        } else {
            crate::planning::caws_complexity_mode::CawsComplexityMode::detect(std::path::Path::new(
                ".",
            ))
            .ok()
        };

        // Create review context for council with spec and mode information
        let mut constraints = std::collections::HashMap::new();
        if let Some(spec_id) = spec_id {
            constraints.insert("spec_id".to_string(), spec_id.to_string());
        }
        if let Some(mode) = complexity_mode {
            constraints.insert("complexity_mode".to_string(), format!("{:?}", mode));
        }

        let review_context = crate::judge_backup::types::ReviewContext {
            session_id: format!("plan_review_{}", uuid::Uuid::new_v4()),
            working_spec: serde_json::to_string(working_spec)
                .map_err(|e| anyhow::anyhow!("Failed to serialize working spec: {}", e))?,
            risk_tier: working_spec.risk_tier as u8,
            previous_reviews: vec![],
            constraints,
        };

        // Conduct council review
        let session = self
            .council
            .conduct_review(working_spec.clone(), review_context)
            .await
            .map_err(|e| anyhow::anyhow!("Council plan review failed: {:?}", e))?;

        // Convert council session result to PlanReviewResult
        let approved = session
            .final_decision
            .as_ref()
            .map(|d| matches!(d, crate::decision_making::FinalDecision::Proceed { .. }))
            .unwrap_or(false);

        let needs_refinement = session
            .final_decision
            .as_ref()
            .map(|d| matches!(d, crate::decision_making::FinalDecision::Refine { .. }))
            .unwrap_or(false);

        let refinement_reason = match session.final_decision.as_ref() {
            Some(crate::decision_making::FinalDecision::Refine {
                refinement_directive,
                ..
            }) => {
                format!("Refinement required: {:?}", refinement_directive)
            }
            Some(crate::decision_making::FinalDecision::Reject { reason, .. }) => {
                format!("Rejected: {}", reason)
            }
            _ => {
                // Extract reasoning from contributions if available
                session
                    .contributions
                    .iter()
                    .find_map(|c| {
                        if !c.reasoning.is_empty() {
                            Some(c.reasoning.clone())
                        } else {
                            None
                        }
                    })
                    .unwrap_or_default()
            }
        };

        Ok(PlanReviewResult {
            approved,
            needs_refinement,
            refinement_reason: refinement_reason.clone(),
            council_feedback: session.contributions.iter().find_map(|c| {
                if !c.reasoning.is_empty() {
                    Some(c.reasoning.clone())
                } else {
                    None
                }
            }),
        })
    }

    async fn present_work(
        &self,
        artifacts: &[ExecutionArtifacts],
        milestone_id: &str,
        worker_id: Uuid,
    ) -> Result<WorkPresentationResult> {
        // Convert artifacts to working spec for council review
        if artifacts.is_empty() {
            return Err(anyhow::anyhow!("No artifacts to present"));
        }

        // Use primary artifact to create working spec
        let _primary_artifact = &artifacts[0];

        // Create working spec from artifact metadata
        let working_spec = WorkingSpec {
            version: "1.0".to_string(),
            id: format!("milestone_{}", milestone_id),
            title: format!("Work presentation for milestone {}", milestone_id),
            description: format!(
                "Completed work from worker {} for milestone {}",
                worker_id, milestone_id
            ),
            goals: vec!["Complete milestone execution".to_string()],
            risk_tier: 2, // Default risk tier
            constraints: agent_agency_contracts::working_spec::WorkingSpecConstraints {
                max_duration_minutes: None,
                max_iterations: None,
                budget_limits: None,
                scope_restrictions: None,
            },
            acceptance_criteria: vec![],
            test_plan: agent_agency_contracts::TestPlan {
                unit_tests: vec![],
                integration_tests: vec![],
                e2e_scenarios: vec![],
                coverage_targets: None,
            },
            rollback_plan: agent_agency_contracts::RollbackPlan::default(),
            context: agent_agency_contracts::WorkingSpecContext {
                workspace_root: ".".to_string(),
                git_branch: "main".to_string(),
                recent_changes: vec![],
                dependencies: std::collections::HashMap::new(),
                environment: agent_agency_contracts::task_request::Environment::Development,
            },
            non_functional_requirements: None,
            validation_results: None,
            quality_gates: None,
            scope: vec![],
            metadata: None,
            milestones: vec![],
            change_budget: agent_agency_contracts::planning_io::ChangeBudget {
                max_files: 50,
                max_loc: 1000,
                max_migrations: 0,
                allow_breaking_changes: false,
                allow_new_dependencies: false,
                enforcement_mode: agent_agency_contracts::planning_io::BudgetEnforcement::Strict,
            },
            file_changes: vec![],
            coverage_targets: None,
            overview: format!(
                "Completed work from worker {} for milestone {}",
                worker_id, milestone_id
            ),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };

        // Create review context
        let review_context = crate::judge_backup::types::ReviewContext {
            session_id: format!("presentation_{}", uuid::Uuid::new_v4()),
            working_spec: serde_json::to_string(&working_spec)
                .map_err(|e| anyhow::anyhow!("Failed to serialize working spec: {}", e))?,
            risk_tier: working_spec.risk_tier as u8,
            previous_reviews: vec![],
            constraints: std::collections::HashMap::new(),
        };

        // Conduct council review
        let session = self
            .council
            .conduct_review(working_spec.clone(), review_context)
            .await
            .map_err(|e| anyhow::anyhow!("Council review failed: {:?}", e))?;

        // Convert council session result to WorkPresentationResult
        let approved = session
            .final_decision
            .as_ref()
            .map(|d| matches!(d, crate::decision_making::FinalDecision::Proceed { .. }))
            .unwrap_or(false);

        let needs_refinement = session
            .final_decision
            .as_ref()
            .map(|d| matches!(d, crate::decision_making::FinalDecision::Refine { .. }))
            .unwrap_or(false);

        let refinement_reason = match session.final_decision.as_ref() {
            Some(crate::decision_making::FinalDecision::Refine {
                refinement_directive,
                ..
            }) => {
                format!("Refinement required: {:?}", refinement_directive)
            }
            Some(crate::decision_making::FinalDecision::Reject { reason, .. }) => {
                format!("Rejected: {}", reason)
            }
            _ => String::new(),
        };

        // Convert FinalDecision to FinalVerdictContract
        let verdict = session.final_decision.as_ref().map(|d| {
            let (decision, remediation) = match d {
                crate::decision_making::FinalDecision::Proceed { .. } => (
                    agent_agency_contracts::final_verdict::FinalDecision::Accept,
                    vec![],
                ),
                crate::decision_making::FinalDecision::Refine {
                    refinement_directive,
                    ..
                } => (
                    agent_agency_contracts::final_verdict::FinalDecision::Modify,
                    vec![format!("Refinement required: {:?}", refinement_directive)],
                ),
                crate::decision_making::FinalDecision::Reject { reason, .. } => (
                    agent_agency_contracts::final_verdict::FinalDecision::Reject,
                    vec![reason.clone()],
                ),
                crate::decision_making::FinalDecision::Escalate { reason, .. } => (
                    agent_agency_contracts::final_verdict::FinalDecision::Modify,
                    vec![format!("Escalation required: {}", reason)],
                ),
            };

            FinalVerdictContract {
                decision,
                votes: session
                    .contributions
                    .iter()
                    .map(|c| agent_agency_contracts::final_verdict::VoteEntry {
                        judge_id: c.judge_id.clone(),
                        weight: c.confidence as f32,
                        verdict: match &c.verdict {
                            crate::judge_backup::verdicts::JudgeVerdict::Approve { .. } => {
                                agent_agency_contracts::final_verdict::VoteVerdict::Pass
                            }
                            crate::judge_backup::verdicts::JudgeVerdict::Reject { .. } => {
                                agent_agency_contracts::final_verdict::VoteVerdict::Fail
                            }
                            crate::judge_backup::verdicts::JudgeVerdict::Refine { .. } => {
                                agent_agency_contracts::final_verdict::VoteVerdict::Uncertain
                            }
                        },
                    })
                    .collect(),
                dissent: String::new(),
                remediation,
                constitutional_refs: vec![],
                verification_summary: agent_agency_contracts::final_verdict::VerificationSummary {
                    claims_total: 0,
                    claims_verified: 0,
                    coverage_pct: 0.0,
                },
            }
        });

        Ok(WorkPresentationResult {
            approved,
            needs_refinement,
            refinement_reason,
            verdict,
        })
    }

    async fn get_verdict(
        &self,
        artifacts: &ExecutionArtifacts,
        _working_spec: &WorkingSpec,
    ) -> Result<FinalVerdictContract> {
        // Present work to get verdict
        // ArtifactMetadata doesn't support arbitrary key-value storage, so use working_spec_id as fallback
        let milestone_id = artifacts.working_spec_id.as_str();

        let worker_id = artifacts
            .provenance
            .worker_id
            .as_ref()
            .and_then(|w| Uuid::parse_str(w).ok())
            .unwrap_or_else(Uuid::new_v4);

        let presentation_result = self
            .present_work(&[artifacts.clone()], milestone_id, worker_id)
            .await?;

        presentation_result
            .verdict
            .ok_or_else(|| anyhow::anyhow!("No verdict returned from council presentation"))
    }

    fn requires_approval(&self, risk_tier: u8) -> bool {
        // Risk tier 1 always requires approval
        risk_tier == 1
    }
}
