//! Regression detection for model performance

use crate::types::*;
use anyhow::Result;
use chrono::Utc;
use std::collections::HashMap;

/// Regression detector for performance monitoring
pub struct RegressionDetector {}

impl RegressionDetector {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn check_for_regressions(
        &self,
        results: &[BenchmarkResult],
    ) -> Result<Vec<RegressionAlert>> {
        const SCORE_THRESHOLD: f64 = 0.08;
        const METRIC_THRESHOLD: f64 = 0.07;

        let mut alerts = Vec::new();
        let mut score_baselines: HashMap<(Uuid, String), f64> = HashMap::new();
        let mut metric_baselines: HashMap<(Uuid, String, &'static str), f64> = HashMap::new();

        for result in results {
            let benchmark_key = format!("{:?}", result.benchmark_type);
            let score_key = (result.model_id, benchmark_key.clone());

            if let Some(previous_score) = score_baselines.get(&score_key) {
                let drop = previous_score - result.score;
                if drop > SCORE_THRESHOLD && previous_score.abs() > f64::EPSILON {
                    alerts.push(Self::build_alert(
                        result.model_id,
                        "score",
                        result.score,
                        *previous_score,
                        drop / previous_score * 100.0,
                    ));
                }

                // Update baseline with a light decay to follow long-term trend.
                let updated = (*previous_score * 0.7) + (result.score * 0.3);
                score_baselines.insert(score_key.clone(), updated);
            } else {
                score_baselines.insert(score_key.clone(), result.score);
            }

            let metrics = &result.metrics;
            for (metric_name, current, baseline_entry) in [
                ("accuracy", metrics.accuracy, &mut metric_baselines),
                ("speed", metrics.speed, &mut metric_baselines),
                ("efficiency", metrics.efficiency, &mut metric_baselines),
                ("quality", metrics.quality, &mut metric_baselines),
                ("compliance", metrics.compliance, &mut metric_baselines),
            ] {
                let metric_key = (result.model_id, benchmark_key.clone(), metric_name);
                if let Some(previous_metric) = baseline_entry.get(&metric_key) {
                    let drop = previous_metric - current;
                    if drop > METRIC_THRESHOLD && previous_metric.abs() > f64::EPSILON {
                        alerts.push(Self::build_alert(
                            result.model_id,
                            metric_name,
                            current,
                            *previous_metric,
                            drop / previous_metric * 100.0,
                        ));
                    }
                    let updated = (*previous_metric * 0.65) + (current * 0.35);
                    baseline_entry.insert(metric_key, updated);
                } else {
                    baseline_entry.insert(metric_key, current);
                }
            }
        }

        Ok(alerts)
    }

    fn build_alert(
        model_id: Uuid,
        metric_name: &str,
        current_value: f64,
        previous_value: f64,
        regression_percentage: f64,
    ) -> RegressionAlert {
        RegressionAlert {
            model_id,
            metric_name: metric_name.to_string(),
            current_value,
            previous_value,
            regression_percentage,
            severity: Self::map_severity(regression_percentage),
            timestamp: Utc::now(),
        }
    }

    fn map_severity(regression_percentage: f64) -> RegressionSeverity {
        if regression_percentage >= 35.0 {
            RegressionSeverity::Critical
        } else if regression_percentage >= 20.0 {
            RegressionSeverity::High
        } else if regression_percentage >= 10.0 {
            RegressionSeverity::Medium
        } else {
            RegressionSeverity::Low
        }
    }
}
