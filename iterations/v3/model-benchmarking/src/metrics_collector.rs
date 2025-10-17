//! Metrics collection for benchmarking

use crate::types::*;
use anyhow::Result;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use uuid::Uuid;

pub struct MetricsCollector {
    /// Storage for benchmark results
    benchmark_results: Arc<RwLock<HashMap<Uuid, Vec<BenchmarkResult>>>>,
    /// Performance history for trend analysis
    performance_history: Arc<RwLock<Vec<PerformanceSnapshot>>>,
    /// Model performance summaries
    model_summaries: Arc<RwLock<HashMap<Uuid, ModelPerformanceSummary>>>,
}

#[derive(Debug, Clone)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub model_id: Uuid,
    pub benchmark_type: BenchmarkType,
    pub metrics: BenchmarkMetrics,
    pub score: f64,
}

#[derive(Debug, Clone)]
pub struct ModelPerformanceSummary {
    pub model_id: Uuid,
    pub model_name: String,
    pub total_benchmarks: usize,
    pub average_score: f64,
    pub best_score: f64,
    pub worst_score: f64,
    pub last_updated: DateTime<Utc>,
    pub performance_trend: PerformanceTrend,
}

impl MetricsCollector {
    pub fn new() -> Self {
        Self {
            benchmark_results: Arc::new(RwLock::new(HashMap::new())),
            performance_history: Arc::new(RwLock::new(Vec::new())),
            model_summaries: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Store a benchmark result
    pub async fn store_benchmark_result(&self, result: BenchmarkResult) -> Result<()> {
        info!("Storing benchmark result for model {}", result.model_id);

        let mut results = self.benchmark_results.write().await;
        let model_results = results.entry(result.model_id).or_insert_with(Vec::new);
        model_results.push(result.clone());

        // Keep only the last 100 results per model to prevent memory bloat
        if model_results.len() > 100 {
            model_results.drain(0..model_results.len() - 100);
        }

        // Update performance history
        self.update_performance_history(&result).await?;

        // Update model summary
        self.update_model_summary(&result).await?;

        debug!("Stored benchmark result successfully");
        Ok(())
    }

    /// Get benchmark results for a specific model
    pub async fn get_model_benchmark_results(
        &self,
        model_id: Uuid,
    ) -> Result<Vec<BenchmarkResult>> {
        let results = self.benchmark_results.read().await;
        Ok(results.get(&model_id).cloned().unwrap_or_default())
    }

    /// Get performance history for a model
    pub async fn get_performance_history(
        &self,
        model_id: Uuid,
        limit: Option<usize>,
    ) -> Result<Vec<PerformanceSnapshot>> {
        let history = self.performance_history.read().await;
        let mut model_history: Vec<PerformanceSnapshot> = history
            .iter()
            .filter(|snapshot| snapshot.model_id == model_id)
            .cloned()
            .collect();

        // Sort by timestamp (newest first)
        model_history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        if let Some(limit) = limit {
            model_history.truncate(limit);
        }

        Ok(model_history)
    }

    /// Get model performance summary
    pub async fn get_model_summary(
        &self,
        model_id: Uuid,
    ) -> Result<Option<ModelPerformanceSummary>> {
        let summaries = self.model_summaries.read().await;
        Ok(summaries.get(&model_id).cloned())
    }

    /// Get all model summaries
    pub async fn get_all_model_summaries(&self) -> Result<Vec<ModelPerformanceSummary>> {
        let summaries = self.model_summaries.read().await;
        Ok(summaries.values().cloned().collect())
    }

    /// Calculate performance trends
    pub async fn calculate_performance_trend(
        &self,
        model_id: Uuid,
        window_size: usize,
    ) -> Result<PerformanceTrend> {
        let history = self
            .get_performance_history(model_id, Some(window_size))
            .await?;

        if history.len() < 3 {
            return Ok(PerformanceTrend::Stable);
        }

        // Calculate trend using linear regression on recent scores
        let scores: Vec<f64> = history.iter().map(|snapshot| snapshot.score).collect();
        let trend = self.calculate_linear_trend(&scores);

        Ok(match trend {
            t if t > 0.05 => PerformanceTrend::Improving,
            t if t < -0.05 => PerformanceTrend::Declining,
            _ => PerformanceTrend::Stable,
        })
    }

    /// Update performance history with new benchmark result
    async fn update_performance_history(&self, result: &BenchmarkResult) -> Result<()> {
        let mut history = self.performance_history.write().await;

        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            model_id: result.model_id,
            benchmark_type: result.benchmark_type.clone(),
            metrics: result.metrics.clone(),
            score: result.score,
        };

        history.push(snapshot);

        // Keep only the last 1000 snapshots to prevent memory bloat
        if history.len() > 1000 {
            let len = history.len();
            history.drain(0..len - 1000);
        }

        Ok(())
    }

    /// Update model summary with new benchmark result
    async fn update_model_summary(&self, result: &BenchmarkResult) -> Result<()> {
        // Calculate performance trend first (before acquiring write lock)
        let performance_trend = self
            .calculate_performance_trend(result.model_id, 10)
            .await
            .unwrap_or(PerformanceTrend::Stable);

        // Get model results for average calculation
        let model_results = self.benchmark_results.read().await;
        let average_score = if let Some(results) = model_results.get(&result.model_id) {
            let total_score: f64 = results.iter().map(|r| r.score).sum();
            total_score / results.len() as f64
        } else {
            0.0
        };
        drop(model_results); // Release read lock

        // Now acquire write lock and update summary
        let mut summaries = self.model_summaries.write().await;

        let summary = summaries
            .entry(result.model_id)
            .or_insert_with(|| ModelPerformanceSummary {
                model_id: result.model_id,
                model_name: format!("Model-{}", result.model_id),
                total_benchmarks: 0,
                average_score: 0.0,
                best_score: 0.0,
                worst_score: 1.0,
                last_updated: Utc::now(),
                performance_trend: PerformanceTrend::Stable,
            });

        // Update summary statistics
        summary.total_benchmarks += 1;
        summary.best_score = summary.best_score.max(result.score);
        summary.worst_score = summary.worst_score.min(result.score);
        summary.last_updated = Utc::now();
        summary.average_score = average_score;
        summary.performance_trend = performance_trend;

        Ok(())
    }

    /// Calculate linear trend from a series of scores
    fn calculate_linear_trend(&self, scores: &[f64]) -> f64 {
        if scores.len() < 2 {
            return 0.0;
        }

        let n = scores.len() as f64;
        let x_mean = (n - 1.0) / 2.0;
        let y_mean = scores.iter().sum::<f64>() / n;

        let numerator: f64 = scores
            .iter()
            .enumerate()
            .map(|(i, score)| (i as f64 - x_mean) * (score - y_mean))
            .sum();

        let denominator: f64 = scores
            .iter()
            .enumerate()
            .map(|(i, _)| (i as f64 - x_mean).powi(2))
            .sum();

        if denominator == 0.0 {
            0.0
        } else {
            numerator / denominator
        }
    }

    /// Generate performance report
    pub async fn generate_performance_report(&self) -> Result<PerformanceReport> {
        let summaries = self.get_all_model_summaries().await?;
        let history = self.performance_history.read().await;

        let mut report = PerformanceReport {
            generated_at: Utc::now(),
            total_models: summaries.len(),
            total_benchmarks: history.len(),
            average_performance: 0.0,
            top_performers: Vec::new(),
            performance_distribution: HashMap::new(),
            recommendations: Vec::new(),
        };

        if !summaries.is_empty() {
            report.average_performance =
                summaries.iter().map(|s| s.average_score).sum::<f64>() / summaries.len() as f64;

            // Find top performers
            let mut sorted_summaries = summaries.clone();
            sorted_summaries.sort_by(|a, b| b.average_score.partial_cmp(&a.average_score).unwrap());
            report.top_performers = sorted_summaries.into_iter().take(5).collect();

            // Calculate performance distribution
            for summary in &summaries {
                let category = match summary.average_score {
                    s if s >= 0.8 => "Excellent",
                    s if s >= 0.6 => "Good",
                    s if s >= 0.4 => "Average",
                    _ => "Poor",
                };
                *report
                    .performance_distribution
                    .entry(category.to_string())
                    .or_insert(0) += 1;
            }
        }

        // Generate recommendations
        self.generate_recommendations(&mut report, &summaries).await;

        Ok(report)
    }

    /// Generate performance recommendations
    async fn generate_recommendations(
        &self,
        report: &mut PerformanceReport,
        summaries: &[ModelPerformanceSummary],
    ) {
        for summary in summaries {
            match summary.performance_trend {
                PerformanceTrend::Declining => {
                    report.recommendations.push(Recommendation {
                        recommendation_type: RecommendationType::PerformanceOptimization,
                        description: format!(
                            "Model {} shows declining performance trend",
                            summary.model_name
                        ),
                        priority: Priority::High,
                        expected_impact: 0.2,
                    });
                }
                PerformanceTrend::Stable if summary.average_score < 0.5 => {
                    report.recommendations.push(Recommendation {
                        recommendation_type: RecommendationType::QualityImprovement,
                        description: format!(
                            "Model {} has consistently low performance",
                            summary.model_name
                        ),
                        priority: Priority::Medium,
                        expected_impact: 0.3,
                    });
                }
                _ => {}
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct PerformanceReport {
    pub generated_at: DateTime<Utc>,
    pub total_models: usize,
    pub total_benchmarks: usize,
    pub average_performance: f64,
    pub top_performers: Vec<ModelPerformanceSummary>,
    pub performance_distribution: HashMap<String, usize>,
    pub recommendations: Vec<Recommendation>,
}
