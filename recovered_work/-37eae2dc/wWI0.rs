//! Satisficing Evaluator
//!
//! Prevents infinite refinement loops by determining when "good enough"
//! quality has been achieved based on diminishing returns and cost-benefit analysis.

use std::collections::HashMap;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::gates::{QualityGateResult, GateStatus};
use super::orchestrator::QualityReport;

/// Satisficing evaluator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatisficingConfig {
    /// Maximum refinement iterations
    pub max_iterations: usize,
    /// Minimum improvement threshold (percentage points)
    pub min_improvement_threshold: f64,
    /// Cost-benefit ratio threshold
    pub cost_benefit_threshold: f64,
    /// Diminishing returns threshold
    pub diminishing_returns_threshold: f64,
    /// Time budget per refinement (seconds)
    pub time_budget_per_refinement_seconds: u64,
    /// Enable learning from historical data
    pub enable_learning: bool,
}

/// Satisficing evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SatisficingResult {
    pub should_continue: bool,
    pub reason: String,
    pub quality_improvement: f64,
    pub cost_benefit_ratio: f64,
    pub iterations_remaining: usize,
    pub estimated_completion_time: Option<DateTime<Utc>>,
    pub recommendations: Vec<String>,
}

/// Quality metrics for satisficing analysis
#[derive(Debug, Clone)]
struct QualityMetrics {
    overall_score: f64,
    gate_scores: HashMap<String, f64>,
    failed_gates: Vec<String>,
    warning_gates: Vec<String>,
    execution_time_ms: u64,
}

/// Satisficing evaluator
pub struct SatisficingEvaluator {
    config: SatisficingConfig,
    historical_data: Vec<QualityReport>,
}

impl SatisficingEvaluator {
    pub fn new(config: SatisficingConfig) -> Self {
        Self {
            config,
            historical_data: Vec::new(),
        }
    }

    /// Evaluate if refinement should continue
    pub async fn evaluate_satisficing(
        &mut self,
        current_report: &QualityReport,
        previous_reports: &[QualityReport],
        iterations_completed: usize,
    ) -> Result<SatisficingResult, SatisficingError> {
        tracing::debug!("Evaluating satisficing for iteration {} (task: {})",
            iterations_completed, current_report.task_id);

        // Check maximum iterations
        if iterations_completed >= self.config.max_iterations {
            return Ok(SatisficingResult {
                should_continue: false,
                reason: format!("Maximum iterations reached ({})", self.config.max_iterations),
                quality_improvement: 0.0,
                cost_benefit_ratio: 0.0,
                iterations_remaining: 0,
                estimated_completion_time: None,
                recommendations: vec![
                    "Consider manual code review".to_string(),
                    "Evaluate if requirements can be relaxed".to_string(),
                ],
            });
        }

        // Calculate quality improvement
        let quality_improvement = self.calculate_quality_improvement(current_report, previous_reports);

        // Check minimum improvement threshold
        if quality_improvement < self.config.min_improvement_threshold {
            return Ok(SatisficingResult {
                should_continue: false,
                reason: format!("Quality improvement below threshold ({:.2}% < {:.2}%)",
                    quality_improvement, self.config.min_improvement_threshold),
                quality_improvement,
                cost_benefit_ratio: 0.0,
                iterations_remaining: self.config.max_iterations - iterations_completed - 1,
                estimated_completion_time: None,
                recommendations: vec![
                    "Quality gains are diminishing".to_string(),
                    "Consider accepting current quality level".to_string(),
                ],
            });
        }

        // Check for diminishing returns
        if self.detect_diminishing_returns(previous_reports) {
            return Ok(SatisficingResult {
                should_continue: false,
                reason: "Diminishing returns detected".to_string(),
                quality_improvement,
                cost_benefit_ratio: 0.0,
                iterations_remaining: self.config.max_iterations - iterations_completed - 1,
                estimated_completion_time: None,
                recommendations: vec![
                    "Further iterations yield minimal quality gains".to_string(),
                    "Cost-benefit ratio unfavorable".to_string(),
                ],
            });
        }

        // Calculate cost-benefit ratio
        let cost_benefit_ratio = self.calculate_cost_benefit_ratio(current_report, previous_reports);

        if cost_benefit_ratio < self.config.cost_benefit_threshold {
            return Ok(SatisficingResult {
                should_continue: false,
                reason: format!("Cost-benefit ratio too low ({:.2} < {:.2})",
                    cost_benefit_ratio, self.config.cost_benefit_threshold),
                quality_improvement,
                cost_benefit_ratio,
                iterations_remaining: self.config.max_iterations - iterations_completed - 1,
                estimated_completion_time: None,
                recommendations: vec![
                    "Refinement cost exceeds quality benefits".to_string(),
                    "Consider alternative improvement strategies".to_string(),
                ],
            });
        }

        // Check if we've reached target quality
        if self.has_reached_target_quality(current_report) {
            return Ok(SatisficingResult {
                should_continue: false,
                reason: "Target quality level achieved".to_string(),
                quality_improvement,
                cost_benefit_ratio,
                iterations_remaining: self.config.max_iterations - iterations_completed - 1,
                estimated_completion_time: None,
                recommendations: vec![
                    "Quality standards met".to_string(),
                    "Ready for next phase".to_string(),
                ],
            });
        }

        // Update historical data for learning
        if self.config.enable_learning {
            self.historical_data.push(current_report.clone());
        }

        // Estimate completion time
        let estimated_completion = self.estimate_completion_time(iterations_completed, quality_improvement);

        // Continue refinement
        let recommendations = self.generate_refinement_recommendations(current_report);

        Ok(SatisficingResult {
            should_continue: true,
            reason: format!("Quality improving ({:.2}%), cost-benefit ratio {:.2}",
                quality_improvement, cost_benefit_ratio),
            quality_improvement,
            cost_benefit_ratio,
            iterations_remaining: self.config.max_iterations - iterations_completed - 1,
            estimated_completion_time: estimated_completion,
            recommendations,
        })
    }

    /// Calculate quality improvement from previous iterations
    fn calculate_quality_improvement(
        &self,
        current: &QualityReport,
        previous: &[QualityReport],
    ) -> f64 {
        if previous.is_empty() {
            return 0.0; // First iteration, no improvement to measure
        }

        // Get the most recent previous report
        let previous_report = previous.last().unwrap();

        // Calculate improvement in overall score
        let improvement = current.overall_score - previous_report.overall_score;

        // Weight by the number of improved gates
        let improved_gates = self.count_improved_gates(current, previous_report);
        let gate_weight = improved_gates as f64 / current.gate_results.len() as f64;

        improvement * gate_weight
    }

    /// Count gates that improved from previous report
    fn count_improved_gates(&self, current: &QualityReport, previous: &QualityReport) -> usize {
        let mut improved = 0;

        // Create maps of gate results by name
        let current_gates: HashMap<_, _> = current.gate_results.iter()
            .map(|r| (r.name.clone(), r))
            .collect();

        let previous_gates: HashMap<_, _> = previous.gate_results.iter()
            .map(|r| (r.name.clone(), r))
            .collect();

        for (gate_name, current_result) in &current_gates {
            if let Some(previous_result) = previous_gates.get(gate_name) {
                // Check if status improved or score increased
                let status_improved = matches!(previous_result.status, GateStatus::Failed | GateStatus::Error)
                    && matches!(current_result.status, GateStatus::Passed | GateStatus::Warning);

                let score_improved = current_result.score > previous_result.score;

                if status_improved || score_improved {
                    improved += 1;
                }
            }
        }

        improved
    }

    /// Detect diminishing returns in quality improvement
    fn detect_diminishing_returns(&self, reports: &[QualityReport]) -> bool {
        if reports.len() < 3 {
            return false; // Need at least 3 reports to detect trend
        }

        // Calculate improvement rates for last few iterations
        let mut improvements = Vec::new();
        for i in 1..reports.len() {
            let improvement = reports[i].overall_score - reports[i-1].overall_score;
            improvements.push(improvement);
        }

        // Check if improvements are consistently decreasing
        let mut decreasing_trend = true;
        for i in 1..improvements.len() {
            if improvements[i] >= improvements[i-1] {
                decreasing_trend = false;
                break;
            }
        }

        // Check if latest improvement is below threshold
        let latest_improvement = improvements.last().copied().unwrap_or(0.0);
        let diminishing = latest_improvement < self.config.diminishing_returns_threshold;

        decreasing_trend && diminishing
    }

    /// Calculate cost-benefit ratio of refinement
    fn calculate_cost_benefit_ratio(
        &self,
        current: &QualityReport,
        previous: &[QualityReport],
    ) -> f64 {
        if previous.is_empty() {
            return 1.0; // First iteration has infinite benefit
        }

        let improvement = self.calculate_quality_improvement(current, previous);
        let time_cost = current.total_duration_ms as f64 / 1000.0; // Convert to seconds

        if time_cost == 0.0 {
            return 1.0; // Avoid division by zero
        }

        // Cost-benefit ratio: quality improvement per unit time
        improvement / time_cost
    }

    /// Check if target quality has been reached
    fn has_reached_target_quality(&self, report: &QualityReport) -> bool {
        // Target is all gates passed or warning, no failures
        report.gates_failed == 0 && report.gates_warning <= (report.gates_executed / 4).max(1)
    }

    /// Estimate completion time based on historical trends
    fn estimate_completion_time(
        &self,
        iterations_completed: usize,
        current_improvement: f64,
    ) -> Option<DateTime<Utc>> {
        if !self.config.enable_learning || self.historical_data.len() < 2 {
            return None;
        }

        // Simple linear extrapolation based on improvement rate
        let avg_time_per_iteration = self.historical_data.iter()
            .map(|r| r.total_duration_ms as f64 / 1000.0)
            .sum::<f64>() / self.historical_data.len() as f64;

        let remaining_improvement_needed = 1.0 - self.historical_data.last()?.overall_score;
        let iterations_needed = if current_improvement > 0.0 {
            (remaining_improvement_needed / current_improvement).ceil() as usize
        } else {
            self.config.max_iterations - iterations_completed
        };

        let estimated_seconds = iterations_needed as f64 * avg_time_per_iteration;
        Some(Utc::now() + chrono::Duration::seconds(estimated_seconds as i64))
    }

    /// Generate recommendations for next refinement iteration
    fn generate_refinement_recommendations(&self, report: &QualityReport) -> Vec<String> {
        let mut recommendations = Vec::new();

        // Find failing gates
        let failing_gates: Vec<_> = report.gate_results.iter()
            .filter(|r| matches!(r.status, GateStatus::Failed | GateStatus::Error))
            .collect();

        // Prioritize recommendations based on failing gates
        for gate in &failing_gates {
            match gate.name.as_str() {
                "caws_compliance" => {
                    recommendations.push("Address CAWS compliance violations".to_string());
                    recommendations.push("Review scope boundaries and change budget".to_string());
                }
                "linting" => {
                    recommendations.push("Fix linting errors and warnings".to_string());
                    recommendations.push("Consider updating linting configuration".to_string());
                }
                "testing" => {
                    recommendations.push("Fix failing tests".to_string());
                    recommendations.push("Add missing test coverage".to_string());
                }
                "coverage" => {
                    recommendations.push("Increase test coverage".to_string());
                    recommendations.push("Add tests for uncovered code paths".to_string());
                }
                "type_check" => {
                    recommendations.push("Fix TypeScript compilation errors".to_string());
                    recommendations.push("Update type definitions".to_string());
                }
                "mutation" => {
                    recommendations.push("Improve test quality to kill more mutants".to_string());
                    recommendations.push("Add assertions to existing tests".to_string());
                }
                _ => {
                    recommendations.push(format!("Address issues in {} gate", gate.name));
                }
            }
        }

        // General recommendations
        if failing_gates.is_empty() {
            recommendations.push("Focus on warning-level issues".to_string());
            recommendations.push("Consider performance optimizations".to_string());
        }

        // Risk tier specific advice
        match report.risk_tier {
            crate::planning::types::RiskTier::Critical => {
                recommendations.push("Critical tier: Ensure zero failures before proceeding".to_string());
            }
            crate::planning::types::RiskTier::High => {
                recommendations.push("High tier: Minimize warnings and ensure test coverage".to_string());
            }
            crate::planning::types::RiskTier::Standard => {
                recommendations.push("Standard tier: Balance quality with development velocity".to_string());
            }
        }

        recommendations
    }

    /// Get historical satisficing data for analysis
    pub fn get_historical_data(&self) -> &[QualityReport] {
        &self.historical_data
    }

    /// Clear historical data
    pub fn clear_historical_data(&mut self) {
        self.historical_data.clear();
    }
}

/// Builder for satisficing evaluator
pub struct SatisficingEvaluatorBuilder {
    config: SatisficingConfig,
}

impl SatisficingEvaluatorBuilder {
    pub fn new() -> Self {
        Self {
            config: SatisficingConfig {
                max_iterations: 5,
                min_improvement_threshold: 5.0, // 5% improvement minimum
                cost_benefit_threshold: 0.1,     // 0.1 quality points per second
                diminishing_returns_threshold: 1.0, // 1% improvement
                time_budget_per_refinement_seconds: 300, // 5 minutes
                enable_learning: true,
            },
        }
    }

    pub fn config(mut self, config: SatisficingConfig) -> Self {
        self.config = config;
        self
    }

    pub fn max_iterations(mut self, max: usize) -> Self {
        self.config.max_iterations = max;
        self
    }

    pub fn min_improvement_threshold(mut self, threshold: f64) -> Self {
        self.config.min_improvement_threshold = threshold;
        self
    }

    pub fn cost_benefit_threshold(mut self, threshold: f64) -> Self {
        self.config.cost_benefit_threshold = threshold;
        self
    }

    pub fn build(self) -> SatisficingEvaluator {
        SatisficingEvaluator::new(self.config)
    }
}

impl Default for SatisficingEvaluatorBuilder {
    fn default() -> Self {
        Self::new()
    }
}

pub type Result<T> = std::result::Result<T, SatisficingError>;

#[derive(Debug, thiserror::Error)]
pub enum SatisficingError {
    #[error("Insufficient historical data for analysis")]
    InsufficientData,

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Analysis failed: {0}")]
    AnalysisError(String),
}
