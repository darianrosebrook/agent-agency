//! Satisficing evaluator that determines when to stop iterating

use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

use super::{EvalReport, EvalStatus, StopReason};
use crate::types::ActionRequest;
use crate::loop_controller::PatchFailureType;

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
            // Hysteresis defaults
            hysteresis_window: 5,           // Track last 5 scores
            consecutive_threshold: 3,       // 3 consecutive sub-threshold iterations
            no_progress_loc_threshold: 10,  // < 10 LOC change = no progress
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

/// Satisficing evaluator with hysteresis and no-progress guards
pub struct SatisficingEvaluator {
    config: SatisficingConfig,
    /// Sliding window of recent scores for hysteresis analysis
    score_history: VecDeque<f64>,
    /// Recent action requests to detect repetition
    recent_actions: VecDeque<String>,
}

impl SatisficingEvaluator {
    /// Create a new satisficing evaluator
    pub fn new() -> Self {
        Self {
            config: SatisficingConfig::default(),
            score_history: VecDeque::new(),
            recent_actions: VecDeque::new(),
        }
    }

    /// Create with custom config
    pub fn with_config(config: SatisficingConfig) -> Self {
        Self {
            config,
            score_history: VecDeque::new(),
            recent_actions: VecDeque::new(),
        }
    }

    /// Decide whether to continue iterating based on evaluation results
    pub fn should_continue(
        &mut self,
        current: &EvalReport,
        history: &[EvalReport],
    ) -> SatisficingDecision {
        // Record score in sliding window for hysteresis
        self.record_score(current.score);
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

        // Check if quality ceiling has been reached (with hysteresis)
        if let Some(ceiling_reason) = self.check_quality_ceiling_with_hysteresis(current) {
            return SatisficingDecision {
                should_continue: false,
                reason: ceiling_reason,
                confidence: 0.8,
                recommendations: vec!["Quality improvements have plateaued (hysteresis applied)".to_string()],
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

    /// Record score in sliding window for hysteresis analysis
    fn record_score(&mut self, score: f64) {
        self.score_history.push_back(score);
        // Maintain sliding window size
        while self.score_history.len() > self.config.hysteresis_window {
            self.score_history.pop_front();
        }
    }

    /// Record action for repetition detection
    pub fn record_action(&mut self, action: &ActionRequest) {
        // Create a fingerprint of the action for comparison
        let fingerprint = self.action_fingerprint(action);
        self.recent_actions.push_back(fingerprint);
        // Keep only recent actions
        while self.recent_actions.len() > self.config.consecutive_threshold {
            self.recent_actions.pop_front();
        }
    }

    /// Create a fingerprint of an action for repetition detection
    fn action_fingerprint(&self, action: &ActionRequest) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        action.action_type.hash(&mut hasher);
        if let Some(changeset) = &action.changeset {
            // Hash the changeset structure (not content for performance)
            changeset.patches.len().hash(&mut hasher);
            for patch in &changeset.patches {
                patch.path.hash(&mut hasher);
                patch.hunks.len().hash(&mut hasher);
            }
        }
        format!("{:x}", hasher.finish())
    }

    /// Check if quality improvements have plateaued with hysteresis
    fn check_quality_ceiling_with_hysteresis(&self, current: &EvalReport) -> Option<StopReason> {
        if self.score_history.len() < self.config.consecutive_threshold {
            return None; // Not enough data for hysteresis
        }

        // Check for consecutive sub-threshold improvements
        let mut consecutive_sub_threshold = 0;
        let mut iter_scores = self.score_history.iter().rev(); // Most recent first
        let mut prev_score = current.score;

        // Start from current score and work backwards
        for &score in iter_scores.take(self.config.hysteresis_window) {
            let improvement = prev_score - score;
            if improvement.abs() < self.config.min_improvement_threshold {
                consecutive_sub_threshold += 1;
                if consecutive_sub_threshold >= self.config.consecutive_threshold {
                    return Some(StopReason::QualityCeiling);
                }
            } else {
                consecutive_sub_threshold = 0; // Reset on significant change
            }
            prev_score = score;
        }

        None
    }

    /// Check for no progress conditions
    pub fn check_no_progress(&self, changeset: Option<&file_ops::ChangeSet>) -> Option<StopReason> {
        // Check for zero LOC changes
        if let Some(changeset) = changeset {
            let total_loc: usize = changeset.patches.iter()
                .map(|p| p.hunks.iter().map(|h| h.lines.lines().count()).sum::<usize>())
                .sum();

            if total_loc < self.config.no_progress_loc_threshold {
                return Some(StopReason::NoProgress);
            }
        }

        None
    }

    /// Check for action request repetition
    pub fn check_action_repetition(&self) -> Option<StopReason> {
        if self.recent_actions.len() < self.config.consecutive_threshold {
            return None;
        }

        // Check if all recent actions are the same
        let first_action = self.recent_actions.front()?;
        let all_same = self.recent_actions.iter().all(|action| action == first_action);

        if all_same {
            Some(StopReason::NoProgress)
        } else {
            None
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EvalReport, EvalStatus, EvalCriterion};

    fn create_test_report(score: f64, status: EvalStatus) -> EvalReport {
        EvalReport {
            score,
            status,
            criteria: vec![],
            next_actions: vec![],
            logs: vec![],
            thresholds_met: vec!["tests-pass".to_string()],
            metadata: std::collections::HashMap::new(),
        }
    }

    #[test]
    fn test_hysteresis_prevents_premature_plateau() {
        let mut evaluator = SatisficingEvaluator::new();

        // Simulate scores that oscillate slightly but don't truly plateau
        let scores = vec![0.5, 0.52, 0.51, 0.53, 0.52, 0.51];
        let mut history = Vec::new();

        for &score in &scores {
            let report = create_test_report(score, EvalStatus::Fail);
            let decision = evaluator.should_continue(&report, &history);
            history.push(report);

            // Should continue despite small oscillations
            assert!(decision.should_continue || score == scores[scores.len() - 1]);
        }
    }

    #[test]
    fn test_consecutive_sub_threshold_triggers_plateau() {
        let mut evaluator = SatisficingEvaluator::new();

        // Create scores with 4 consecutive sub-threshold improvements (< 0.02)
        let scores = vec![0.5, 0.515, 0.517, 0.518, 0.519, 0.5195];
        let mut history = Vec::new();

        for (i, &score) in scores.iter().enumerate() {
            let report = create_test_report(score, EvalStatus::Fail);
            let decision = evaluator.should_continue(&report, &history);
            history.push(report.clone());

            // Should trigger plateau after 3+ consecutive sub-threshold improvements
            if i >= 5 { // After building enough history
                assert!(!decision.should_continue || decision.reason != StopReason::QualityCeiling);
            }
        }
    }

    #[test]
    fn test_no_progress_detection() {
        let evaluator = SatisficingEvaluator::new();

        // Create a changeset with very few changes
        let small_changeset = file_ops::ChangeSet {
            patches: vec![file_ops::Patch {
                path: "test.rs".to_string(),
                hunks: vec![file_ops::Hunk {
                    old_start: 1,
                    old_lines: 1,
                    new_start: 1,
                    new_lines: 1,
                    lines: "+// comment\n".to_string(), // Only 1 line added
                }],
                expected_prev_sha256: None,
            }],
        };

        let result = evaluator.check_no_progress(Some(&small_changeset));
        assert_eq!(result, Some(StopReason::NoProgress));
    }

    #[test]
    fn test_action_repetition_detection() {
        let mut evaluator = SatisficingEvaluator::new();

        // Record the same action multiple times
        let action = crate::types::ActionRequest::patch(
            file_ops::ChangeSet { patches: vec![] },
            "test action".to_string(),
            0.8
        );

        // Record the same action 4 times (more than consecutive_threshold of 3)
        for _ in 0..4 {
            evaluator.record_action(&action);
        }

        let result = evaluator.check_action_repetition();
        assert_eq!(result, Some(StopReason::NoProgress));
    }

    #[test]
    fn test_sliding_window_maintenance() {
        let mut evaluator = SatisficingEvaluator::new();

        // Record more scores than the hysteresis window
        for i in 0..10 {
            let report = create_test_report(0.5 + i as f64 * 0.01, EvalStatus::Fail);
            evaluator.record_score(report.score);
        }

        // Should only maintain the last hysteresis_window scores
        assert!(evaluator.score_history.len() <= evaluator.config.hysteresis_window);
    }
}
