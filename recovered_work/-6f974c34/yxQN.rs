//! Refinement Coordinator
//!
//! Coordinates council-driven refinement decisions based on quality feedback,
//! implementing intelligent refinement loops that balance improvement with efficiency.

use std::collections::HashMap;
use std::sync::Arc;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::planning::types::{WorkingSpec, ExecutionArtifacts};
use crate::quality::{QualityReport, SatisficingResult};
use crate::council::plan_review::{PlanReviewService, PlanReviewVerdict};

/// Refinement coordinator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementCoordinatorConfig {
    /// Maximum refinement iterations
    pub max_iterations: usize,
    /// Minimum quality improvement threshold
    pub min_quality_improvement: f64,
    /// Council voting threshold for refinement decisions
    pub council_vote_threshold: f64,
    /// Enable council consultation for all decisions
    pub always_consult_council: bool,
    /// Refinement strategy selection mode
    pub strategy_selection_mode: StrategySelectionMode,
}

/// Strategy selection mode
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StrategySelectionMode {
    /// Council decides strategy
    CouncilDriven,
    /// Automated selection based on quality patterns
    PatternBased,
    /// Hybrid: council validates automated recommendations
    Hybrid,
}

/// Refinement decision
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RefinementDecision {
    pub task_id: Uuid,
    pub iteration: usize,
    pub should_refine: bool,
    pub strategy: Option<RefinementStrategy>,
    pub priority: RefinementPriority,
    pub scope: RefinementScope,
    pub reasoning: String,
    pub council_votes: HashMap<String, bool>,
    pub quality_improvement_needed: f64,
    pub time_estimate: Option<u64>, // seconds
    pub confidence_score: f64,
    pub decided_at: DateTime<Utc>,
}

/// Refinement strategy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefinementStrategy {
    /// Fix specific failing gates
    TargetedFixes(Vec<String>),
    /// Comprehensive quality improvement
    QualityOverhaul,
    /// Architectural restructuring
    RefactorArchitecture,
    /// Add comprehensive testing
    EnhanceTesting,
    /// Optimize performance
    PerformanceOptimization,
    /// Security hardening
    SecurityEnhancement,
    /// Documentation improvement
    DocumentationFocus,
    /// Custom council-specified approach
    CouncilSpecified(String),
}

/// Refinement priority levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RefinementPriority {
    /// Critical issues requiring immediate attention
    Critical,
    /// High priority quality improvements
    High,
    /// Medium priority enhancements
    Medium,
    /// Low priority polish items
    Low,
    /// Optional nice-to-have improvements
    Optional,
}

/// Refinement scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RefinementScope {
    /// Fix only failing components
    Minimal,
    /// Address all quality issues
    Comprehensive,
    /// Full architectural review
    Architectural,
    /// Complete rewrite if necessary
    Complete,
}

/// Refinement context for decision making
#[derive(Debug, Clone)]
pub struct RefinementContext {
    pub working_spec: WorkingSpec,
    pub quality_report: QualityReport,
    pub satisficing_result: SatisficingResult,
    pub execution_artifacts: ExecutionArtifacts,
    pub iteration_history: Vec<QualityReport>,
    pub council_reviews: Vec<PlanReviewVerdict>,
}

/// Refinement coordinator
pub struct RefinementCoordinator {
    config: RefinementCoordinatorConfig,
    plan_review_service: Arc<dyn PlanReviewService>,
}

impl RefinementCoordinator {
    pub fn new(
        config: RefinementCoordinatorConfig,
        plan_review_service: Arc<dyn PlanReviewService>,
    ) -> Self {
        Self {
            config,
            plan_review_service,
        }
    }

    /// Make refinement decision based on quality feedback
    pub async fn decide_refinement(
        &self,
        context: RefinementContext,
        iteration: usize,
    ) -> Result<RefinementDecision, RefinementCoordinatorError> {
        tracing::info!("Making refinement decision for task {} iteration {}",
            context.quality_report.task_id, iteration);

        // Quick checks first
        if iteration >= self.config.max_iterations {
            return Ok(RefinementDecision {
                task_id: context.quality_report.task_id,
                iteration,
                should_refine: false,
                strategy: None,
                priority: RefinementPriority::Optional,
                scope: RefinementScope::Minimal,
                reasoning: format!("Maximum iterations ({}) reached", self.config.max_iterations),
                council_votes: HashMap::new(),
                quality_improvement_needed: 0.0,
                time_estimate: None,
                confidence_score: 1.0,
                decided_at: Utc::now(),
            });
        }

        // Check satisficing result
        if !context.satisficing_result.should_continue {
            return Ok(RefinementDecision {
                task_id: context.quality_report.task_id,
                iteration,
                should_refine: false,
                strategy: None,
                priority: RefinementPriority::Optional,
                scope: RefinementScope::Minimal,
                reasoning: context.satisficing_result.reason.clone(),
                council_votes: HashMap::new(),
                quality_improvement_needed: context.satisficing_result.quality_improvement,
                time_estimate: None,
                confidence_score: 0.9,
                decided_at: Utc::now(),
            });
        }

        // Analyze quality issues
        let quality_analysis = self.analyze_quality_issues(&context.quality_report);

        // Determine if refinement is needed
        let should_refine = self.should_refine(&context, &quality_analysis, iteration)?;

        if !should_refine {
            return Ok(RefinementDecision {
                task_id: context.quality_report.task_id,
                iteration,
                should_refine: false,
                strategy: None,
                priority: RefinementPriority::Optional,
                scope: RefinementScope::Minimal,
                reasoning: "Quality standards met or improvement not justified".to_string(),
                council_votes: HashMap::new(),
                quality_improvement_needed: 0.0,
                time_estimate: None,
                confidence_score: 0.8,
                decided_at: Utc::now(),
            });
        }

        // Consult council for refinement strategy
        let council_decision = self.consult_council_for_strategy(
            &context,
            &quality_analysis,
            iteration,
        ).await?;

        // Create refinement decision
        let strategy = self.select_refinement_strategy(&context, &quality_analysis, &council_decision)?;
        let priority = self.determine_priority(&context, &quality_analysis);
        let scope = self.determine_scope(&context, &quality_analysis);
        let time_estimate = self.estimate_refinement_time(&strategy, &scope);

        Ok(RefinementDecision {
            task_id: context.quality_report.task_id,
            iteration,
            should_refine: true,
            strategy: Some(strategy),
            priority,
            scope,
            reasoning: council_decision.reasoning,
            council_votes: council_decision.votes,
            quality_improvement_needed: quality_analysis.improvement_needed,
            time_estimate,
            confidence_score: council_decision.confidence,
            decided_at: Utc::now(),
        })
    }

    /// Analyze quality issues to determine refinement needs
    fn analyze_quality_issues(&self, report: &QualityReport) -> QualityAnalysis {
        let failing_gates = report.gate_results.iter()
            .filter(|r| matches!(r.status, crate::quality::gates::GateStatus::Failed | crate::quality::gates::GateStatus::Error))
            .map(|r| r.name.clone())
            .collect::<Vec<_>>();

        let warning_gates = report.gate_results.iter()
            .filter(|r| r.status == crate::quality::gates::GateStatus::Warning)
            .map(|r| r.name.clone())
            .collect::<Vec<_>>();

        let overall_score = report.overall_score;
        let improvement_needed = if overall_score < 0.8 {
            0.8 - overall_score
        } else if overall_score < 0.9 {
            0.9 - overall_score
        } else {
            1.0 - overall_score
        };

        let has_critical_failures = failing_gates.contains(&"caws_compliance".to_string())
            || failing_gates.contains(&"testing".to_string());

        let has_security_issues = failing_gates.contains(&"security".to_string());

        QualityAnalysis {
            failing_gates,
            warning_gates,
            overall_score,
            improvement_needed,
            has_critical_failures,
            has_security_issues,
            risk_tier: report.risk_tier,
        }
    }

    /// Determine if refinement should continue
    fn should_refine(
        &self,
        context: &RefinementContext,
        analysis: &QualityAnalysis,
        iteration: usize,
    ) -> Result<bool, RefinementCoordinatorError> {
        // Always consult council for critical/high risk tasks
        if matches!(analysis.risk_tier, crate::planning::types::RiskTier::Critical)
            && self.config.always_consult_council {
            return Ok(true);
        }

        // Check for critical failures
        if analysis.has_critical_failures {
            return Ok(true);
        }

        // Check minimum improvement threshold
        if analysis.improvement_needed >= self.config.min_quality_improvement {
            return Ok(true);
        }

        // Check if we're early in the process and could still improve
        if iteration < 2 && analysis.overall_score < 0.95 {
            return Ok(true);
        }

        // Check if satisficing allows continuation
        Ok(context.satisficing_result.should_continue)
    }

    /// Consult council for refinement strategy
    async fn consult_council_for_strategy(
        &self,
        context: &RefinementContext,
        analysis: &QualityAnalysis,
        iteration: usize,
    ) -> Result<CouncilStrategyDecision, RefinementCoordinatorError> {
        // Create council review request
        let review_request = self.create_council_review_request(context, analysis, iteration);

        // Get council verdict
        let verdict = self.plan_review_service.review_plan(&review_request).await?;

        // Convert verdict to strategy decision
        let votes = verdict.votes.iter()
            .map(|(judge, vote)| (judge.clone(), vote.approved))
            .collect();

        let approved_count = votes.values().filter(|&&approved| approved).count() as f64;
        let total_count = votes.len() as f64;
        let approval_rate = if total_count > 0.0 { approved_count / total_count } else { 0.0 };

        let should_refine = approval_rate >= self.config.council_vote_threshold;

        let reasoning = if should_refine {
            format!("Council approved refinement with {:.1}% approval rate. Issues: {}",
                approval_rate * 100.0,
                verdict.reasoning)
        } else {
            format!("Council rejected refinement with {:.1}% approval rate. Reasoning: {}",
                approval_rate * 100.0,
                verdict.reasoning)
        };

        Ok(CouncilStrategyDecision {
            should_refine,
            votes,
            reasoning,
            confidence: approval_rate,
            council_recommendations: verdict.recommendations.clone(),
        })
    }

    /// Select appropriate refinement strategy
    fn select_refinement_strategy(
        &self,
        context: &RefinementContext,
        analysis: &QualityAnalysis,
        council_decision: &CouncilStrategyDecision,
    ) -> Result<RefinementStrategy, RefinementCoordinatorError> {
        match self.config.strategy_selection_mode {
            StrategySelectionMode::CouncilDriven => {
                // Use council recommendations
                if let Some(recommendation) = council_decision.council_recommendations.first() {
                    Ok(RefinementStrategy::CouncilSpecified(recommendation.clone()))
                } else {
                    self.select_automated_strategy(analysis)
                }
            }
            StrategySelectionMode::PatternBased => {
                self.select_automated_strategy(analysis)
            }
            StrategySelectionMode::Hybrid => {
                // Council validates automated recommendation
                let automated = self.select_automated_strategy(analysis)?;
                if council_decision.council_recommendations.is_empty() {
                    Ok(automated)
                } else {
                    // Council provided specific guidance
                    Ok(RefinementStrategy::CouncilSpecified(
                        council_decision.council_recommendations.join("; ")
                    ))
                }
            }
        }
    }

    /// Select strategy based on quality analysis patterns
    fn select_automated_strategy(
        &self,
        analysis: &QualityAnalysis,
    ) -> Result<RefinementStrategy, RefinementCoordinatorError> {
        // Prioritize based on failing gates
        if analysis.failing_gates.contains(&"caws_compliance".to_string()) {
            return Ok(RefinementStrategy::TargetedFixes(
                vec!["caws_compliance".to_string()]
            ));
        }

        if analysis.failing_gates.contains(&"testing".to_string()) {
            return Ok(RefinementStrategy::EnhanceTesting);
        }

        if analysis.failing_gates.contains(&"type_check".to_string()) {
            return Ok(RefinementStrategy::TargetedFixes(
                vec!["type_check".to_string()]
            ));
        }

        if analysis.failing_gates.contains(&"linting".to_string()) {
            return Ok(RefinementStrategy::TargetedFixes(
                vec!["linting".to_string()]
            ));
        }

        if analysis.failing_gates.len() > 2 {
            return Ok(RefinementStrategy::QualityOverhaul);
        }

        // Default to comprehensive approach
        Ok(RefinementStrategy::TargetedFixes(analysis.failing_gates.clone()))
    }

    /// Determine refinement priority
    fn determine_priority(
        &self,
        _context: &RefinementContext,
        analysis: &QualityAnalysis,
    ) -> RefinementPriority {
        if analysis.has_security_issues {
            return RefinementPriority::Critical;
        }

        if analysis.has_critical_failures || analysis.improvement_needed > 0.2 {
            return RefinementPriority::High;
        }

        if analysis.improvement_needed > 0.1 {
            return RefinementPriority::Medium;
        }

        if analysis.failing_gates.is_empty() && !analysis.warning_gates.is_empty() {
            return RefinementPriority::Low;
        }

        RefinementPriority::Optional
    }

    /// Determine refinement scope
    fn determine_scope(
        &self,
        _context: &RefinementContext,
        analysis: &QualityAnalysis,
    ) -> RefinementScope {
        if analysis.failing_gates.len() > 3 || analysis.has_critical_failures {
            return RefinementScope::Comprehensive;
        }

        if analysis.failing_gates.len() > 1 {
            return RefinementScope::Architectural;
        }

        if !analysis.failing_gates.is_empty() {
            return RefinementScope::Minimal;
        }

        RefinementScope::Minimal
    }

    /// Estimate refinement time
    fn estimate_refinement_time(&self, strategy: &RefinementStrategy, scope: &RefinementScope) -> Option<u64> {
        let base_time = match strategy {
            RefinementStrategy::TargetedFixes(gates) => gates.len() as u64 * 300, // 5 min per gate
            RefinementStrategy::QualityOverhaul => 3600, // 1 hour
            RefinementStrategy::RefactorArchitecture => 7200, // 2 hours
            RefinementStrategy::EnhanceTesting => 1800, // 30 min
            RefinementStrategy::PerformanceOptimization => 2400, // 40 min
            RefinementStrategy::SecurityEnhancement => 3600, // 1 hour
            RefinementStrategy::DocumentationFocus => 1200, // 20 min
            RefinementStrategy::CouncilSpecified(_) => 1800, // 30 min
        };

        let scope_multiplier = match scope {
            RefinementScope::Minimal => 1.0,
            RefinementScope::Comprehensive => 2.0,
            RefinementScope::Architectural => 3.0,
            RefinementScope::Complete => 5.0,
        };

        Some((base_time as f64 * scope_multiplier) as u64)
    }

    /// Create council review request
    fn create_council_review_request(
        &self,
        context: &RefinementContext,
        analysis: &QualityAnalysis,
        iteration: usize,
    ) -> crate::council::plan_review::PlanReviewRequest {
        crate::council::plan_review::PlanReviewRequest {
            task_id: context.quality_report.task_id.to_string(),
            working_spec: context.working_spec.clone(),
            quality_report: Some(context.quality_report.clone()),
            satisficing_analysis: Some(context.satisficing_result.clone()),
            iteration,
            context: format!(
                "Quality Analysis: Score {:.2}, Failing Gates: {:?}, Risk Tier: {:?}",
                analysis.overall_score,
                analysis.failing_gates,
                analysis.risk_tier
            ),
        }
    }
}

/// Quality analysis result
#[derive(Debug, Clone)]
struct QualityAnalysis {
    failing_gates: Vec<String>,
    warning_gates: Vec<String>,
    overall_score: f64,
    improvement_needed: f64,
    has_critical_failures: bool,
    has_security_issues: bool,
    risk_tier: crate::planning::types::RiskTier,
}

/// Council strategy decision
#[derive(Debug, Clone)]
struct CouncilStrategyDecision {
    should_refine: bool,
    votes: HashMap<String, bool>,
    reasoning: String,
    confidence: f64,
    council_recommendations: Vec<String>,
}

pub type Result<T> = std::result::Result<T, RefinementCoordinatorError>;

#[derive(Debug, thiserror::Error)]
pub enum RefinementCoordinatorError {
    #[error("Council consultation failed: {0}")]
    CouncilError(String),

    #[error("Strategy selection failed: {0}")]
    StrategyError(String),

    #[error("Analysis failed: {0}")]
    AnalysisError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

