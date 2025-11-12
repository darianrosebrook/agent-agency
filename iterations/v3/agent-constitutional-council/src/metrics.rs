//! Council Metrics and Observability
//!
//! This module provides comprehensive metrics and observability for the
//! constitutional council, tracking performance, decision quality, and
//! system health.
//!
//! ## Metrics Categories
//!
//! - **Session Metrics**: Council sessions, decision latency
//! - **Judge Metrics**: Per-judge performance, verdict distributions
//! - **Decision Quality**: Consensus tracking, violation patterns
//! - **Performance**: Latency percentiles, throughput
//! - **Health**: Error rates, cache effectiveness

use schemars::JsonSchema;
use std::collections::HashMap;
use std::time::{Duration, Instant};

use agent_agency_contracts::{JudgeType, VerdictLabel, JudgeVerdict};

/// Council performance and observability metrics
#[derive(Debug)]
pub struct CouncilMetrics {
    /// Total council sessions processed
    pub sessions_total: u64,

    /// Sessions by verdict label
    pub verdicts_by_label: HashMap<VerdictLabel, u64>,

    /// Judge-specific latency tracking (milliseconds)
    pub judge_latency_ms: HashMap<JudgeType, Vec<u64>>,

    /// End-to-end evaluation latency (milliseconds)
    pub evaluation_latency_ms: Vec<u64>,

    /// Consensus failure count
    pub consensus_failures: u64,

    /// Non-waivable violation count
    pub non_waivable_violations: u64,

    /// Judge verdict score distributions
    pub judge_score_distributions: HashMap<JudgeType, Vec<f32>>,

    /// Cache hit rates (if applicable)
    pub cache_hit_rate: Option<f64>,

    /// Last evaluation timestamp
    pub last_evaluation: Option<Instant>,
}

impl CouncilMetrics {
    /// Create new metrics instance
    pub fn new() -> Self {
        Self {
            sessions_total: 0,
            verdicts_by_label: HashMap::new(),
            judge_latency_ms: HashMap::new(),
            evaluation_latency_ms: Vec::new(),
            consensus_failures: 0,
            non_waivable_violations: 0,
            judge_score_distributions: HashMap::new(),
            cache_hit_rate: None,
            last_evaluation: None,
        }
    }

    /// Record a council session
    pub fn record_session(&mut self) {
        self.sessions_total += 1;
    }

    /// Record evaluation completion
    pub fn record_evaluation(&mut self, duration: Duration, judge_verdicts: &[(JudgeType, JudgeVerdict)], final_decision: &crate::FinalDecision) {
        let duration_ms = duration.as_millis() as u64;
        self.evaluation_latency_ms.push(duration_ms);

        // Record verdict distribution
        *self.verdicts_by_label.entry(final_decision.label.clone()).or_insert(0) += 1;

        // Record judge metrics
        for (judge_type, verdict) in judge_verdicts {
            self.judge_score_distributions.entry(judge_type.clone())
                .or_insert_with(Vec::new)
                .push(verdict.score);
        }

        // Track consensus issues
        if !final_decision.consensus_violations.is_empty() {
            self.consensus_failures += 1;
        }

        // Track non-waivable violations
        let total_non_waivable = judge_verdicts.iter()
            .flat_map(|(_, verdict)| &verdict.violations)
            .filter(|v| !v.waivable && v.severity == agent_agency_contracts::judge_io::Severity::Critical)
            .count();
        self.non_waivable_violations += total_non_waivable as u64;

        self.last_evaluation = Some(Instant::now());
    }

    /// Record judge latency
    pub fn record_judge_latency(&mut self, judge_type: JudgeType, latency_ms: u64) {
        self.judge_latency_ms.entry(judge_type)
            .or_insert_with(Vec::new)
            .push(latency_ms);
    }

    /// Get average evaluation latency
    pub fn average_evaluation_latency_ms(&self) -> Option<f64> {
        if self.evaluation_latency_ms.is_empty() {
            None
        } else {
            Some(self.evaluation_latency_ms.iter().sum::<u64>() as f64 / self.evaluation_latency_ms.len() as f64)
        }
    }

    /// Get P95 evaluation latency
    pub fn p95_evaluation_latency_ms(&self) -> Option<u64> {
        if self.evaluation_latency_ms.is_empty() {
            None
        } else {
            let mut sorted = self.evaluation_latency_ms.clone();
            sorted.sort_unstable();
            let p95_index = (sorted.len() as f64 * 0.95) as usize;
            Some(sorted[p95_index.min(sorted.len() - 1)])
        }
    }

    /// Get verdict distribution as percentages
    pub fn verdict_distribution(&self) -> HashMap<VerdictLabel, f64> {
        let mut distribution = HashMap::new();
        if self.sessions_total == 0 {
            return distribution;
        }

        for (label, count) in &self.verdicts_by_label {
            distribution.insert(label.clone(), *count as f64 / self.sessions_total as f64);
        }

        distribution
    }

    /// Get average judge scores
    pub fn average_judge_scores(&self) -> HashMap<JudgeType, f64> {
        let mut averages = HashMap::new();

        for (judge_type, scores) in &self.judge_score_distributions {
            if !scores.is_empty() {
                let avg = scores.iter().sum::<f32>() / scores.len() as f32;
                averages.insert(judge_type.clone(), avg as f64);
            }
        }

        averages
    }

    /// Get consensus failure rate
    pub fn consensus_failure_rate(&self) -> f64 {
        if self.sessions_total == 0 {
            0.0
        } else {
            self.consensus_failures as f64 / self.sessions_total as f64
        }
    }

    /// Get non-waivable violation rate
    pub fn non_waivable_violation_rate(&self) -> f64 {
        if self.sessions_total == 0 {
            0.0
        } else {
            self.non_waivable_violations as f64 / self.sessions_total as f64
        }
    }

    /// Generate health report
    pub fn health_report(&self) -> CouncilHealthReport {
        CouncilHealthReport {
            sessions_total: self.sessions_total,
            average_latency_ms: self.average_evaluation_latency_ms(),
            p95_latency_ms: self.p95_evaluation_latency_ms(),
            verdict_distribution: self.verdict_distribution(),
            average_judge_scores: self.average_judge_scores(),
            consensus_failure_rate: self.consensus_failure_rate(),
            non_waivable_violation_rate: self.non_waivable_violation_rate(),
            last_evaluation_minutes_ago: self.last_evaluation
                .map(|t| t.elapsed().as_secs() / 60)
                .unwrap_or(u64::MAX),
        }
    }
}

/// Health report for council monitoring
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, JsonSchema)]
pub struct CouncilHealthReport {
    /// Total sessions processed
    pub sessions_total: u64,

    /// Average evaluation latency (ms)
    pub average_latency_ms: Option<f64>,

    /// P95 evaluation latency (ms)
    pub p95_latency_ms: Option<u64>,

    /// Verdict distribution (label -> percentage)
    pub verdict_distribution: HashMap<VerdictLabel, f64>,

    /// Average scores by judge type
    pub average_judge_scores: HashMap<JudgeType, f64>,

    /// Consensus failure rate (0.0-1.0)
    pub consensus_failure_rate: f64,

    /// Non-waivable violation rate per session
    pub non_waivable_violation_rate: f64,

    /// Minutes since last evaluation
    pub last_evaluation_minutes_ago: u64,
}

impl Default for CouncilMetrics {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_metrics_recording() {
        let mut metrics = CouncilMetrics::new();

        // Record a session
        metrics.record_session();
        assert_eq!(metrics.sessions_total, 1);

        // Create mock verdict
        let verdict = agent_agency_contracts::JudgeVerdict {
            score: 0.85,
            label: VerdictLabel::Pass,
            rationale: "Good spec".to_string(),
            violations: vec![],
            evidence_refs: vec![],
        };

        let final_decision = crate::FinalDecision {
            label: VerdictLabel::Pass,
            score: 0.85,
            rationale: "Approved".to_string(),
            judge_verdicts: vec![verdict.clone()],
            consensus_violations: vec![],
            recommended_actions: vec!["Proceed".to_string()],
        };

        // Record evaluation - create tuples of (JudgeType, JudgeVerdict)
        let judge_verdicts_with_types: Vec<(JudgeType, agent_agency_contracts::JudgeVerdict)> = vec![
            (JudgeType::Constitutional, verdict.clone()),
        ];
        metrics.record_evaluation(Duration::from_millis(150), &judge_verdicts_with_types, &final_decision);

        assert_eq!(metrics.evaluation_latency_ms, vec![150]);
        assert_eq!(metrics.verdicts_by_label[&VerdictLabel::Pass], 1);
    }

    #[test]
    fn test_average_evaluation_latency() {
        let mut metrics = CouncilMetrics::new();

        // Add some latencies
        metrics.evaluation_latency_ms = vec![100, 200, 150];

        let avg = metrics.average_evaluation_latency_ms().unwrap();
        assert_eq!(avg, 150.0);
    }

    #[test]
    fn test_verdict_distribution() {
        let mut metrics = CouncilMetrics::new();
        metrics.sessions_total = 10;
        metrics.verdicts_by_label.insert(VerdictLabel::Pass, 7);
        metrics.verdicts_by_label.insert(VerdictLabel::Fail, 3);

        let dist = metrics.verdict_distribution();
        assert_eq!(dist[&VerdictLabel::Pass], 0.7);
        assert_eq!(dist[&VerdictLabel::Fail], 0.3);
    }
}
