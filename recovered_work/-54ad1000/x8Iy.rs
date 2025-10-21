//! Council Approval Workflow for CAWS Compliance
//!
//! Handles budget overrun pleas, council reviews, and waiver generation
//! for autonomous file editing operations.
//!
//! @author @darianrosebrook

use std::sync::Arc;
use std::time::Duration;
use tokio::time;
use uuid::Uuid;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::types::{Task, StopReason};
use crate::evaluation::EvalReport;

/// Council approval workflow for budget overruns
pub struct CouncilApprovalWorkflow {
    council: Arc<dyn CouncilInterface>,
    default_timeout: Duration,
    waiver_persistence_path: std::path::PathBuf,
}

/// Budget overrun plea with evidence and justification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetOverrunPlea {
    pub task_id: Uuid,
    pub current_budget: BudgetLimits,
    pub proposed_budget: BudgetLimits,
    pub rationale: String,
    pub evidence: PleaEvidence,
    pub mitigation_plan: String,
    pub risk_assessment: RiskAssessment,
    pub rl_prediction: Option<RLPrediction>,
    pub timestamp: DateTime<Utc>,
}

/// Evidence supporting the budget overrun plea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PleaEvidence {
    pub iterations_attempted: usize,
    pub best_score_achieved: f64,
    pub score_history: Vec<f64>,
    pub failed_criteria: Vec<String>,
    pub complexity_justification: String,
    pub recent_artifacts: Vec<String>, // File paths and summaries
}

/// Risk assessment for the plea
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskAssessment {
    pub impact_level: ImpactLevel,
    pub rollback_complexity: RollbackComplexity,
    pub alternative_approaches: Vec<String>,
    pub monitoring_plan: String,
}

/// RL prediction for budget requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLPrediction {
    pub predicted_files: usize,
    pub predicted_loc: usize,
    pub confidence: f64,
    pub similar_tasks: Vec<String>,
}

/// Decision from council review
#[derive(Debug, Clone)]
pub enum CouncilDecision {
    Approved(Waiver),
    Rejected(String), // rejection reason
}

/// Impact level for risk assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImpactLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Rollback complexity assessment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RollbackComplexity {
    Simple,
    Moderate,
    Complex,
    HighRisk,
}

/// Budget limits (re-export from budget_checker)
pub use super::budget_checker::{BudgetLimits, BudgetState};

/// Waiver granting increased budget limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waiver {
    pub id: Uuid,
    pub task_id: Uuid,
    pub granted_by: String, // Council session ID or "auto-approved"
    pub original_limits: BudgetLimits,
    pub granted_limits: BudgetLimits,
    pub justification: String,
    pub conditions: Vec<String>,
    pub expires_at: DateTime<Utc>,
    pub issued_at: DateTime<Utc>,
}

/// Interface for council operations
#[async_trait::async_trait]
pub trait CouncilInterface: Send + Sync {
    async fn review_plea(&self, plea: &BudgetOverrunPlea) -> Result<CouncilVerdict, CouncilError>;
}

/// Council verdict from review
#[derive(Debug, Clone)]
pub struct CouncilVerdict {
    pub approved: bool,
    pub confidence: f64,
    pub reasoning: String,
    pub conditions: Vec<String>,
    pub reviewer_count: usize,
}

/// Council operation errors
#[derive(Debug, thiserror::Error)]
pub enum CouncilError {
    #[error("Council session failed: {0}")]
    SessionFailed(String),

    #[error("Review timeout after {0:?}")]
    Timeout(Duration),

    #[error("Invalid plea format: {0}")]
    InvalidPlea(String),

    #[error("Council unavailable: {0}")]
    Unavailable(String),
}

impl Default for CouncilApprovalWorkflow {
    fn default() -> Self {
        // Try to create real council, fall back to no-op if it fails
        let council: Arc<dyn CouncilInterface> = match std::env::var("AGENT_DISABLE_COUNCIL") {
            Ok(val) if val == "1" => Arc::new(NoOpCouncil),
            _ => {
                // Try to create real council, fall back to no-op on failure
                match futures::executor::block_on(RealCouncil::new()) {
                    Ok(real_council) => Arc::new(real_council),
                    Err(_) => {
                        eprintln!("Warning: Failed to initialize real council, using no-op fallback");
                        Arc::new(NoOpCouncil)
                    }
                }
            }
        };

        Self {
            council,
            default_timeout: Duration::from_secs(30),
            waiver_persistence_path: std::path::PathBuf::from(".caws/waivers"),
        }
    }
}

impl CouncilApprovalWorkflow {
    /// Create new council approval workflow
    pub fn new(council: Arc<dyn CouncilInterface>) -> Self {
        Self {
            council,
            default_timeout: Duration::from_secs(30),
            waiver_persistence_path: std::path::PathBuf::from(".caws/waivers"),
        }
    }

    /// Plead case with timeout (default = reject on timeout)
    pub async fn plead_case_with_timeout(
        &self,
        plea: BudgetOverrunPlea,
        timeout: Duration,
    ) -> Result<CouncilDecision, CouncilError> {
        // Validate plea format
        self.validate_plea(&plea)?;

        // Submit to council with timeout
        let verdict = time::timeout(
            timeout,
            self.council.review_plea(&plea)
        ).await
        .unwrap_or_else(|_| {
            // Timeout default = reject
            CouncilVerdict {
                approved: false,
                confidence: 0.0,
                reasoning: "Council review timeout".to_string(),
                conditions: Vec::new(),
                reviewer_count: 0,
            }
        })?;

        if verdict.approved {
            let waiver = self.generate_waiver(&plea, &verdict).await?;
            self.persist_waiver(&waiver).await?;
            Ok(CouncilDecision::Approved(waiver))
        } else {
            Ok(CouncilDecision::Rejected(verdict.reasoning))
        }
    }

    /// Plead case with default timeout
    pub async fn plead_case(&self, plea: BudgetOverrunPlea) -> Result<CouncilDecision, CouncilError> {
        self.plead_case_with_timeout(plea, self.default_timeout).await
    }

    /// Generate waiver from plea and verdict
    async fn generate_waiver(
        &self,
        plea: &BudgetOverrunPlea,
        verdict: &CouncilVerdict,
    ) -> Result<Waiver, CouncilError> {
        let waiver = Waiver {
            id: Uuid::new_v4(),
            task_id: plea.task_id,
            granted_by: format!("council-session-{}", Uuid::new_v4()),
            original_limits: plea.current_budget.clone(),
            granted_limits: plea.proposed_budget.clone(),
            justification: verdict.reasoning.clone(),
            conditions: verdict.conditions.clone(),
            expires_at: Utc::now() + chrono::Duration::hours(24), // 24 hour waiver
            issued_at: Utc::now(),
        };

        Ok(waiver)
    }

    /// Persist waiver to filesystem
    async fn persist_waiver(&self, waiver: &Waiver) -> Result<(), CouncilError> {
        // Ensure directory exists
        tokio::fs::create_dir_all(&self.waiver_persistence_path)
            .await
            .map_err(|e| CouncilError::SessionFailed(format!("Failed to create waivers dir: {}", e)))?;

        let waiver_path = self.waiver_persistence_path.join(format!("{}.yaml", waiver.id));

        let yaml = serde_yaml::to_string(waiver)
            .map_err(|e| CouncilError::SessionFailed(format!("Failed to serialize waiver: {}", e)))?;

        tokio::fs::write(&waiver_path, yaml)
            .await
            .map_err(|e| CouncilError::SessionFailed(format!("Failed to write waiver: {}", e)))?;

        Ok(())
    }

    /// Validate plea format and completeness
    fn validate_plea(&self, plea: &BudgetOverrunPlea) -> Result<(), CouncilError> {
        if plea.rationale.trim().is_empty() {
            return Err(CouncilError::InvalidPlea("Rationale cannot be empty".to_string()));
        }

        if plea.evidence.score_history.is_empty() {
            return Err(CouncilError::InvalidPlea("Evidence must include score history".to_string()));
        }

        if plea.proposed_budget.max_files <= plea.current_budget.max_files &&
           plea.proposed_budget.max_loc <= plea.current_budget.max_loc {
            return Err(CouncilError::InvalidPlea("Proposed budget must exceed current limits".to_string()));
        }

        Ok(())
    }

    /// Create plea from task context
    pub fn create_plea(
        &self,
        task_id: Uuid,
        current_limits: BudgetLimits,
        proposed_limits: BudgetLimits,
        task_context: &Task,
        eval_reports: &[EvalReport],
        stop_reason: &StopReason,
    ) -> BudgetOverrunPlea {
        let score_history: Vec<f64> = eval_reports.iter()
            .map(|r| r.score)
            .collect();

        let best_score = score_history.iter().cloned().fold(0.0, f64::max);

        let failed_criteria: Vec<String> = eval_reports.last()
            .map(|r| r.failed_criteria.clone())
            .unwrap_or_default();

        BudgetOverrunPlea {
            task_id,
            current_budget: current_limits,
            proposed_budget: proposed_limits,
            rationale: format!(
                "Task '{}' requires budget increase to complete. Current progress shows \
                 {} iterations with best score {:.2}. Stop reason: {:?}. \
                 Failed criteria: {:?}. Requesting increase from {} files/{} LOC to {} files/{} LOC.",
                task_context.description,
                eval_reports.len(),
                best_score,
                stop_reason,
                failed_criteria,
                current_limits.max_files, current_limits.max_loc,
                proposed_limits.max_files, proposed_limits.max_loc
            ),
            evidence: PleaEvidence {
                iterations_attempted: eval_reports.len(),
                best_score_achieved: best_score,
                score_history,
                failed_criteria,
                complexity_justification: format!("Task involves {} and has shown diminishing returns", task_context.description),
                recent_artifacts: Vec::new(), // TODO: populate with actual artifacts
            },
            mitigation_plan: "Will implement strict monitoring, allow-list enforcement, and atomic operations".to_string(),
            risk_assessment: RiskAssessment {
                impact_level: ImpactLevel::Medium,
                rollback_complexity: RollbackComplexity::Moderate,
                alternative_approaches: vec![
                    "Split into smaller tasks".to_string(),
                    "Optimize existing implementation".to_string(),
                    "Use different approach with lower budget requirements".to_string(),
                ],
                monitoring_plan: "Track file changes, LOC deltas, and evaluation scores per iteration".to_string(),
            },
            rl_prediction: None, // TODO: integrate with RL system
            timestamp: Utc::now(),
        }
    }
}

/// Real council integration using agent-agency-council
pub struct RealCouncil {
    council: Council,
}

impl RealCouncil {
    pub async fn new() -> Result<Self, CouncilError> {
        let config = CouncilConfig {
            session_timeout_seconds: 30,
            min_judges_required: 1,
            max_judges_per_session: 3,
            judge_selection_strategy: JudgeSelectionStrategy::RoundRobin,
            consensus_strategy: ConsensusStrategy::Majority,
            risk_thresholds: RiskThresholds {
                max_risk_score: 0.8,
                min_confidence_threshold: 0.6,
            },
            enable_parallel_reviews: false,
            judge_timeout_seconds: 10,
            enable_circuit_breakers: true,
            enable_graceful_degradation: true,
        };

        let council = Council::new(config)
            .await
            .map_err(|e| CouncilError::Unavailable(format!("Failed to initialize council: {}", e)))?;

        Ok(Self { council })
    }
}

#[async_trait::async_trait]
impl CouncilInterface for RealCouncil {
    async fn review_plea(&self, plea: &BudgetOverrunPlea) -> Result<CouncilVerdict, CouncilError> {
        use agent_agency_council::{Task, Priority, TaskType};

        // Convert our plea to council task format
        let council_task = Task {
            id: plea.task_id.to_string(),
            title: format!("Budget overrun plea for task {}", plea.task_id),
            description: plea.rationale.clone(),
            priority: match plea.risk_assessment.impact_level {
                ImpactLevel::Critical => Priority::Critical,
                ImpactLevel::High => Priority::High,
                _ => Priority::Medium,
            },
            task_type: TaskType::Decision,
            metadata: serde_json::json!({
                "plea": plea,
                "evidence": &plea.evidence,
                "mitigation": &plea.mitigation_plan,
                "risk_assessment": &plea.risk_assessment
            }),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        // Submit to council for review
        let decision = self.council.review_task(council_task)
            .await
            .map_err(|e| CouncilError::SessionFailed(format!("Council review failed: {}", e)))?;

        Ok(CouncilVerdict {
            approved: decision.approved,
            confidence: decision.confidence,
            reasoning: decision.reasoning,
            conditions: decision.conditions,
            reviewer_count: decision.reviewer_count,
        })
    }
}

/// No-op council for testing/fallback
pub struct NoOpCouncil;

#[async_trait::async_trait]
impl CouncilInterface for NoOpCouncil {
    async fn review_plea(&self, _plea: &BudgetOverrunPlea) -> Result<CouncilVerdict, CouncilError> {
        // Auto-approve for development/testing
        Ok(CouncilVerdict {
            approved: true,
            confidence: 0.8,
            reasoning: "Auto-approved for development".to_string(),
            conditions: vec!["Monitor closely".to_string()],
            reviewer_count: 1,
        })
    }
}
