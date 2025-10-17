//! Predictive learning system components for V3

use std::collections::HashSet;
use tracing::{debug, instrument};

use crate::types::{
    FailureCategory, LearningStrategy, LearningSystemError, PerformancePrediction,
    PredictiveLearningInsights, QualityIndicator, ResourcePrediction, ResourcePressureLevel,
    RiskLevel, StrategyAdjustmentFocus, StrategyAdjustmentSuggestion, StrategyOptimizationPlan,
    TaskLearningSnapshot, TaskOutcome,
};

/// Configuration parameters for predictive learning
#[derive(Debug, Clone)]
pub struct PredictiveLearningConfig {
    pub success_baseline: f64,
    pub partial_penalty: f64,
    pub failure_penalty: f64,
    pub timeout_penalty: f64,
    pub completion_baseline_ms: u64,
}

impl Default for PredictiveLearningConfig {
    fn default() -> Self {
        Self {
            success_baseline: 0.75,
            partial_penalty: 0.25,
            failure_penalty: 0.4,
            timeout_penalty: 0.2,
            completion_baseline_ms: 60_000,
        }
    }
}

/// Predictive learning system orchestrator
#[derive(Debug)]
pub struct PredictiveLearningSystem {
    config: PredictiveLearningConfig,
    performance_predictor: PerformancePredictor,
    strategy_optimizer: StrategyOptimizer,
    resource_predictor: ResourcePredictor,
}

impl PredictiveLearningSystem {
    pub fn new(config: PredictiveLearningConfig) -> Self {
        let performance_predictor = PerformancePredictor::new(config.clone());
        let strategy_optimizer = StrategyOptimizer::new(config.clone());
        let resource_predictor = ResourcePredictor::new(config.clone());

        Self {
            config,
            performance_predictor,
            strategy_optimizer,
            resource_predictor,
        }
    }

    /// Convenience constructor with default configuration
    pub fn with_defaults() -> Self {
        Self::new(PredictiveLearningConfig::default())
    }

    /// Predictive learning workflow entry point
    #[instrument(skip(self, snapshot))]
    pub async fn learn_and_predict(
        &self,
        snapshot: &TaskLearningSnapshot,
    ) -> Result<PredictiveLearningInsights, LearningSystemError> {
        let performance = self.performance_predictor.predict_future(snapshot).await?;
        let strategy = self
            .strategy_optimizer
            .optimize_strategies(snapshot, &performance)
            .await?;
        let resources = self.resource_predictor.predict_needs(snapshot).await?;

        Ok(PredictiveLearningInsights {
            performance,
            strategy,
            resources,
        })
    }

    /// Compatibility helper for the V3 superiority plan signature
    pub async fn learn_and_predict_from_outcome(
        &self,
        outcome: &TaskOutcome,
    ) -> Result<PredictiveLearningInsights, LearningSystemError> {
        let snapshot = TaskLearningSnapshot::from_outcome(outcome.clone());
        self.learn_and_predict(&snapshot).await
    }
}

/// Future performance prediction engine
#[derive(Debug)]
pub struct PerformancePredictor {
    config: PredictiveLearningConfig,
}

impl PerformancePredictor {
    pub fn new(config: PredictiveLearningConfig) -> Self {
        Self { config }
    }

    #[instrument(skip(self, snapshot))]
    pub async fn predict_future(
        &self,
        snapshot: &TaskLearningSnapshot,
    ) -> Result<PerformancePrediction, LearningSystemError> {
        let mut supporting_factors = Vec::new();
        let mut success_probability: f64;
        let mut expected_quality_score: f64;
        let mut predicted_completion_time_ms = self.config.completion_baseline_ms as f64;
        let mut confidence: f64 = 0.6;

        match &snapshot.outcome {
            TaskOutcome::Success {
                confidence: outcome_confidence,
                quality_indicators,
            } => {
                success_probability =
                    self.config.success_baseline + (*outcome_confidence as f64) * 0.2;
                for indicator in quality_indicators {
                    success_probability +=
                        quality_indicator_bonus(indicator, &mut supporting_factors);
                }

                if let Some(progress) = &snapshot.progress_metrics {
                    let quality_delta = (progress.quality_score - 0.7_f64).max(0.0) * 0.15;
                    success_probability += quality_delta;
                    predicted_completion_time_ms *=
                        1.0 - (progress.efficiency_score - 0.6).max(0.0) * 0.3;
                    supporting_factors.push(format!(
                        "Efficiency {:.2} reduces completion time",
                        progress.efficiency_score
                    ));
                    if progress.learning_velocity > 0.1 {
                        supporting_factors.push(format!(
                            "Learning velocity {:.2} supports continued gains",
                            progress.learning_velocity
                        ));
                    }
                } else {
                    predicted_completion_time_ms *= 0.95;
                }

                if let Some(history) = &snapshot.historical_performance {
                    success_probability += (history.success_rate - 0.7).max(0.0) * 0.1;
                    supporting_factors.push(format!(
                        "Historical success rate {:.2} boosts confidence",
                        history.success_rate
                    ));
                }

                expected_quality_score = if let Some(progress) = &snapshot.progress_metrics {
                    ((progress.quality_score + *outcome_confidence as f64) / 2.0
                        + quality_indicators.len() as f64 * 0.015)
                        .min(0.98)
                } else {
                    (*outcome_confidence as f64 + quality_indicators.len() as f64 * 0.02).min(0.98)
                };

                confidence = 0.75 + (*outcome_confidence as f64 * 0.2);
                supporting_factors.push(format!(
                    "Outcome confidence {:.2} indicates sustained performance",
                    outcome_confidence
                ));
            }
            TaskOutcome::PartialSuccess {
                issues,
                confidence: outcome_confidence,
                remediation_applied,
            } => {
                success_probability = self.config.success_baseline - self.config.partial_penalty;
                success_probability += (*outcome_confidence as f64) * 0.15;
                success_probability -= issues.len() as f64 * 0.05;
                if *remediation_applied {
                    success_probability += 0.05;
                    supporting_factors.push("Remediation already applied".to_string());
                } else {
                    supporting_factors.push("Pending remediation work".to_string());
                }

                expected_quality_score = snapshot
                    .progress_metrics
                    .as_ref()
                    .map(|p| (p.quality_score * 0.7).min(0.8))
                    .unwrap_or((*outcome_confidence as f64 * 0.8).min(0.8));

                predicted_completion_time_ms *= 1.1 + (issues.len() as f64 * 0.05);
                supporting_factors.push(format!(
                    "{} outstanding issues require additional iterations",
                    issues.len()
                ));

                if let Some(progress) = &snapshot.progress_metrics {
                    confidence += 0.1;
                    if progress.efficiency_score < 0.6 {
                        supporting_factors.push("Efficiency below target thresholds".to_string());
                    }
                }
            }
            TaskOutcome::Failure {
                failure_category, ..
            } => {
                success_probability = self.config.success_baseline - self.config.failure_penalty;
                expected_quality_score = snapshot
                    .progress_metrics
                    .as_ref()
                    .map(|p| (p.quality_score * 0.6).min(0.7))
                    .unwrap_or(0.45);

                match failure_category {
                    FailureCategory::ResourceExhaustion => {
                        success_probability -= 0.15;
                        predicted_completion_time_ms *= 1.5;
                        supporting_factors
                            .push("Previous attempt exhausted compute capacity".to_string());
                    }
                    FailureCategory::CAWSViolation => {
                        success_probability -= 0.1;
                        predicted_completion_time_ms *= 1.35;
                        supporting_factors
                            .push("Compliance violations require thorough review".to_string());
                    }
                    FailureCategory::ConsensusFailure => {
                        success_probability -= 0.07;
                        predicted_completion_time_ms *= 1.25;
                        supporting_factors
                            .push("Consensus failure indicates misaligned strategies".to_string());
                    }
                    FailureCategory::ClaimVerificationFailure => {
                        success_probability -= 0.09;
                        predicted_completion_time_ms *= 1.4;
                        supporting_factors.push(
                            "Claim verification gap requires additional evidence".to_string(),
                        );
                    }
                    _ => {
                        predicted_completion_time_ms *= 1.25;
                        supporting_factors
                            .push("General failure requires corrective action".into());
                    }
                }

                confidence = 0.55
                    + snapshot
                        .progress_metrics
                        .as_ref()
                        .map(|_| 0.1)
                        .unwrap_or(0.05);
            }
            TaskOutcome::Timeout {
                duration_ms,
                partial_results,
            } => {
                success_probability = self.config.success_baseline - self.config.timeout_penalty;
                expected_quality_score = snapshot
                    .progress_metrics
                    .as_ref()
                    .map(|p| p.quality_score * 0.75)
                    .unwrap_or(0.5);

                let timeout_influence =
                    (*duration_ms as f64 / self.config.completion_baseline_ms as f64).max(1.0);
                predicted_completion_time_ms = (self.config.completion_baseline_ms as f64 * 1.6)
                    .max(*duration_ms as f64 * 1.1);
                supporting_factors.push(format!(
                    "Previous timeout after {} ms extends schedule",
                    duration_ms
                ));

                if let Some(results) = partial_results {
                    success_probability += (results.partial_consensus as f64 * 0.1).min(0.08);
                    supporting_factors.push(format!(
                        "Partial consensus {:.2} boosts momentum",
                        results.partial_consensus
                    ));
                }

                if let Some(progress) = &snapshot.progress_metrics {
                    confidence += 0.1;
                    if progress.completion_percentage > 50.0 {
                        success_probability += 0.05;
                        supporting_factors.push("Meaningful partial progress preserved".into());
                    }
                }

                predicted_completion_time_ms *= timeout_influence.max(1.0);
            }
        }

        if let Some(utilization) = &snapshot.recent_resource_usage {
            if utilization.efficiency_ratio > 0.7 {
                success_probability += 0.04;
                supporting_factors.push("High resource efficiency detected".to_string());
            } else if utilization.efficiency_ratio < 0.45 {
                success_probability -= 0.05;
                supporting_factors.push("Resource efficiency below optimal".to_string());
            }
            confidence += 0.05;
        }

        confidence = confidence
            + snapshot
                .historical_performance
                .as_ref()
                .map(|_| 0.05)
                .unwrap_or(0.0);
        confidence = confidence.clamp(0.4, 0.95);

        success_probability = success_probability.clamp(0.05, 0.995);
        predicted_completion_time_ms = predicted_completion_time_ms
            .max(self.config.completion_baseline_ms as f64 * 0.6)
            .min(self.config.completion_baseline_ms as f64 * 2.2);
        expected_quality_score = expected_quality_score.clamp(0.2, 0.99);

        let risk_level = determine_risk_level(success_probability);
        debug!(
            success_probability,
            expected_quality_score,
            predicted_completion_time_ms,
            ?risk_level,
            "Generated performance prediction"
        );

        Ok(PerformancePrediction {
            expected_quality_score,
            success_probability,
            predicted_completion_time_ms: predicted_completion_time_ms.round() as u64,
            risk_level,
            confidence,
            supporting_factors,
        })
    }
}

/// Proactive strategy optimization engine
#[derive(Debug)]
pub struct StrategyOptimizer {
    config: PredictiveLearningConfig,
}

impl StrategyOptimizer {
    pub fn new(config: PredictiveLearningConfig) -> Self {
        Self { config }
    }

    #[instrument(skip(self, snapshot, performance_prediction))]
    pub async fn optimize_strategies(
        &self,
        snapshot: &TaskLearningSnapshot,
        performance_prediction: &PerformancePrediction,
    ) -> Result<StrategyOptimizationPlan, LearningSystemError> {
        let mut adjustments = Vec::new();
        let mut rationale = Vec::new();
        let mut expected_quality_gain: f64 = 0.06;
        let mut expected_efficiency_gain: f64 = 0.05;
        let mut recommended_strategy = LearningStrategy::Balanced;
        let mut confidence: f64 = 0.7;

        match &snapshot.outcome {
            TaskOutcome::Success {
                quality_indicators, ..
            } => {
                let progress = snapshot.progress_metrics.as_ref();
                let quality_score = progress.map(|p| p.quality_score).unwrap_or(0.8);
                let efficiency_score = progress.map(|p| p.efficiency_score).unwrap_or(0.7);

                if quality_score >= 0.8 && efficiency_score >= 0.7 {
                    recommended_strategy = LearningStrategy::Adaptive;
                    adjustments.push(StrategyAdjustmentSuggestion {
                        focus: StrategyAdjustmentFocus::Exploration,
                        magnitude: 0.15,
                        description:
                            "Reserve turns for exploratory improvements on successful path"
                                .to_string(),
                    });
                    rationale
                        .push("High quality and efficiency support adaptive exploration".into());
                    expected_quality_gain = 0.05;
                    expected_efficiency_gain = 0.04;
                } else {
                    recommended_strategy = LearningStrategy::Balanced;
                    adjustments.push(StrategyAdjustmentSuggestion {
                        focus: StrategyAdjustmentFocus::Efficiency,
                        magnitude: 0.12,
                        description: "Reinforce efficiency heuristics to maintain pace".to_string(),
                    });
                    rationale.push("Quality is strong but efficiency can improve".into());
                }

                if quality_indicators.contains(&QualityIndicator::StrongCAWSCompliance) {
                    adjustments.push(StrategyAdjustmentSuggestion {
                        focus: StrategyAdjustmentFocus::Context,
                        magnitude: 0.08,
                        description: "Preserve compliance-sensitive context windows".to_string(),
                    });
                }

                confidence = (performance_prediction.confidence * 0.9).clamp(0.75, 0.9);
            }
            TaskOutcome::PartialSuccess {
                issues,
                remediation_applied,
                ..
            } => {
                recommended_strategy = LearningStrategy::Adaptive;
                adjustments.push(StrategyAdjustmentSuggestion {
                    focus: StrategyAdjustmentFocus::Quality,
                    magnitude: 0.22,
                    description: "Introduce targeted review turns to resolve outstanding issues"
                        .to_string(),
                });
                adjustments.push(StrategyAdjustmentSuggestion {
                    focus: StrategyAdjustmentFocus::Context,
                    magnitude: 0.16,
                    description: "Increase context preservation for remediation artifacts"
                        .to_string(),
                });
                rationale.push(format!(
                    "Partial success with {} outstanding issue(s)",
                    issues.len()
                ));
                if !remediation_applied {
                    rationale.push("Remediation plan pending; schedule guided follow-up".into());
                }

                expected_quality_gain = 0.12;
                expected_efficiency_gain = 0.07;
                confidence = (performance_prediction.confidence * 0.85).clamp(0.65, 0.82);
            }
            TaskOutcome::Failure {
                failure_category, ..
            } => {
                recommended_strategy = match failure_category {
                    FailureCategory::ResourceExhaustion => LearningStrategy::Conservative,
                    FailureCategory::CAWSViolation => LearningStrategy::Conservative,
                    FailureCategory::ConsensusFailure => LearningStrategy::Balanced,
                    _ => LearningStrategy::Conservative,
                };

                adjustments.push(StrategyAdjustmentSuggestion {
                    focus: StrategyAdjustmentFocus::Resource,
                    magnitude: 0.28,
                    description: "Throttle parallelism and prioritize efficient primitives".into(),
                });
                adjustments.push(StrategyAdjustmentSuggestion {
                    focus: StrategyAdjustmentFocus::Quality,
                    magnitude: 0.18,
                    description: "Introduce fail-fast checks before committing to strategy".into(),
                });

                rationale.push(format!(
                    "Failure category {:?} necessitates protective strategy",
                    failure_category
                ));

                expected_quality_gain = 0.09;
                expected_efficiency_gain = 0.2;
                confidence = (performance_prediction.confidence * 0.9).clamp(0.6, 0.78);
            }
            TaskOutcome::Timeout { .. } => {
                recommended_strategy = LearningStrategy::Conservative;
                adjustments.push(StrategyAdjustmentSuggestion {
                    focus: StrategyAdjustmentFocus::Efficiency,
                    magnitude: 0.2,
                    description: "Insert interim checkpoints to avoid repeating timeouts".into(),
                });
                adjustments.push(StrategyAdjustmentSuggestion {
                    focus: StrategyAdjustmentFocus::Resource,
                    magnitude: 0.17,
                    description: "Pre-allocate resource buffer before next attempt".into(),
                });

                rationale.push("Timeout indicates need for finer-grained pacing".into());

                expected_quality_gain = 0.08;
                expected_efficiency_gain = 0.16;
                confidence = (performance_prediction.confidence * 0.88).clamp(0.62, 0.8);
            }
        }

        let mut unique_foci = HashSet::new();
        adjustments.retain(|adj| unique_foci.insert(adj.focus.clone()));

        Ok(StrategyOptimizationPlan {
            recommended_strategy,
            adjustments,
            expected_quality_gain,
            expected_efficiency_gain,
            confidence,
            rationale,
        })
    }
}

/// Resource requirement prediction engine
#[derive(Debug)]
pub struct ResourcePredictor {
    config: PredictiveLearningConfig,
}

impl ResourcePredictor {
    pub fn new(config: PredictiveLearningConfig) -> Self {
        Self { config }
    }

    #[instrument(skip(self, snapshot))]
    pub async fn predict_needs(
        &self,
        snapshot: &TaskLearningSnapshot,
    ) -> Result<ResourcePrediction, LearningSystemError> {
        let mut cpu_usage: f64 = 0.5;
        let mut memory_mb: f64 = 1_024.0;
        let mut token_usage: f64 = 2_400.0;
        let mut duration_ms = self.config.completion_baseline_ms;
        let mut pressure_score: f64 = 0.5;
        let mut bottlenecks = Vec::new();

        if let Some(utilization) = &snapshot.recent_resource_usage {
            cpu_usage = utilization.cpu_usage.clamp(0.0, 1.0);
            memory_mb = (utilization.memory_usage * 16_384.0).clamp(512.0, 24_576.0);
            token_usage = (utilization.token_usage * 10_000.0).clamp(800.0, 18_000.0);
            duration_ms = ((self.config.completion_baseline_ms as f64)
                * (1.0 + utilization.time_usage.clamp(0.0, 1.5)))
            .round() as u64;
            pressure_score = pressure_score
                .max(utilization.cpu_usage)
                .max(utilization.memory_usage)
                .max(utilization.token_usage)
                .max(utilization.time_usage);

            if utilization.efficiency_ratio < 0.5 {
                bottlenecks.push("Efficiency ratio below target threshold".into());
                pressure_score = pressure_score.max(0.75);
            }
        }

        match &snapshot.outcome {
            TaskOutcome::Success {
                quality_indicators, ..
            } => {
                if quality_indicators
                    .iter()
                    .any(|indicator| matches!(indicator, QualityIndicator::EfficientExecution))
                {
                    cpu_usage = (cpu_usage * 0.85).max(0.35);
                    duration_ms = ((duration_ms as f64) * 0.85) as u64;
                    pressure_score *= 0.85;
                    bottlenecks.push("Efficient execution reduces future load".into());
                }
                if quality_indicators
                    .iter()
                    .any(|indicator| matches!(indicator, QualityIndicator::ComprehensiveEvidence))
                {
                    token_usage *= 1.05;
                    bottlenecks
                        .push("Comprehensive evidence increases token demand slightly".into());
                }
            }
            TaskOutcome::PartialSuccess { issues, .. } => {
                let penalty = (issues.len() as f64 * 0.05).min(0.2);
                cpu_usage = (cpu_usage + penalty).min(0.95);
                memory_mb *= 1.0 + penalty;
                token_usage *= 1.0 + penalty;
                duration_ms = ((duration_ms as f64) * (1.1 + penalty / 2.0)) as u64;
                pressure_score = pressure_score.max(cpu_usage).max(0.65);
                if !issues.is_empty() {
                    bottlenecks.push("Outstanding issues demand extra passes".into());
                }
            }
            TaskOutcome::Failure {
                failure_category, ..
            } => match failure_category {
                FailureCategory::ResourceExhaustion => {
                    cpu_usage = (cpu_usage + 0.1).min(0.99);
                    memory_mb *= 1.4;
                    token_usage *= 1.35;
                    duration_ms = ((duration_ms as f64) * 1.5) as u64;
                    pressure_score = pressure_score.max(0.95);
                    bottlenecks.push("Prior run exhausted compute resources".into());
                }
                FailureCategory::CAWSViolation => {
                    duration_ms = ((duration_ms as f64) * 1.35) as u64;
                    token_usage *= 1.2;
                    pressure_score = pressure_score.max(0.7);
                    bottlenecks.push("Compliance pass adds verification overhead".into());
                }
                FailureCategory::ConsensusFailure => {
                    cpu_usage = (cpu_usage + 0.08).min(0.92);
                    duration_ms = ((duration_ms as f64) * 1.25) as u64;
                    pressure_score = pressure_score.max(0.72);
                    bottlenecks.push("Consensus recovery requires iterative debate".into());
                }
                _ => {
                    cpu_usage = (cpu_usage + 0.05).min(0.9);
                    duration_ms = ((duration_ms as f64) * 1.2) as u64;
                    pressure_score = pressure_score.max(0.68);
                    bottlenecks.push("Failure recovery requires protective buffer".into());
                }
            },
            TaskOutcome::Timeout {
                duration_ms: timeout_ms,
                ..
            } => {
                cpu_usage = (cpu_usage + 0.12).min(0.9);
                duration_ms = ((*timeout_ms as f64 * 1.1)
                    .max(self.config.completion_baseline_ms as f64 * 1.6))
                .round() as u64;
                token_usage *= 1.1;
                pressure_score = pressure_score.max(0.78);
                bottlenecks.push("Previous timeout indicates longer execution window".into());
            }
        }

        if let Some(progress) = &snapshot.progress_metrics {
            if progress.efficiency_score < 0.55 {
                cpu_usage = (cpu_usage + 0.05).min(0.98);
                pressure_score = pressure_score.max(cpu_usage);
                bottlenecks.push("Recent efficiency dip increases compute demand".into());
            }
            if progress.error_rate > 0.15 {
                duration_ms = ((duration_ms as f64) * 1.1) as u64;
                bottlenecks.push("Elevated error rate increases iteration count".into());
            }
        }

        let pressure_level = match pressure_score {
            score if score >= 0.9 => ResourcePressureLevel::Critical,
            score if score >= 0.75 => ResourcePressureLevel::High,
            score if score >= 0.6 => ResourcePressureLevel::Moderate,
            _ => ResourcePressureLevel::Low,
        };

        let mut confidence: f64 = 0.6;
        if snapshot.recent_resource_usage.is_some() {
            confidence += 0.2;
        }
        if snapshot.progress_metrics.is_some() {
            confidence += 0.1;
        }
        confidence = confidence.min(0.9);

        deduplicate(&mut bottlenecks);

        debug!(
            cpu_usage,
            memory_mb,
            token_usage,
            duration_ms,
            ?pressure_level,
            "Generated resource prediction"
        );

        Ok(ResourcePrediction {
            expected_cpu_usage: cpu_usage,
            expected_memory_mb: memory_mb,
            expected_token_usage: token_usage,
            expected_duration_ms: duration_ms,
            pressure_level,
            confidence,
            bottlenecks,
        })
    }
}

fn determine_risk_level(success_probability: f64) -> RiskLevel {
    if success_probability >= 0.85 {
        RiskLevel::Low
    } else if success_probability >= 0.65 {
        RiskLevel::Medium
    } else if success_probability >= 0.35 {
        RiskLevel::High
    } else {
        RiskLevel::Critical
    }
}

fn quality_indicator_bonus(indicator: &QualityIndicator, factors: &mut Vec<String>) -> f64 {
    match indicator {
        QualityIndicator::HighConfidence => {
            factors.push("High confidence score achieved by council".into());
            0.08
        }
        QualityIndicator::ComprehensiveEvidence => {
            factors.push("Comprehensive evidence validated".into());
            0.06
        }
        QualityIndicator::MinimalDissent => {
            factors.push("Minimal dissent recorded".into());
            0.05
        }
        QualityIndicator::EfficientExecution => {
            factors.push("Execution efficiency exceeded baseline".into());
            0.05
        }
        QualityIndicator::StrongCAWSCompliance => {
            factors.push("Strong CAWS compliance maintained".into());
            0.07
        }
        QualityIndicator::CompleteClaimVerification => {
            factors.push("Claim verification complete with evidence".into());
            0.06
        }
    }
}

fn deduplicate(items: &mut Vec<String>) {
    let mut seen = HashSet::new();
    items.retain(|item| seen.insert(item.clone()));
}
