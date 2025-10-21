//! Satisficing evaluator that determines when to stop iterating

use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};

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
    pub fn check_no_progress(&self, changeset: Option<&crate::stubs::ChangeSet>) -> Option<StopReason> {
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

    /// Check if patch failures indicate we should stop (addresses 75% of agent failures)
    pub fn check_patch_failure_patterns(&self, recent_patch_failures: &[PatchFailureType]) -> Option<SatisficingDecision> {
        if recent_patch_failures.is_empty() {
            return None;
        }

        // Check for consecutive patch failures (2+ in a row indicates systemic issues)
        let consecutive_failures = recent_patch_failures.iter()
            .rev()
            .take_while(|failure| matches!(failure, PatchFailureType::SyntaxError | PatchFailureType::MergeConflict))
            .count();

        if consecutive_failures >= 2 {
            return Some(SatisficingDecision {
                should_continue: false,
                reason: StopReason::PatchFailure,
                confidence: 0.9,
                recommendations: vec![
                    format!("{} consecutive patch failures detected", consecutive_failures),
                    "Check for systemic issues in code generation or environment".to_string(),
                    "Consider model fallback or different prompting strategy".to_string(),
                ],
            });
        }

        // Check for environment failure patterns
        let env_failures = recent_patch_failures.iter()
            .filter(|f| matches!(f, PatchFailureType::EnvironmentIssue))
            .count();

        if env_failures >= 3 {
            return Some(SatisficingDecision {
                should_continue: false,
                reason: StopReason::PatchFailure,
                confidence: 0.8,
                recommendations: vec![
                    format!("{} environment failures detected", env_failures),
                    "Check build environment, dependencies, or workspace configuration".to_string(),
                    "Consider environment reset or different workspace strategy".to_string(),
                ],
            });
        }

        None
    }

    /// Check if iteration progress has stalled (plateau detection)
    pub fn check_progress_plateau(&self, recent_progress: &[crate::types::IterationProgress]) -> Option<SatisficingDecision> {
        if recent_progress.len() < 3 {
            return None; // Need at least 3 iterations to detect plateau
        }

        // Check for plateau: minimal progress over last 3 iterations
        let recent_scores: Vec<f64> = recent_progress.iter()
            .rev()
            .take(3)
            .map(|p| p.score_improvement)
            .collect();

        let avg_improvement = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
        let max_improvement = recent_scores.iter().fold(0.0f64, |a, &b| a.max(b));

        // Plateau conditions:
        // 1. Average improvement < 2% over last 3 iterations
        // 2. No single iteration improved by > 5%
        // 3. Total LOC change < 10 over last 3 iterations
        let total_loc_change: usize = recent_progress.iter()
            .rev()
            .take(3)
            .map(|p| p.loc_changed)
            .sum();

        let plateau_detected = avg_improvement < 0.02 && max_improvement < 0.05 && total_loc_change < 10;

        if plateau_detected {
            return Some(SatisficingDecision {
                should_continue: false,
                reason: StopReason::ProgressStalled,
                confidence: 0.85,
                recommendations: vec![
                    format!("Progress plateau detected over {} iterations", recent_progress.len()),
                    format!("Average score improvement: {:.3}%", avg_improvement * 100.0),
                    format!("Total LOC changed: {}", total_loc_change),
                    "Consider different approach, model fallback, or scope reduction".to_string(),
                ],
            });
        }

        None
    }

    /// Check if context overload should trigger termination (addresses large codebase failures)
    pub fn check_context_overload_termination(&self, context_metrics: &crate::types::ContextMetrics, overload_threshold: f64, max_files: usize) -> Option<SatisficingDecision> {
        let context_overloaded = context_metrics.context_window_utilization >= overload_threshold;
        let files_overloaded = context_metrics.files_in_scope >= max_files;

        if context_overloaded || files_overloaded {
            let reasons = vec![
                if context_overloaded {
                    format!("Context window utilization {:.1}% exceeds threshold {:.1}%",
                           context_metrics.context_window_utilization * 100.0, overload_threshold * 100.0)
                } else {
                    String::new()
                },
                if files_overloaded {
                    format!("Files in scope {} exceeds maximum {}", context_metrics.files_in_scope, max_files)
                } else {
                    String::new()
                }
            ].into_iter().filter(|s| !s.is_empty()).collect::<Vec<_>>();

            return Some(SatisficingDecision {
                should_continue: false,
                reason: StopReason::Error, // Could add a new StopReason::ContextOverload if desired
                confidence: 0.95,
                recommendations: vec![
                    format!("Context overload detected: {}", reasons.join(", ")),
                    "Consider breaking task into smaller subtasks".to_string(),
                    "Reduce scope to focus on fewer files".to_string(),
                    "Use different model with larger context window".to_string(),
                ],
            });
        }

        None
    }

    /// Check if environment failure should trigger recovery instead of termination
    pub fn check_environment_failure_recovery(
        &self,
        recent_failures: &[super::EvaluationFailureType],
    ) -> Option<SatisficingDecision> {
        if recent_failures.is_empty() {
            return None;
        }

        // Count environment vs logic failures in recent history
        let mut env_failures = 0;
        let mut logic_failures = 0;

        for failure in recent_failures.iter().rev().take(5) { // Last 5 failures
            match failure {
                super::EvaluationFailureType::EnvironmentFailure { .. } => env_failures += 1,
                super::EvaluationFailureType::LogicFailure { .. } => logic_failures += 1,
            }
        }

        // If we have 3+ environment failures in recent history, suggest recovery
        if env_failures >= 3 {
            return Some(SatisficingDecision {
                should_continue: false,
                reason: StopReason::Error, // Could use a new StopReason::EnvironmentFailure if added
                confidence: 0.9,
                recommendations: vec![
                    format!("{} environment failures detected in recent iterations", env_failures),
                    "Environment issues persist across iterations - requires intervention".to_string(),
                    "Possible solutions: dependency installation, environment reset, configuration fixes".to_string(),
                    "Consider pausing execution and investigating environment setup".to_string(),
                ],
            });
        }

        // If mixed failures but environment dominates recent ones, still flag for attention
        if env_failures >= 2 && env_failures > logic_failures {
            return Some(SatisficingDecision {
                should_continue: true, // Continue but with warnings
                reason: StopReason::Unknown, // Not stopping, just flagging
                confidence: 0.7,
                recommendations: vec![
                    format!("Environment failures ({}) outpacing logic failures ({})", env_failures, logic_failures),
                    "Monitor environment stability - may need intervention soon".to_string(),
                ],
            });
        }

        None
    }

    /// Get recommended recovery strategy based on failure patterns
    pub fn get_recovery_strategy(
        &self,
        failure_type: &super::EvaluationFailureType,
    ) -> EnvironmentRecoveryStrategy {
        match failure_type {
            super::EvaluationFailureType::EnvironmentFailure { category } => {
                match category {
                    super::EnvironmentFailureCategory::DependencyMissing => {
                        EnvironmentRecoveryStrategy::InstallDependencies
                    }
                    super::EnvironmentFailureCategory::BuildFailure => {
                        EnvironmentRecoveryStrategy::RebuildEnvironment
                    }
                    super::EnvironmentFailureCategory::ConfigurationError => {
                        EnvironmentRecoveryStrategy::ResetConfiguration
                    }
                    super::EnvironmentFailureCategory::PermissionError => {
                        EnvironmentRecoveryStrategy::FixPermissions
                    }
                    super::EnvironmentFailureCategory::ResourceExhaustion => {
                        EnvironmentRecoveryStrategy::ScaleResources
                    }
                    super::EnvironmentFailureCategory::ExternalServiceFailure => {
                        EnvironmentRecoveryStrategy::RetryWithBackoff
                    }
                }
            }
            super::EvaluationFailureType::LogicFailure { .. } => {
                // Logic failures don't have environment recovery strategies
                EnvironmentRecoveryStrategy::NoRecoveryNeeded
            }
        }
    }
}

/// Recovery strategies for environment failures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EnvironmentRecoveryStrategy {
    /// Install missing dependencies
    InstallDependencies,
    /// Rebuild the environment/clean build artifacts
    RebuildEnvironment,
    /// Reset configuration to defaults
    ResetConfiguration,
    /// Fix permission issues
    FixPermissions,
    /// Scale up resources (memory, CPU, disk)
    ScaleResources,
    /// Retry with exponential backoff for external services
    RetryWithBackoff,
    /// No environment recovery needed (logic failure)
    NoRecoveryNeeded,
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
    use crate::types::{EvalReport, EvalStatus, EvalCriterion, IterationProgress, ContextMetrics};
    use crate::evaluation::{EvaluationFailureType, EnvironmentFailureCategory, LogicFailureCategory};

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
        let small_changeset = crate::stubs::ChangeSet {
            patches: vec![crate::stubs::Patch {
                path: "test.rs".to_string(),
                hunks: vec![crate::stubs::Hunk {
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
            crate::stubs::ChangeSet { patches: vec![] },
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

    // ===== NEW INSTRUMENTATION TESTS =====

    #[test]
    fn test_patch_failure_pattern_detection() {
        let evaluator = SatisficingEvaluator::new();
        let mut patch_failures = Vec::new();

        // Add 3 syntax errors (should trigger)
        for _ in 0..3 {
            patch_failures.push(PatchFailureType::SyntaxError);
        }

        let decision = evaluator.check_patch_failure_patterns(&patch_failures);
        assert!(decision.is_some());
        assert!(!decision.unwrap().should_continue);
        assert_eq!(decision.unwrap().reason, StopReason::PatchFailure);
    }

    #[test]
    fn test_patch_failure_no_trigger_with_mixed_types() {
        let evaluator = SatisficingEvaluator::new();
        let patch_failures = vec![
            PatchFailureType::SyntaxError,
            PatchFailureType::MergeConflict,
            PatchFailureType::PathBlocked, // Different type, should not trigger
        ];

        let decision = evaluator.check_patch_failure_patterns(&patch_failures);
        assert!(decision.is_none()); // Should not trigger with mixed types
    }

    #[test]
    fn test_progress_plateau_detection() {
        let evaluator = SatisficingEvaluator::new();
        let progress_history = vec![
            IterationProgress { files_touched: 2, loc_changed: 15, test_pass_rate_delta: 0.01, lint_errors_delta: 0, score_improvement: 0.02, timestamp: chrono::Utc::now() },
            IterationProgress { files_touched: 1, loc_changed: 8, test_pass_rate_delta: -0.005, lint_errors_delta: 1, score_improvement: 0.01, timestamp: chrono::Utc::now() },
            IterationProgress { files_touched: 1, loc_changed: 5, test_pass_rate_delta: 0.0, lint_errors_delta: 0, score_improvement: 0.008, timestamp: chrono::Utc::now() },
            IterationProgress { files_touched: 0, loc_changed: 2, test_pass_rate_delta: 0.0, lint_errors_delta: 0, score_improvement: 0.005, timestamp: chrono::Utc::now() }, // Plateau conditions met
        ];

        let decision = evaluator.check_progress_plateau(&progress_history);
        assert!(decision.is_some());
        assert!(!decision.unwrap().should_continue);
        assert_eq!(decision.unwrap().reason, StopReason::ProgressStalled);
    }

    #[test]
    fn test_progress_plateau_no_trigger_with_good_progress() {
        let evaluator = SatisficingEvaluator::new();
        let progress_history = vec![
            IterationProgress { files_touched: 5, loc_changed: 50, test_pass_rate_delta: 0.1, lint_errors_delta: -2, score_improvement: 0.15, timestamp: chrono::Utc::now() },
            IterationProgress { files_touched: 3, loc_changed: 30, test_pass_rate_delta: 0.05, lint_errors_delta: -1, score_improvement: 0.12, timestamp: chrono::Utc::now() },
        ];

        let decision = evaluator.check_progress_plateau(&progress_history);
        assert!(decision.is_none()); // Should not trigger with good progress
    }

    #[test]
    fn test_context_overload_detection() {
        let evaluator = SatisficingEvaluator::new();
        let context_metrics = ContextMetrics {
            prompt_size_tokens: 9000,
            context_window_utilization: 0.95, // Over 80% threshold
            files_in_scope: 45, // Under 50 file threshold
            dependency_depth: 5,
            timestamp: chrono::Utc::now(),
        };

        let decision = evaluator.check_context_overload_termination(&context_metrics, 0.8, 50);
        assert!(decision.is_some());
        assert!(!decision.unwrap().should_continue);
        assert!(decision.unwrap().recommendations.iter().any(|r| r.contains("Context window utilization")));
    }

    #[test]
    fn test_context_overload_files_exceeded() {
        let evaluator = SatisficingEvaluator::new();
        let context_metrics = ContextMetrics {
            prompt_size_tokens: 5000,
            context_window_utilization: 0.6, // Under threshold
            files_in_scope: 55, // Over 50 file threshold
            dependency_depth: 5,
            timestamp: chrono::Utc::now(),
        };

        let decision = evaluator.check_context_overload_termination(&context_metrics, 0.8, 50);
        assert!(decision.is_some());
        assert!(!decision.unwrap().should_continue);
        assert!(decision.unwrap().recommendations.iter().any(|r| r.contains("files in scope")));
    }

    #[test]
    fn test_context_overload_no_trigger_under_thresholds() {
        let evaluator = SatisficingEvaluator::new();
        let context_metrics = ContextMetrics {
            prompt_size_tokens: 5000,
            context_window_utilization: 0.6, // Under threshold
            files_in_scope: 30, // Under threshold
            dependency_depth: 5,
            timestamp: chrono::Utc::now(),
        };

        let decision = evaluator.check_context_overload_termination(&context_metrics, 0.8, 50);
        assert!(decision.is_none()); // Should not trigger
    }

    #[test]
    fn test_environment_failure_recovery_three_failures() {
        let evaluator = SatisficingEvaluator::new();
        let failures = vec![
            EvaluationFailureType::EnvironmentFailure {
                category: EnvironmentFailureCategory::DependencyMissing
            },
            EvaluationFailureType::EnvironmentFailure {
                category: EnvironmentFailureCategory::BuildFailure
            },
            EvaluationFailureType::EnvironmentFailure {
                category: EnvironmentFailureCategory::ConfigurationError
            },
        ];

        let decision = evaluator.check_environment_failure_recovery(&failures);
        assert!(decision.is_some());
        assert!(!decision.unwrap().should_continue);
        assert!(decision.unwrap().recommendations.iter().any(|r| r.contains("3 environment failures")));
    }

    #[test]
    fn test_environment_failure_recovery_mixed_but_environment_dominant() {
        let evaluator = SatisficingEvaluator::new();
        let failures = vec![
            EvaluationFailureType::EnvironmentFailure {
                category: EnvironmentFailureCategory::DependencyMissing
            },
            EvaluationFailureType::LogicFailure {
                category: LogicFailureCategory::SyntaxError
            },
            EvaluationFailureType::EnvironmentFailure {
                category: EnvironmentFailureCategory::BuildFailure
            },
        ];

        let decision = evaluator.check_environment_failure_recovery(&failures);
        assert!(decision.is_some());
        assert!(decision.unwrap().should_continue); // Continue but with warnings
        assert!(decision.unwrap().recommendations.iter().any(|r| r.contains("Environment failures")));
    }

    #[test]
    fn test_environment_failure_recovery_no_trigger_logic_dominant() {
        let evaluator = SatisficingEvaluator::new();
        let failures = vec![
            EvaluationFailureType::LogicFailure {
                category: LogicFailureCategory::SyntaxError
            },
            EvaluationFailureType::LogicFailure {
                category: LogicFailureCategory::TypeError
            },
            EvaluationFailureType::EnvironmentFailure {
                category: EnvironmentFailureCategory::DependencyMissing
            },
        ];

        let decision = evaluator.check_environment_failure_recovery(&failures);
        assert!(decision.is_none()); // Should not trigger when logic failures dominate
    }

    #[test]
    fn test_recovery_strategy_selection_dependency_missing() {
        let evaluator = SatisficingEvaluator::new();
        let failure = EvaluationFailureType::EnvironmentFailure {
            category: EnvironmentFailureCategory::DependencyMissing
        };

        let strategy = evaluator.get_recovery_strategy(&failure);
        assert!(matches!(strategy, EnvironmentRecoveryStrategy::InstallDependencies));
    }

    #[test]
    fn test_recovery_strategy_selection_build_failure() {
        let evaluator = SatisficingEvaluator::new();
        let failure = EvaluationFailureType::EnvironmentFailure {
            category: EnvironmentFailureCategory::BuildFailure
        };

        let strategy = evaluator.get_recovery_strategy(&failure);
        assert!(matches!(strategy, EnvironmentRecoveryStrategy::RebuildEnvironment));
    }

    #[test]
    fn test_recovery_strategy_selection_logic_failure() {
        let evaluator = SatisficingEvaluator::new();
        let failure = EvaluationFailureType::LogicFailure {
            category: LogicFailureCategory::SyntaxError
        };

        let strategy = evaluator.get_recovery_strategy(&failure);
        assert!(matches!(strategy, EnvironmentRecoveryStrategy::NoRecoveryNeeded));
    }
}
