//! Council Coordinator
//!
//! Generic coordinator that composes judges with an inference engine.
//! Engine-agnostic design allows for different inference backends (CoreML, API, etc.).

use std::sync::Arc;
use async_trait::async_trait;
use tracing::{info, instrument, warn};

use agent_agency_contracts::{JudgeEngine, JudgeType, VerdictLabel, JudgeVerdict, WorkingSpec};
use agent_agency_contracts::judge_io::Severity;

use crate::{Judges, FinalDecision, CouncilError, CouncilMetrics, CouncilResult};
use crate::judges::Judge;


/// Review context for a working spec
#[derive(Debug, Clone)]
pub struct ReviewContext {
    /// The working spec being reviewed
    pub working_spec: WorkingSpec,

    /// Additional context for the review
    pub context: std::collections::HashMap<String, serde_json::Value>,

    /// Whether this is a high-priority review
    pub priority: ReviewPriority,
}

/// Review priority levels
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ReviewPriority {
    Low,
    Normal,
    High,
    Critical,
}

/// Council coordinator generic over engine type
#[derive(Debug)]
pub struct CouncilCoordinator<E: JudgeEngine> {
    /// Inference engine for judges
    engine: Arc<E>,

    /// The four constitutional judges
    judges: Judges,

    /// Decision aggregator for consensus
    aggregator: VerdictAggregator,

    /// Decision engine for final judgments
    decision_engine: DecisionEngine,

    /// Performance and observability metrics
    metrics: CouncilMetrics,
}

impl<E: JudgeEngine> CouncilCoordinator<E> {
    /// Create new council coordinator
    pub fn new(engine: Arc<E>, judges: Judges) -> Self {
        Self {
            engine,
            judges,
            aggregator: VerdictAggregator::default(),
            decision_engine: DecisionEngine::default(),
            metrics: CouncilMetrics::new(),
        }
    }

    /// Evaluate a working spec through all judges
    #[instrument(skip(self, ctx), fields(spec_id = %ctx.working_spec.id))]
    pub async fn evaluate(&mut self, ctx: &ReviewContext) -> CouncilResult<FinalDecision> {
        let start = std::time::Instant::now();
        self.metrics.record_session();

        info!("🏛️  Constitutional Council evaluating spec {}", ctx.working_spec.id);

        // Collect verdicts from all four judges
        let judge_verdicts_with_types = self.collect_verdicts(ctx).await?;

        // Extract just the verdicts for aggregation
        let judge_verdicts: Vec<_> = judge_verdicts_with_types.iter().map(|(_, v)| v.clone()).collect();

        // Aggregate into consensus
        let aggregation = self.aggregator.aggregate(&judge_verdicts)?;

        // Make final decision
        let decision = self.decision_engine.decide(ctx, &aggregation)?;

        // Record metrics
        let duration = start.elapsed();
        self.metrics.record_evaluation(duration, &judge_verdicts_with_types, &decision);

        info!(
            "🏛️  Council decision: {} (score: {:.2}, duration: {:.1}ms)",
            match decision.label {
                VerdictLabel::Pass => "PASS",
                VerdictLabel::Fail => "FAIL",
                VerdictLabel::NeedsInfo => "NEEDS INFO",
                VerdictLabel::Conditional => "CONDITIONAL",
            },
            decision.score,
            duration.as_millis()
        );

        Ok(decision)
    }

    /// Collect verdicts from all judges concurrently
    async fn collect_verdicts(&self, ctx: &ReviewContext) -> CouncilResult<Vec<(JudgeType, JudgeVerdict)>> {
        use futures::future::join_all;

        let futures = vec![
            (JudgeType::Constitutional, self.judges.constitutional.review_spec(ctx)),
            (JudgeType::Technical, self.judges.technical.review_spec(ctx)),
            (JudgeType::Quality, self.judges.quality.review_spec(ctx)),
            (JudgeType::Integration, self.judges.integration.review_spec(ctx)),
        ];

        let results = join_all(futures.into_iter().map(|(jt, fut)| async move {
            match fut.await {
                Ok(verdict) => Ok((jt, verdict)),
                Err(e) => Err((jt, e)),
            }
        })).await;

        // Check for judge failures
        let mut verdicts = Vec::new();
        for result in results {
            match result {
                Ok((judge_type, verdict)) => verdicts.push((judge_type, verdict)),
                Err((judge_type, e)) => {
                    warn!("Judge {:?} returned error: {}", judge_type, e);
                    return Err(CouncilError::Judge(format!("Judge {:?}: {}", judge_type, e)));
                }
            }
        }

        Ok(verdicts)
    }

    /// Get council metrics
    pub fn metrics(&self) -> &CouncilMetrics {
        &self.metrics
    }
}

/// Aggregates judge verdicts into consensus
#[derive(Debug, Default)]
struct VerdictAggregator;

impl VerdictAggregator {
    /// Aggregate multiple judge verdicts
    fn aggregate(&self, verdicts: &[JudgeVerdict]) -> CouncilResult<VerdictAggregation> {
        if verdicts.is_empty() {
            return Err(CouncilError::Consensus("No verdicts to aggregate".to_string()));
        }

        // Calculate weighted scores (some judges may have higher weight)
        let total_score: f32 = verdicts.iter().map(|v| v.score).sum();
        let average_score = total_score / verdicts.len() as f32;

        // Determine consensus label
        let label_counts = verdicts.iter()
            .fold(std::collections::HashMap::new(), |mut acc, v| {
                *acc.entry(&v.label).or_insert(0) += 1;
                acc
            });

        let consensus_label = label_counts.iter()
            .max_by_key(|(_, count)| *count)
            .map(|(label, _)| **label)
            .unwrap_or(VerdictLabel::NeedsInfo);

        // Check for consensus violations (high disagreement)
        let consensus_violations = self.check_consensus_violations(verdicts, &consensus_label);

        Ok(VerdictAggregation {
            average_score,
            consensus_label,
            consensus_violations,
            all_verdicts: verdicts.to_vec(),
        })
    }

    /// Check if there's significant disagreement among judges
    fn check_consensus_violations(&self, verdicts: &[JudgeVerdict], consensus: &VerdictLabel) -> Vec<String> {
        let mut violations = Vec::new();

        // Count dissenting votes
        let dissent_count = verdicts.iter()
            .filter(|v| &v.label != consensus)
            .count();

        if dissent_count > verdicts.len() / 2 {
            violations.push(format!(
                "Majority disagreement: {} out of {} judges dissent from consensus",
                dissent_count, verdicts.len()
            ));
        }

        // Check for critical violations that override consensus
        let critical_violations: Vec<_> = verdicts.iter()
            .flat_map(|v| &v.violations)
            .filter(|v| v.severity == Severity::Critical)
            .collect();

        if !critical_violations.is_empty() {
            violations.push(format!(
                "Critical violations present: {} critical issues found",
                critical_violations.len()
            ));
        }

        violations
    }
}

/// Aggregation of multiple judge verdicts
#[derive(Debug)]
struct VerdictAggregation {
    /// Average score across all judges
    average_score: f32,

    /// Consensus label (majority vote)
    consensus_label: VerdictLabel,

    /// Any consensus violations detected
    consensus_violations: Vec<String>,

    /// All individual verdicts
    all_verdicts: Vec<JudgeVerdict>,
}

/// Makes final decisions based on verdict aggregation
#[derive(Debug, Default)]
struct DecisionEngine;

impl DecisionEngine {
    /// Make final decision from aggregated verdicts
    fn decide(&self, ctx: &ReviewContext, aggregation: &VerdictAggregation) -> CouncilResult<FinalDecision> {
        // Check for blocking consensus violations
        if !aggregation.consensus_violations.is_empty() {
            // If there are critical violations, always fail
            let has_critical = aggregation.all_verdicts.iter()
                .any(|v| v.violations.iter().any(|vi| vi.severity == Severity::Critical));

            if has_critical {
                return Ok(FinalDecision {
                    label: VerdictLabel::Fail,
                    score: 0.1, // Low confidence due to critical issues
                    rationale: format!(
                        "Rejected due to critical violations: {}",
                        aggregation.consensus_violations.join(", ")
                    ),
                    judge_verdicts: aggregation.all_verdicts.clone(),
                    consensus_violations: aggregation.consensus_violations.clone(),
                    recommended_actions: vec![
                        "Address critical violations before proceeding".to_string(),
                        "Review spec against CAWS invariants".to_string(),
                        "Consider breaking down into smaller changes".to_string(),
                    ],
                });
            }
        }

        // Use consensus label and average score
        let score = aggregation.average_score;
        let label = aggregation.consensus_label.clone();

        // Generate rationale based on context and verdicts
        let rationale = self.generate_rationale(ctx, aggregation);

        // Generate recommended actions
        let recommended_actions = self.generate_actions(ctx, aggregation);

        Ok(FinalDecision {
            label,
            score,
            rationale,
            judge_verdicts: aggregation.all_verdicts.clone(),
            consensus_violations: aggregation.consensus_violations.clone(),
            recommended_actions,
        })
    }

    /// Generate rationale for the decision
    fn generate_rationale(&self, _ctx: &ReviewContext, aggregation: &VerdictAggregation) -> String {
        match aggregation.consensus_label {
            VerdictLabel::Pass => {
                format!(
                    "Spec approved by constitutional council with average score {:.2}. All judges found the spec compliant with CAWS standards.",
                    aggregation.average_score
                )
            }
            VerdictLabel::Fail => {
                format!(
                    "Spec rejected by constitutional council with average score {:.2}. Critical compliance issues identified.",
                    aggregation.average_score
                )
            }
            VerdictLabel::NeedsInfo => {
                format!(
                    "Spec requires additional information before approval. Average score {:.2} indicates uncertainty among judges.",
                    aggregation.average_score
                )
            }
            VerdictLabel::Conditional => {
                format!(
                    "Spec conditionally approved with average score {:.2}. Some concerns remain but are not blocking.",
                    aggregation.average_score
                )
            }
        }
    }

    /// Generate recommended actions based on decision
    fn generate_actions(&self, _ctx: &ReviewContext, aggregation: &VerdictAggregation) -> Vec<String> {
        match aggregation.consensus_label {
            VerdictLabel::Pass => vec![
                "Proceed with implementation".to_string(),
                "Monitor for runtime compliance".to_string(),
                "Document successful patterns".to_string(),
            ],
            VerdictLabel::Fail => vec![
                "Address identified violations".to_string(),
                "Re-submit after fixes".to_string(),
                "Consider consulting with judges for guidance".to_string(),
            ],
            VerdictLabel::NeedsInfo => vec![
                "Provide additional context or evidence".to_string(),
                "Clarify ambiguous requirements".to_string(),
                "Break down complex changes".to_string(),
            ],
            VerdictLabel::Conditional => vec![
                "Address non-critical concerns".to_string(),
                "Implement with monitoring".to_string(),
                "Plan for follow-up review".to_string(),
            ],
        }
    }
}
