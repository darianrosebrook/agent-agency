//! Evaluation Framework for Autonomous Task Execution
//!
//! Provides comprehensive evaluation of task execution quality with:
//! - Iteration limits to prevent infinite loops
//! - Quality ceiling detection (stopping when quality stops improving)
//! - Delta thresholds for diminishing returns detection
//! - Evaluation hooks for integration with autonomous executor

use schemars::JsonSchema;
use async_trait::async_trait;
use serde::{Serialize, Deserialize};
use tracing::{info, warn};
use chrono::{DateTime, Utc};

use agent_agency_contracts::final_verdict::FinalVerdictContract;

/// Evaluation configuration for iteration limits and quality thresholds
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvaluationConfig {
    /// Maximum number of refinement iterations allowed
    pub max_iterations: u32,
    
    /// Quality threshold for satisficing (0.0-1.0)
    /// When quality score >= this threshold, stop refining
    pub satisficing_threshold: f64,
    
    /// Minimum improvement delta required to continue refining (0.0-1.0)
    /// If improvement < this threshold, stop due to diminishing returns
    pub delta_threshold: f64,
    
    /// Quality ceiling threshold (0.0-1.0)
    /// If quality score >= this threshold, stop immediately
    pub quality_ceiling: f64,
    
    /// Window size for plateau detection (number of iterations to analyze)
    pub plateau_detection_window: usize,
    
    /// Standard deviation threshold for plateau detection
    /// If std dev of quality scores < this, consider it a plateau
    pub plateau_std_dev_threshold: f64,
}

impl Default for EvaluationConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            satisficing_threshold: 0.9,
            delta_threshold: 0.05,
            quality_ceiling: 0.95,
            plateau_detection_window: 3,
            plateau_std_dev_threshold: 0.01,
        }
    }
}

/// Evaluation result for a single iteration
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct IterationEvaluation {
    pub iteration: u32,
    #[schemars(with = "String")]
    pub timestamp: DateTime<Utc>,
    pub quality_score: f64,
    pub improvement_delta: f64,
    pub verdict: FinalVerdictContract,
    pub should_continue: bool,
    pub stop_reason: Option<StopReason>,
}

/// Reasons for stopping iteration refinement
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, JsonSchema)]
pub enum StopReason {
    /// Maximum iterations reached
    MaxIterationsReached,
    
    /// Quality threshold met (satisficing)
    SatisficingThresholdMet,
    
    /// Quality ceiling reached
    QualityCeilingReached,
    
    /// Diminishing returns detected (delta too small)
    DiminishingReturns,
    
    /// Quality plateau detected (no improvement over window)
    QualityPlateau,
    
    /// Council approval granted
    CouncilApproved,
    
    /// Council rejection (no further refinement possible)
    CouncilRejected,
}

/// Evaluation orchestrator for autonomous task execution
#[derive(Debug, Clone, Serialize, Deserialize, schemars::JsonSchema)]
pub struct EvaluationOrchestrator {
    config: EvaluationConfig,
}

impl EvaluationOrchestrator {
    /// Create a new evaluation orchestrator with default configuration
    pub fn new() -> Self {
        Self {
            config: EvaluationConfig::default(),
        }
    }
    
    /// Create a new evaluation orchestrator with custom configuration
    pub fn with_config(config: EvaluationConfig) -> Self {
        Self { config }
    }
    
    /// Get the current configuration
    pub fn config(&self) -> &EvaluationConfig {
        &self.config
    }
    
    /// Update the configuration
    pub fn set_config(&mut self, config: EvaluationConfig) {
        self.config = config;
    }
    
    /// Evaluate an iteration and determine if refinement should continue
    /// 
    /// Returns:
    /// - `IterationEvaluation` with evaluation results and stop recommendation
    /// - `StopReason` if iteration should stop, `None` if should continue
    pub async fn evaluate_iteration(
        &self,
        iteration: u32,
        quality_score: f64,
        quality_history: &[f64],
        verdict: FinalVerdictContract,
        council_approved: bool,
    ) -> IterationEvaluation {
        // Calculate improvement delta
        let improvement_delta = if iteration >= 2 && quality_history.len() >= 2 {
            let previous_score = quality_history[quality_history.len() - 2];
            quality_score - previous_score
        } else {
            0.0
        };
        
        let mut stop_reason = None;
        let mut should_continue = true;
        
        // Check iteration limit
        if iteration >= self.config.max_iterations {
            warn!(
                "Iteration limit reached: {} >= {}",
                iteration, self.config.max_iterations
            );
            stop_reason = Some(StopReason::MaxIterationsReached);
            should_continue = false;
        }
        // Check quality ceiling
        else if quality_score >= self.config.quality_ceiling {
            info!(
                "Quality ceiling reached: {:.3} >= {:.3}",
                quality_score, self.config.quality_ceiling
            );
            stop_reason = Some(StopReason::QualityCeilingReached);
            should_continue = false;
        }
        // Check satisficing threshold
        else if quality_score >= self.config.satisficing_threshold {
            info!(
                "Satisficing threshold met: {:.3} >= {:.3}",
                quality_score, self.config.satisficing_threshold
            );
            stop_reason = Some(StopReason::SatisficingThresholdMet);
            should_continue = false;
        }
        // Check diminishing returns (need at least 2 iterations)
        else if iteration >= 2 && improvement_delta < self.config.delta_threshold {
            warn!(
                "Diminishing returns detected: improvement delta {:.3} < threshold {:.3}",
                improvement_delta, self.config.delta_threshold
            );
            stop_reason = Some(StopReason::DiminishingReturns);
            should_continue = false;
        }
        // Check quality plateau (need at least plateau_detection_window iterations)
        else if quality_history.len() >= self.config.plateau_detection_window {
            if let Some(plateau_detected) = self.detect_plateau(quality_history) {
                if plateau_detected {
                    warn!(
                        "Quality plateau detected over last {} iterations",
                        self.config.plateau_detection_window
                    );
                    stop_reason = Some(StopReason::QualityPlateau);
                    should_continue = false;
                }
            }
        }
        
        // Council decisions override other logic
        if council_approved {
            info!("Council approved - stopping refinement");
            stop_reason = Some(StopReason::CouncilApproved);
            should_continue = false;
        }
        
        IterationEvaluation {
            iteration,
            timestamp: Utc::now(),
            quality_score,
            improvement_delta,
            verdict,
            should_continue,
            stop_reason,
        }
    }
    
    /// Detect if quality has plateaued (no significant improvement over window)
    fn detect_plateau(&self, quality_history: &[f64]) -> Option<bool> {
        if quality_history.len() < self.config.plateau_detection_window {
            return None;
        }
        
        // Get the last N scores
        let window_start = quality_history.len() - self.config.plateau_detection_window;
        let window_scores = &quality_history[window_start..];
        
        // Calculate mean and standard deviation
        let mean = window_scores.iter().sum::<f64>() / window_scores.len() as f64;
        let variance = window_scores
            .iter()
            .map(|&score| (score - mean).powi(2))
            .sum::<f64>() / window_scores.len() as f64;
        let std_dev = variance.sqrt();
        
        // If standard deviation is very small, consider it a plateau
        Some(std_dev < self.config.plateau_std_dev_threshold)
    }
    
    /// Calculate quality score from a verdict
    /// 
    /// Quality score is a weighted combination of:
    /// - Decision (40%): Accept = 1.0, Reject = 0.0
    /// - Vote confidence (30%): Average confidence of all votes
    /// - Coverage (20%): Verification summary coverage percentage
    /// - Claims verified (10%): Percentage of claims verified
    pub fn calculate_quality_score(&self, verdict: &FinalVerdictContract) -> f64 {
        // Decision weight: 40%
        let decision_score = match verdict.decision {
            agent_agency_contracts::final_verdict::FinalDecision::Accept => 1.0,
            agent_agency_contracts::final_verdict::FinalDecision::Reject => 0.0,
            agent_agency_contracts::final_verdict::FinalDecision::Modify => 0.5, // Partial acceptance
        };
        
        // Vote confidence weight: 30%
        let vote_confidence = if verdict.votes.is_empty() {
            0.5 // Default if no votes
        } else {
            // Calculate weighted average based on vote verdicts
            let total_weight: f64 = verdict.votes.iter()
                .map(|vote| vote.weight as f64)
                .sum();
            let weighted_sum: f64 = verdict.votes.iter()
                .map(|vote| {
                    let vote_score = match vote.verdict {
                        agent_agency_contracts::final_verdict::VoteVerdict::Pass => 1.0,
                        agent_agency_contracts::final_verdict::VoteVerdict::Fail => 0.0,
                        agent_agency_contracts::final_verdict::VoteVerdict::Uncertain => 0.5,
                    };
                    vote.weight as f64 * vote_score
                })
                .sum();
            if total_weight > 0.0 {
                weighted_sum / total_weight
            } else {
                0.5
            }
        };
        
        // Coverage weight: 20%
        let coverage_score = verdict.verification_summary.coverage_pct as f64 / 100.0;
        
        // Claims verified weight: 10%
        let claims_score = if verdict.verification_summary.claims_total > 0 {
            verdict.verification_summary.claims_verified as f64
                / verdict.verification_summary.claims_total as f64
        } else {
            0.5 // Default if no claims
        };
        
        // Weighted combination
        let quality_score = (decision_score * 0.4)
            + (vote_confidence * 0.3)
            + (coverage_score * 0.2)
            + (claims_score * 0.1);
        
        // Clamp to [0.0, 1.0]
        quality_score.max(0.0).min(1.0)
    }
    
    /// Check if iteration limit has been reached
    pub fn is_iteration_limit_reached(&self, iteration: u32) -> bool {
        iteration >= self.config.max_iterations
    }
    
    /// Check if quality ceiling has been reached
    pub fn is_quality_ceiling_reached(&self, quality_score: f64) -> bool {
        quality_score >= self.config.quality_ceiling
    }
    
    /// Check if satisficing threshold has been met
    pub fn is_satisficing_threshold_met(&self, quality_score: f64) -> bool {
        quality_score >= self.config.satisficing_threshold
    }
    
    /// Check if diminishing returns detected
    pub fn is_diminishing_returns(&self, improvement_delta: f64) -> bool {
        improvement_delta < self.config.delta_threshold
    }
}

impl Default for EvaluationOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

/// Evaluation hook trait for integration with autonomous executor
#[async_trait]
pub trait EvaluationHook: Send + Sync {
    /// Called before each iteration
    async fn before_iteration(&self, iteration: u32) -> Result<(), String>;
    
    /// Called after each iteration evaluation
    async fn after_iteration(
        &self,
        evaluation: &IterationEvaluation,
    ) -> Result<(), String>;
    
    /// Called when iteration stops
    async fn on_stop(&self, reason: &StopReason, final_quality: f64) -> Result<(), String>;
}

/// No-op evaluation hook for default behavior
#[derive(Debug, Default, Serialize, Deserialize, schemars::JsonSchema)]
pub struct NoOpEvaluationHook ;

#[async_trait]
impl EvaluationHook for NoOpEvaluationHook {
    async fn before_iteration(&self, _iteration: u32) -> Result<(), String> {
        Ok(())
    }
    
    async fn after_iteration(&self, _evaluation: &IterationEvaluation) -> Result<(), String> {
        Ok(())
    }
    
    async fn on_stop(&self, _reason: &StopReason, _final_quality: f64) -> Result<(), String> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;
    use agent_agency_contracts::final_verdict::{FinalDecision, FinalVerdictContract, VerificationSummary, VoteEntry, VoteVerdict};
    use uuid::Uuid;
    
    fn create_test_verdict(decision: FinalDecision, coverage: f64) -> Arc<FinalVerdictContract> {
        Arc::new(FinalVerdictContract {
            decision: decision.clone(),
            votes: vec![
                VoteEntry {
                    judge_id: Uuid::new_v4().to_string(),
                    weight: 1.0,
                    verdict: match decision {
                        FinalDecision::Accept => VoteVerdict::Pass,
                        FinalDecision::Reject => VoteVerdict::Fail,
                        FinalDecision::Modify => VoteVerdict::Uncertain,
                    },
                }
            ],
            dissent: String::new(),
            remediation: vec![],
            constitutional_refs: vec![],
            verification_summary: VerificationSummary {
                claims_total: 10,
                claims_verified: (coverage * 10.0) as u32,
                coverage_pct: (coverage * 100.0) as f32,
            },
        })
    }
    
    #[tokio::test]
    async fn test_iteration_limit() {
        let evaluator = EvaluationOrchestrator::new();
        let config = evaluator.config();

        let verdict = create_test_verdict(FinalDecision::Accept, 0.8);
        let quality_score = evaluator.calculate_quality_score(&verdict);
        let quality_history = vec![0.7, 0.75, 0.8];

        // Test iteration limit
        let eval = evaluator.evaluate_iteration(
            config.max_iterations,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            false,
        ).await;
        
        assert!(!eval.should_continue);
        assert_eq!(eval.stop_reason, Some(StopReason::MaxIterationsReached));
    }
    
    #[tokio::test]
    async fn test_quality_ceiling() {
        let evaluator = EvaluationOrchestrator::new();
        
        let verdict = create_test_verdict(FinalDecision::Accept, 0.98);
        let quality_score = evaluator.calculate_quality_score(&verdict);
        let quality_history = vec![0.9, 0.92, quality_score];
        
        // Test quality ceiling
        let eval = evaluator.evaluate_iteration(
            1,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            false,
        ).await;
        
        assert!(!eval.should_continue);
        assert_eq!(eval.stop_reason, Some(StopReason::QualityCeilingReached));
    }
    
    #[tokio::test]
    async fn test_satisficing_threshold() {
        let evaluator = EvaluationOrchestrator::new();
        
        let verdict = create_test_verdict(FinalDecision::Accept, 0.9);
        let quality_score = evaluator.calculate_quality_score(&verdict);
        let quality_history = vec![0.85, 0.88, quality_score];
        
        // Test satisficing threshold
        let eval = evaluator.evaluate_iteration(
            2,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            false,
        ).await;
        
        assert!(!eval.should_continue);
        assert_eq!(eval.stop_reason, Some(StopReason::SatisficingThresholdMet));
    }
    
    #[tokio::test]
    async fn test_diminishing_returns() {
        let evaluator = EvaluationOrchestrator::new();
        
        let verdict = create_test_verdict(FinalDecision::Accept, 0.7);
        let quality_score = 0.71; // Very small improvement
        let quality_history = vec![0.70, quality_score];
        
        // Test diminishing returns
        let eval = evaluator.evaluate_iteration(
            2,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            false,
        ).await;
        
        assert!(!eval.should_continue);
        assert_eq!(eval.stop_reason, Some(StopReason::DiminishingReturns));
    }
    
    #[tokio::test]
    async fn test_plateau_detection() {
        let mut config = EvaluationConfig::default();
        config.plateau_detection_window = 3;
        config.plateau_std_dev_threshold = 0.01;
        
        let evaluator = EvaluationOrchestrator::with_config(config);
        
        let verdict = create_test_verdict(FinalDecision::Accept, 0.7);
        let quality_score = 0.75;
        // Create a plateau: scores are very similar
        let quality_history = vec![0.750, 0.751, 0.749, quality_score];
        
        // Test plateau detection
        let eval = evaluator.evaluate_iteration(
            4,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            false,
        ).await;
        
        assert!(!eval.should_continue);
        assert_eq!(eval.stop_reason, Some(StopReason::QualityPlateau));
    }
    
    #[tokio::test]
    async fn test_council_approval() {
        let evaluator = EvaluationOrchestrator::new();
        
        let verdict = create_test_verdict(FinalDecision::Accept, 0.6);
        let quality_score = 0.6;
        let quality_history = vec![0.5, quality_score];
        
        // Test council approval
        let eval = evaluator.evaluate_iteration(
            1,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            true, // Council approved
        ).await;
        
        assert!(!eval.should_continue);
        assert_eq!(eval.stop_reason, Some(StopReason::CouncilApproved));
    }
    
    #[tokio::test]
    async fn test_continue_refinement() {
        let evaluator = EvaluationOrchestrator::new();
        
        let verdict = create_test_verdict(FinalDecision::Accept, 0.6);
        let quality_score = 0.6; // Below thresholds
        let quality_history = vec![0.5, quality_score];
        
        // Test should continue
        let eval = evaluator.evaluate_iteration(
            1,
            quality_score,
            &quality_history,
            (*verdict).clone(),
            false,
        ).await;
        
        assert!(eval.should_continue);
        assert_eq!(eval.stop_reason, None);
    }
    
    #[tokio::test]
    async fn test_quality_score_calculation() {
        let evaluator = EvaluationOrchestrator::new();
        
        // High quality verdict
        let high_verdict = create_test_verdict(FinalDecision::Accept, 1.0);
        let high_score = evaluator.calculate_quality_score(&high_verdict);
        assert!(high_score > 0.8);
        
        // Low quality verdict
        let low_verdict = create_test_verdict(FinalDecision::Reject, 0.0);
        let low_score = evaluator.calculate_quality_score(&low_verdict);
        assert!(low_score < 0.5);
    }
}

