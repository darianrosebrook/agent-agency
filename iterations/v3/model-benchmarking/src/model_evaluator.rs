//! Model evaluator for new model assessment

use crate::types::*;
use anyhow::Result;

/// Model evaluator for assessing new models
pub struct ModelEvaluator {}

impl ModelEvaluator {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn evaluate_model(&self, _model: &ModelSpecification) -> Result<EvaluationMetrics> {
        let capability_scores = self.calculate_capability_scores(_model);
        let performance_metrics = self.calculate_performance_metrics(_model, &capability_scores);
        let overall_score = Self::weighted_overall_score(&performance_metrics);
        let compliance_score = performance_metrics.compliance;

        Ok(EvaluationMetrics {
            overall_score,
            capability_scores,
            performance_metrics,
            compliance_score,
        })
    }

    pub async fn compare_against_baseline(
        &self,
        _model: &ModelSpecification,
        metrics: &EvaluationMetrics,
    ) -> Result<ComparisonResult> {
        let baseline = BenchmarkMetrics {
            accuracy: 0.75,
            speed: 0.7,
            efficiency: 0.7,
            quality: 0.75,
            compliance: 0.7,
        };
        let baseline_score = 0.72;
        let metric_pairs = vec![
            ("accuracy", metrics.performance_metrics.accuracy, baseline.accuracy),
            ("speed", metrics.performance_metrics.speed, baseline.speed),
            (
                "efficiency",
                metrics.performance_metrics.efficiency,
                baseline.efficiency,
            ),
            (
                "quality",
                metrics.performance_metrics.quality,
                baseline.quality,
            ),
            (
                "compliance",
                metrics.performance_metrics.compliance,
                baseline.compliance,
            ),
            ("score", metrics.overall_score, baseline_score),
        ];

        let mut improvement_areas = Vec::new();
        let mut regression_areas = Vec::new();
        let mut positive_deltas = Vec::new();

        for (name, current, reference) in metric_pairs {
            let delta = current - reference;
            if delta > 0.03 {
                improvement_areas.push(name.to_string());
                if reference > f64::EPSILON {
                    positive_deltas.push(delta / reference);
                }
            } else if delta < -0.03 {
                regression_areas.push(name.to_string());
            }
        }

        let improvement_percentage = if positive_deltas.is_empty() {
            0.0
        } else {
            (positive_deltas.iter().sum::<f64>() / positive_deltas.len() as f64) * 100.0
        };

        let recommendation = if regression_areas.is_empty() {
            if improvement_areas.is_empty() {
                "Model performance aligns with established baselines.".to_string()
            } else {
                format!(
                    "Model exceeds baseline in {}.",
                    improvement_areas.join(", ")
                )
            }
        } else {
            format!(
                "Performance regressions detected in {}.",
                regression_areas.join(", ")
            )
        };

        Ok(ComparisonResult {
            improvement_percentage,
            regression_areas,
            improvement_areas,
            recommendation,
        })
    }

    pub async fn generate_recommendation(
        &self,
        _model: &ModelSpecification,
        metrics: &EvaluationMetrics,
        comparison: &ComparisonResult,
    ) -> Result<ModelRecommendation> {
        let has_regressions = !comparison.regression_areas.is_empty();
        let overall = metrics.overall_score;
        let decision = if has_regressions && overall < 0.75 {
            RecommendationDecision::FurtherEvaluation
        } else if has_regressions {
            RecommendationDecision::ConditionalAdopt
        } else if overall >= 0.82 {
            RecommendationDecision::Adopt
        } else {
            RecommendationDecision::ConditionalAdopt
        };

        let mut conditions = Vec::new();
        if has_regressions {
            for area in &comparison.regression_areas {
                conditions.push(Condition {
                    condition_type: ConditionType::PerformanceImprovement,
                    description: format!("Address regression in {area} before rollout"),
                    required: true,
                });
            }
        }

        let reasoning = if has_regressions {
            format!(
                "Observed regressions in {} requiring attention.",
                comparison.regression_areas.join(", ")
            )
        } else if !comparison.improvement_areas.is_empty() {
            format!(
                "Model shows improvements in {}; monitor for sustained gains.",
                comparison.improvement_areas.join(", ")
            )
        } else {
            "Model aligns with baseline expectations; continue iterative validation.".to_string()
        };

        Ok(ModelRecommendation {
            recommendation: decision,
            reasoning,
            confidence: overall.clamp(0.3, 0.95),
            conditions,
        })
    }
}

impl ModelEvaluator {
    fn calculate_capability_scores(&self, model: &ModelSpecification) -> Vec<CapabilityScore> {
        model
            .capabilities
            .iter()
            .map(|capability| {
                let base_score = Self::proficiency_to_score(&capability.proficiency_level);
                let domain_bonus =
                    (capability.supported_domains.len() as f64 * 0.02).min(0.1);
                CapabilityScore {
                    capability: capability.capability_type.clone(),
                    score: (base_score + domain_bonus).min(0.95),
                    confidence: (0.55 + domain_bonus).min(0.95),
                }
            })
            .collect()
    }

    fn calculate_performance_metrics(
        &self,
        model: &ModelSpecification,
        capability_scores: &[CapabilityScore],
    ) -> BenchmarkMetrics {
        let average_capability = if capability_scores.is_empty() {
            0.6
        } else {
            capability_scores
                .iter()
                .map(|score| score.score)
                .sum::<f64>()
                / capability_scores.len() as f64
        };

        let size_mb = model.parameters.size as f64 / 1_000_000.0;
        let speed = (1.0 / (1.0 + (size_mb / 120.0))).clamp(0.35, 0.95);

        let context_factor =
            (model.parameters.context_length as f64 / 8192.0).clamp(0.5, 2.0);
        let efficiency = (0.9 / context_factor).clamp(0.4, 0.95);

        let domain_span =
            model.capabilities.iter().map(|c| c.supported_domains.len()).sum::<usize>() as f64;
        let quality = (average_capability * 0.75 + (0.65 + domain_span * 0.01) * 0.25)
            .clamp(0.4, 0.96);

        let compliance = {
            let compliance_capabilities = model
                .capabilities
                .iter()
                .filter(|cap| {
                    matches!(
                        cap.capability_type,
                        CapabilityType::CodeReview
                            | CapabilityType::Analysis
                            | CapabilityType::Testing
                            | CapabilityType::Research
                    )
                })
                .count();
            (0.58 + compliance_capabilities as f64 * 0.05).min(0.95)
        };

        BenchmarkMetrics {
            accuracy: average_capability,
            speed,
            efficiency,
            quality,
            compliance,
        }
    }

    fn weighted_overall_score(metrics: &BenchmarkMetrics) -> f64 {
        metrics.accuracy * 0.3
            + metrics.quality * 0.25
            + metrics.efficiency * 0.2
            + metrics.speed * 0.15
            + metrics.compliance * 0.1
    }

    fn proficiency_to_score(level: &ProficiencyLevel) -> f64 {
        match level {
            ProficiencyLevel::Basic => 0.55,
            ProficiencyLevel::Intermediate => 0.68,
            ProficiencyLevel::Advanced => 0.8,
            ProficiencyLevel::Expert => 0.9,
        }
    }
}
