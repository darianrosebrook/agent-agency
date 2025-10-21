//! Satisficing evaluator that determines when to stop iterating

use std::collections::HashMap;
use chrono::{DateTime, Utc};

use super::{EvalReport, EvalStatus, StopReason};

/// Satisficing configuration
#[derive(Debug, Clone)]
pub struct SatisficingConfig {
    pub max_iterations: usize,
    pub min_improvement_threshold: f64,
    pub quality_ceiling_budget: usize,
    pub cost_benefit_ratio_threshold: f64,
    pub mandatory_gates: Vec<String>,
    // Hysteresis and no-progress guards
    pub hysteresis_window: usize,         // Sliding window size for hysteresis
    pub consecutive_threshold: usize,     // Consecutive sub-threshold iterations to trigger
    pub no_progress_loc_threshold: usize, // Maximum LOC change to consider "no progress"
}

impl Default for SatisficingConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            min_improvement_threshold: 0.02, // 2% improvement
            quality_ceiling_budget: 3, // Check for ceiling over last 3 iterations
            cost_benefit_ratio_threshold: 0.1, // Minimum cost-benefit ratio
            mandatory_gates: vec![
                "tests-pass".to_string(),
                "lint-clean".to_string(),
                "types-ok".to_string(),
            ],
        }
    }
}

/// Satisficing decision result
#[derive(Debug, Clone)]
pub struct SatisficingDecision {
    pub should_continue: bool,
    pub reason: StopReason,
    pub confidence: f64,
    pub recommendations: Vec<String>,
}

/// Satisficing evaluator
pub struct SatisficingEvaluator {
    config: SatisficingConfig,
}

impl SatisficingEvaluator {
    /// Create a new satisficing evaluator
    pub fn new() -> Self {
        Self {
            config: SatisficingConfig::default(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: SatisficingConfig) -> Self {
        Self { config }
    }

    /// Decide whether to continue iterating based on evaluation results
    pub fn should_continue(
        &mut self,
        current: &EvalReport,
        history: &[EvalReport],
    ) -> SatisficingDecision {
        // Check mandatory gates first
        if let Some(reason) = self.check_mandatory_gates(current) {
            return SatisficingDecision {
                should_continue: false,
                reason,
                confidence: 1.0,
                recommendations: vec!["Fix mandatory gate failures before continuing".to_string()],
            };
        }

        // Check if we've reached max iterations
        if history.len() >= self.config.max_iterations {
            return SatisficingDecision {
                should_continue: false,
                reason: StopReason::MaxIterations,
                confidence: 1.0,
                recommendations: vec!["Reached maximum iteration limit".to_string()],
            };
        }

        // Check if quality ceiling has been reached
        if let Some(ceiling_reason) = self.check_quality_ceiling(current, history) {
            return SatisficingDecision {
                should_continue: false,
                reason: ceiling_reason,
                confidence: 0.8,
                recommendations: vec!["Quality improvements have plateaued".to_string()],
            };
        }

        // Check if satisficed (good enough)
        if current.status == EvalStatus::Pass {
            return SatisficingDecision {
                should_continue: false,
                reason: StopReason::Satisficed,
                confidence: 0.9,
                recommendations: vec!["Quality requirements met".to_string()],
            };
        }

        // Check cost-benefit ratio
        if let Some(cost_decision) = self.check_cost_benefit(current, history) {
            return cost_decision;
        }

        // Continue iterating
        let recommendations = self.generate_iteration_recommendations(current, history);

        SatisficingDecision {
            should_continue: true,
            reason: StopReason::Unknown, // Not applicable when continuing
            confidence: 0.6,
            recommendations,
        }
    }

    /// Check mandatory gates (tests, lint, types, etc.)
    fn check_mandatory_gates(&self, current: &EvalReport) -> Option<StopReason> {
        for gate in &self.config.mandatory_gates {
            if !current.thresholds_met.contains(gate) {
                return Some(StopReason::FailedGates);
            }
        }
        None
    }

    /// Check if quality improvements have plateaued
    fn check_quality_ceiling(&self, current: &EvalReport, history: &[EvalReport]) -> Option<StopReason> {
        if history.len() < self.config.quality_ceiling_budget {
            return None;
        }

        // Look at the last N iterations
        let recent_scores: Vec<f64> = history.iter()
            .rev()
            .take(self.config.quality_ceiling_budget)
            .map(|r| r.score)
            .collect();

        let max_recent = recent_scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let min_improvement = self.config.min_improvement_threshold;

        // Check if current score is not significantly better than recent maximum
        if (current.score - max_recent).abs() < min_improvement {
            // Check if we've been stuck at this level for multiple iterations
            let stuck_count = recent_scores.iter()
                .filter(|&&score| (score - max_recent).abs() < min_improvement)
                .count();

            if stuck_count >= self.config.quality_ceiling_budget - 1 {
                return Some(StopReason::QualityCeiling);
            }
        }

        None
    }

    /// Check cost-benefit ratio of continuing
    fn check_cost_benefit(&self, current: &EvalReport, history: &[EvalReport]) -> Option<SatisficingDecision> {
        if history.is_empty() {
            return None;
        }

        // Calculate improvement trend
        let improvements: Vec<f64> = history.iter()
            .zip(history.iter().skip(1))
            .map(|(prev, curr)| curr.score - prev.score)
            .collect();

        if improvements.is_empty() {
            return None;
        }

        let avg_improvement = improvements.iter().sum::<f64>() / improvements.len() as f64;

        // Estimate cost (iterations, time, tokens)
        let estimated_iterations_to_target = if current.score < 0.8 {
            ((0.8 - current.score) / avg_improvement.max(0.01)).ceil() as usize
        } else {
            1
        };

        let estimated_cost = estimated_iterations_to_target as f64;
        let estimated_benefit = (0.8 - current.score).max(0.0);

        let cost_benefit_ratio = if estimated_cost > 0.0 {
            estimated_benefit / estimated_cost
        } else {
            0.0
        };

        if cost_benefit_ratio < self.config.cost_benefit_ratio_threshold {
            return Some(SatisficingDecision {
                should_continue: false,
                reason: StopReason::QualityCeiling,
                confidence: 0.7,
                recommendations: vec![
                    format!("Cost-benefit ratio too low: {:.3}", cost_benefit_ratio),
                    format!("Estimated {} more iterations needed for minimal improvement", estimated_iterations_to_target),
                ],
            });
        }

        None
    }

    /// Generate recommendations for the next iteration
    fn generate_iteration_recommendations(&self, current: &EvalReport, history: &[EvalReport]) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Analyze failed criteria
        let failed_criteria: Vec<_> = current.criteria.iter()
            .filter(|c| !c.passed)
            .collect();

        if !failed_criteria.is_empty() {
            recommendations.push(format!("Address {} failed criteria", failed_criteria.len()));

            // Prioritize by weight
            let mut prioritized: Vec<_> = failed_criteria.iter()
                .map(|c| (c.weight, c.id.clone()))
                .collect();

            prioritized.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

            for (weight, criterion_id) in prioritized.into_iter().take(3) {
                recommendations.push(format!("Fix '{}' (weight: {:.2})", criterion_id, weight));
            }
        }

        // Analyze improvement trend
        if history.len() >= 2 {
            let recent_improvement = current.score - history.last().unwrap().score;

            if recent_improvement < self.config.min_improvement_threshold {
                recommendations.push("Last iteration showed minimal improvement - consider different approach".to_string());
            } else {
                recommendations.push(format!("Last iteration improved by {:.1}% - continue similar approach", recent_improvement * 100.0));
            }
        }

        // Check for patterns in failures
        let common_failures: HashMap<String, usize> = history.iter()
            .flat_map(|r| r.criteria.iter())
            .filter(|c| !c.passed)
            .fold(HashMap::new(), |mut acc, c| {
                *acc.entry(c.id.clone()).or_insert(0) += 1;
                acc
            });

        for (criterion_id, count) in common_failures.iter().filter(|(_, &count)| count > 1) {
            recommendations.push(format!("'{}' has failed {} times - needs attention", criterion_id, count));
        }

        if recommendations.is_empty() {
            recommendations.push("Continue with current approach".to_string());
        }

        recommendations
    }

    /// Update satisficing parameters based on learning
    pub fn update_parameters(&mut self, feedback: &SatisficingFeedback) {
        match feedback {
            SatisficingFeedback::TooConservative => {
                self.config.min_improvement_threshold *= 0.9; // Lower threshold
            }
            SatisficingFeedback::TooAggressive => {
                self.config.min_improvement_threshold *= 1.1; // Raise threshold
            }
            SatisficingFeedback::MaxIterationsTooLow => {
                self.config.max_iterations += 1;
            }
            SatisficingFeedback::MaxIterationsTooHigh => {
                self.config.max_iterations = self.config.max_iterations.saturating_sub(1);
            }
        }
    }
}

/// Feedback for updating satisficing parameters
#[derive(Debug, Clone)]
pub enum SatisficingFeedback {
    TooConservative,    // Stopped too early, could have continued
    TooAggressive,      // Continued too long, wasted iterations
    MaxIterationsTooLow, // Hit max iterations but could have benefited from more
    MaxIterationsTooHigh, // Rarely hit max iterations
}
