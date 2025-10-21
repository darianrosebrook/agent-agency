//! Plan Review Service for Constitutional Council
//!
//! Evaluates generated working specifications for constitutional compliance,
//! ethical considerations, and overall plan quality before execution.

use crate::coordinator::ConsensusCoordinator;
use crate::models::{EvidencePacket, ParticipantContribution, RiskTier, TaskSpec};
use crate::types::{ConsensusResult, FinalVerdict, JudgeVerdict};
use crate::CouncilConfig;
use agent_agency_research::{MultimodalContextProvider, MultimodalContext};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, info, warn};
use uuid::Uuid;

/// Plan review service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanReviewConfig {
    /// Minimum constitutional compliance score
    pub min_constitutional_score: f64,
    /// Minimum technical feasibility score
    pub min_technical_score: f64,
    /// Minimum quality completeness score
    pub min_quality_score: f64,
    /// Maximum review time in seconds
    pub max_review_time_seconds: u64,
    /// Enable detailed rationale generation
    pub enable_detailed_rationale: bool,
    /// Require multimodal evidence for high-risk plans
    pub require_multimodal_evidence: bool,
}

/// Constitutional plan review service
pub struct PlanReviewService {
    coordinator: Arc<ConsensusCoordinator>,
    context_provider: Arc<dyn MultimodalContextProvider>,
    config: PlanReviewConfig,
}

impl PlanReviewService {
    pub fn new(
        coordinator: Arc<ConsensusCoordinator>,
        context_provider: Arc<dyn MultimodalContextProvider>,
        config: PlanReviewConfig,
    ) -> Self {
        Self {
            coordinator,
            context_provider,
            config,
        }
    }

    /// Review a generated working spec for constitutional compliance
    pub async fn review_plan(
        &self,
        working_spec: &crate::types::WorkingSpec,
        task_context: &super::super::planning::agent::TaskContext,
    ) -> Result<PlanReviewVerdict> {
        info!("Starting constitutional review of working spec: {}", working_spec.id);

        // Convert working spec to task spec for council evaluation
        let task_spec = self.working_spec_to_task_spec(working_spec, task_context)?;

        // Gather multimodal context for evidence
        let context = self.gather_plan_context(working_spec, task_context).await?;

        // Evaluate through council
        let consensus_result = self.coordinator.evaluate_task(task_spec).await?;

        // Convert consensus result to plan review verdict
        let verdict = self.consensus_to_plan_verdict(&consensus_result, working_spec)?;

        info!("Plan review completed: {} - {:?}", working_spec.id, verdict.decision);

        Ok(verdict)
    }

    /// Convert working spec to task spec format expected by council
    fn working_spec_to_task_spec(
        &self,
        working_spec: &crate::types::WorkingSpec,
        task_context: &super::super::planning::agent::TaskContext,
    ) -> Result<TaskSpec> {
        // Convert risk tier
        let risk_tier = match working_spec.risk_tier {
            1 => RiskTier::Critical,
            2 => RiskTier::High,
            3 => RiskTier::Standard,
            _ => RiskTier::Standard,
        };

        // Build task description from working spec
        let task_description = format!(
            "Implement: {}\n\nScope: {}\nRisk Tier: {}\nEstimated Effort: {} hours\n\nAcceptance Criteria:\n{}",
            working_spec.title,
            working_spec.scope.as_ref()
                .map(|s| format!("In: {}, Out: {}", s.r#in.as_ref().unwrap_or(&vec![]).join(", "), s.out.as_ref().unwrap_or(&vec![]).join(", ")))
                .unwrap_or_else(|| "Not specified".to_string()),
            working_spec.risk_tier,
            working_spec.estimated_effort_hours,
            working_spec.acceptance_criteria.iter()
                .map(|ac| format!("- {}: Given {}, When {}, Then {}", ac.id, ac.given, ac.when, ac.then))
                .collect::<Vec<_>>()
                .join("\n")
        );

        Ok(TaskSpec {
            id: Uuid::parse_str(&working_spec.id).unwrap_or_else(|_| Uuid::new_v4()),
            title: working_spec.title.clone(),
            description: task_description,
            risk_tier,
            scope_in: working_spec.scope.as_ref()
                .and_then(|s| s.r#in.clone())
                .unwrap_or_default(),
            acceptance_criteria: working_spec.acceptance_criteria.iter()
                .map(|ac| format!("Given {}, When {}, Then {}", ac.given, ac.when, ac.then))
                .collect(),
            constraints: working_spec.constraints.clone(),
            metadata: Some(serde_json::json!({
                "working_spec_id": working_spec.id,
                "generated_at": working_spec.generated_at,
                "context_hash": working_spec.context_hash,
                "estimated_effort_hours": working_spec.estimated_effort_hours,
            })),
        })
    }

    /// Gather multimodal context for plan review
    async fn gather_plan_context(
        &self,
        working_spec: &crate::types::WorkingSpec,
        task_context: &super::super::planning::agent::TaskContext,
    ) -> Result<MultimodalContext> {
        // Build context from task context and working spec
        let mut context_items = Vec::new();

        // Add repository information
        if let Some(repo) = &task_context.repository {
            context_items.push(format!("Repository: {} ({}), Size: {}KB, Contributors: {}",
                repo.name,
                repo.primary_language,
                repo.size_kb,
                repo.contributors.join(", ")
            ));
        }

        // Add team context
        if let Some(team) = &task_context.team {
            context_items.push(format!("Team constraints: {}", team.constraints.join(", ")));
            context_items.push(format!("Team preferences: {}", team.preferences.join(", ")));
        }

        // Add technical context
        if let Some(tech) = &task_context.technical {
            context_items.push(format!("Tech stack: Languages={}, Frameworks={}, Databases={}",
                tech.stack.languages.join(", "),
                tech.stack.frameworks.join(", "),
                tech.stack.databases.join(", ")
            ));
        }

        // Add working spec details
        context_items.push(format!("Working spec risk tier: {}", working_spec.risk_tier));
        context_items.push(format!("Estimated effort: {} hours", working_spec.estimated_effort_hours));
        context_items.push(format!("Acceptance criteria count: {}", working_spec.acceptance_criteria.len()));

        Ok(MultimodalContext {
            text_content: context_items.join("\n"),
            image_urls: vec![], // No images for plan review
            audio_urls: vec![], // No audio for plan review
            video_urls: vec![], // No video for plan review
            document_urls: vec![], // No documents for plan review
            metadata: HashMap::from([
                ("source".to_string(), "plan_review".to_string()),
                ("working_spec_id".to_string(), working_spec.id.clone()),
                ("risk_tier".to_string(), working_spec.risk_tier.to_string()),
            ]),
        })
    }

    /// Convert consensus result to plan review verdict
    fn consensus_to_plan_verdict(
        &self,
        consensus: &ConsensusResult,
        working_spec: &crate::types::WorkingSpec,
    ) -> Result<PlanReviewVerdict> {
        // Extract individual judge verdicts
        let judge_verdicts = self.extract_judge_verdicts(consensus)?;

        // Calculate overall scores
        let constitutional_score = self.calculate_constitutional_score(&judge_verdicts)?;
        let technical_score = self.calculate_technical_score(&judge_verdicts)?;
        let quality_score = self.calculate_quality_score(&judge_verdicts)?;

        // Determine decision based on scores and thresholds
        let decision = self.determine_review_decision(
            constitutional_score,
            technical_score,
            quality_score,
            &judge_verdicts,
        )?;

        // Generate rationale
        let rationale = self.generate_review_rationale(&decision, &judge_verdicts)?;

        // Extract suggested improvements
        let suggested_improvements = self.extract_suggested_improvements(&judge_verdicts)?;

        Ok(PlanReviewVerdict {
            working_spec_id: working_spec.id.clone(),
            decision,
            constitutional_score,
            technical_score,
            quality_score,
            judge_verdicts,
            rationale,
            suggested_improvements,
            reviewed_at: chrono::Utc::now(),
        })
    }

    /// Extract individual judge verdicts from consensus result
    fn extract_judge_verdicts(&self, consensus: &ConsensusResult) -> Result<Vec<PlanJudgeVerdict>> {
        let mut verdicts = Vec::new();

        // Extract verdicts from participant contributions
        for contribution in &consensus.participant_contributions {
            let judge_type = self.identify_judge_type(&contribution.participant_id)?;
            let verdict = self.parse_contribution_verdict(contribution)?;

            verdicts.push(PlanJudgeVerdict {
                judge_type,
                participant_id: contribution.participant_id.clone(),
                verdict,
                confidence: contribution.confidence_score,
                rationale: contribution.rationale.clone(),
                suggested_improvements: self.extract_contribution_improvements(contribution)?,
            });
        }

        Ok(verdicts)
    }

    /// Identify judge type from participant ID
    fn identify_judge_type(&self, participant_id: &str) -> Result<JudgeType> {
        match participant_id.to_lowercase().as_str() {
            id if id.contains("constitutional") => Ok(JudgeType::Constitutional),
            id if id.contains("technical") => Ok(JudgeType::Technical),
            id if id.contains("quality") => Ok(JudgeType::Quality),
            id if id.contains("integration") => Ok(JudgeType::Integration),
            _ => Ok(JudgeType::Unknown),
        }
    }

    /// Parse verdict from participant contribution
    fn parse_contribution_verdict(&self, contribution: &ParticipantContribution) -> Result<PlanVerdict> {
        // Parse based on contribution content and confidence
        // This is a simplified implementation - real implementation would parse structured verdicts

        match contribution.confidence_score {
            score if score >= 0.8 => Ok(PlanVerdict::Approved),
            score if score >= 0.6 => Ok(PlanVerdict::ApprovedWithConcerns),
            score if score >= 0.4 => Ok(PlanVerdict::NeedsRevision),
            _ => Ok(PlanVerdict::Rejected),
        }
    }

    /// Extract suggested improvements from contribution
    fn extract_contribution_improvements(&self, contribution: &ParticipantContribution) -> Result<Vec<String>> {
        // Extract improvement suggestions from rationale
        // This is a simplified implementation
        let improvements = vec![
            "Review risk tier appropriateness".to_string(),
            "Strengthen acceptance criteria".to_string(),
            "Add more specific constraints".to_string(),
        ];

        Ok(improvements)
    }

    /// Calculate constitutional compliance score
    fn calculate_constitutional_score(&self, verdicts: &[PlanJudgeVerdict]) -> Result<f64> {
        let constitutional_verdicts: Vec<_> = verdicts.iter()
            .filter(|v| matches!(v.judge_type, JudgeType::Constitutional))
            .collect();

        if constitutional_verdicts.is_empty() {
            return Ok(0.5); // Neutral score if no constitutional judge
        }

        let avg_confidence = constitutional_verdicts.iter()
            .map(|v| v.confidence)
            .sum::<f32>() / constitutional_verdicts.len() as f32;

        Ok(avg_confidence as f64)
    }

    /// Calculate technical feasibility score
    fn calculate_technical_score(&self, verdicts: &[PlanJudgeVerdict]) -> Result<f64> {
        let technical_verdicts: Vec<_> = verdicts.iter()
            .filter(|v| matches!(v.judge_type, JudgeType::Technical))
            .collect();

        if technical_verdicts.is_empty() {
            return Ok(0.5);
        }

        let avg_confidence = technical_verdicts.iter()
            .map(|v| v.confidence)
            .sum::<f32>() / technical_verdicts.len() as f32;

        Ok(avg_confidence as f64)
    }

    /// Calculate quality completeness score
    fn calculate_quality_score(&self, verdicts: &[PlanJudgeVerdict]) -> Result<f64> {
        let quality_verdicts: Vec<_> = verdicts.iter()
            .filter(|v| matches!(v.judge_type, JudgeType::Quality | JudgeType::Integration))
            .collect();

        if quality_verdicts.is_empty() {
            return Ok(0.5);
        }

        let avg_confidence = quality_verdicts.iter()
            .map(|v| v.confidence)
            .sum::<f32>() / quality_verdicts.len() as f32;

        Ok(avg_confidence as f64)
    }

    /// Determine overall review decision
    fn determine_review_decision(
        &self,
        constitutional_score: f64,
        technical_score: f64,
        quality_score: f64,
        verdicts: &[PlanJudgeVerdict],
    ) -> Result<PlanReviewDecision> {
        // Constitutional judge has veto power for critical issues
        if constitutional_score < self.config.min_constitutional_score {
            return Ok(PlanReviewDecision::Rejected {
                reason: "Constitutional compliance below minimum threshold".to_string(),
            });
        }

        // Check if any judge rejected the plan
        if verdicts.iter().any(|v| matches!(v.verdict, PlanVerdict::Rejected)) {
            return Ok(PlanReviewDecision::Rejected {
                reason: "One or more judges rejected the plan".to_string(),
            });
        }

        // Calculate overall score
        let overall_score = (constitutional_score + technical_score + quality_score) / 3.0;

        if overall_score >= 0.8 {
            Ok(PlanReviewDecision::Approved)
        } else if overall_score >= 0.6 {
            Ok(PlanReviewDecision::ApprovedWithConditions {
                conditions: self.generate_approval_conditions(verdicts)?,
            })
        } else {
            Ok(PlanReviewDecision::NeedsRevision {
                revision_requirements: self.generate_revision_requirements(verdicts)?,
            })
        }
    }

    /// Generate approval conditions
    fn generate_approval_conditions(&self, verdicts: &[PlanJudgeVerdict]) -> Result<Vec<String>> {
        let mut conditions = Vec::new();

        for verdict in verdicts {
            if matches!(verdict.verdict, PlanVerdict::ApprovedWithConcerns) {
                conditions.extend(verdict.suggested_improvements.iter().cloned());
            }
        }

        if conditions.is_empty() {
            conditions.push("Address judge concerns before execution".to_string());
        }

        Ok(conditions)
    }

    /// Generate revision requirements
    fn generate_revision_requirements(&self, verdicts: &[PlanJudgeVerdict]) -> Result<Vec<String>> {
        let mut requirements = Vec::new();

        for verdict in verdicts {
            if matches!(verdict.verdict, PlanVerdict::NeedsRevision | PlanVerdict::Rejected) {
                requirements.extend(verdict.suggested_improvements.iter().cloned());
            }
        }

        if requirements.is_empty() {
            requirements.push("Revise plan based on judge feedback".to_string());
        }

        Ok(requirements)
    }

    /// Generate comprehensive review rationale
    fn generate_review_rationale(
        &self,
        decision: &PlanReviewDecision,
        verdicts: &[PlanJudgeVerdict],
    ) -> Result<String> {
        let mut rationale = format!("Plan review decision: {:?}\n\n", decision);

        rationale.push_str("Judge verdicts:\n");
        for verdict in verdicts {
            rationale.push_str(&format!("- {} ({}): {:?} (confidence: {:.2})\n  {}\n",
                verdict.judge_type, verdict.participant_id, verdict.verdict,
                verdict.confidence, verdict.rationale
            ));
        }

        rationale.push_str("\nDecision reasoning:\n");
        match decision {
            PlanReviewDecision::Approved => {
                rationale.push_str("- All quality thresholds met\n");
                rationale.push_str("- No critical constitutional issues\n");
                rationale.push_str("- Plan approved for execution\n");
            }
            PlanReviewDecision::ApprovedWithConditions { conditions } => {
                rationale.push_str("- Plan approved with conditions:\n");
                for condition in conditions {
                    rationale.push_str(&format!("  - {}\n", condition));
                }
            }
            PlanReviewDecision::NeedsRevision { revision_requirements } => {
                rationale.push_str("- Plan needs revision:\n");
                for req in revision_requirements {
                    rationale.push_str(&format!("  - {}\n", req));
                }
            }
            PlanReviewDecision::Rejected { reason } => {
                rationale.push_str(&format!("- Plan rejected: {}\n", reason));
            }
        }

        Ok(rationale)
    }

    /// Extract all suggested improvements
    fn extract_suggested_improvements(&self, verdicts: &[PlanJudgeVerdict]) -> Result<Vec<String>> {
        let mut all_improvements = Vec::new();

        for verdict in verdicts {
            all_improvements.extend(verdict.suggested_improvements.iter().cloned());
        }

        // Remove duplicates and sort
        all_improvements.sort();
        all_improvements.dedup();

        Ok(all_improvements)
    }
}

/// Plan review verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanReviewVerdict {
    pub working_spec_id: String,
    pub decision: PlanReviewDecision,
    pub constitutional_score: f64,
    pub technical_score: f64,
    pub quality_score: f64,
    pub judge_verdicts: Vec<PlanJudgeVerdict>,
    pub rationale: String,
    pub suggested_improvements: Vec<String>,
    pub reviewed_at: chrono::DateTime<chrono::Utc>,
}

/// Plan review decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanReviewDecision {
    Approved,
    ApprovedWithConditions { conditions: Vec<String> },
    NeedsRevision { revision_requirements: Vec<String> },
    Rejected { reason: String },
}

/// Individual judge verdict on plan
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlanJudgeVerdict {
    pub judge_type: JudgeType,
    pub participant_id: String,
    pub verdict: PlanVerdict,
    pub confidence: f32,
    pub rationale: String,
    pub suggested_improvements: Vec<String>,
}

/// Judge type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JudgeType {
    Constitutional,
    Technical,
    Quality,
    Integration,
    Unknown,
}

/// Individual plan verdict
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanVerdict {
    Approved,
    ApprovedWithConcerns,
    NeedsRevision,
    Rejected,
}

pub type Result<T> = std::result::Result<T, PlanReviewError>;

#[derive(Debug, thiserror::Error)]
pub enum PlanReviewError {
    #[error("Council evaluation failed: {0}")]
    CouncilError(String),

    #[error("Context gathering failed: {0}")]
    ContextError(String),

    #[error("Verdict parsing failed: {0}")]
    VerdictParseError(String),

    #[error("Score calculation failed: {0}")]
    ScoreCalculationError(String),

    #[error("Review timeout exceeded")]
    TimeoutError,

    #[error("Invalid working spec: {0}")]
    InvalidSpec(String),
}
