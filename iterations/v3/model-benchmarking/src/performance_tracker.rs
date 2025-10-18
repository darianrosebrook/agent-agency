//! Performance tracking for models

use crate::types::*;
use anyhow::Result;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct PerformanceTracker {
    /// Active model specifications
    active_models: Arc<RwLock<Vec<ModelSpecification>>>,
    /// Stored benchmark reports
    benchmark_reports: Arc<RwLock<HashMap<Uuid, BenchmarkReport>>>,
    /// Stored evaluation results
    evaluation_results: Arc<RwLock<HashMap<Uuid, ModelEvaluationResult>>>,
    /// Model performance history
    model_performance: Arc<RwLock<HashMap<Uuid, Vec<ModelPerformance>>>>,
    /// Historical benchmark results
    historical_benchmarks: Arc<RwLock<HashMap<Uuid, Vec<BenchmarkResult>>>>,
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            active_models: Arc::new(RwLock::new(Vec::new())),
            benchmark_reports: Arc::new(RwLock::new(HashMap::new())),
            evaluation_results: Arc::new(RwLock::new(HashMap::new())),
            model_performance: Arc::new(RwLock::new(HashMap::new())),
            historical_benchmarks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn get_active_models(&self) -> Result<Vec<ModelSpecification>> {
        // Retrieve active models from storage
        let active_models = self.active_models.read().await.clone();

        // Validate models are still available (basic check)
        let validated_models = active_models
            .into_iter()
            .filter(|model| {
                // Basic validation: ensure model has required fields
                !model.name.is_empty() && model.id != Uuid::nil()
            })
            .collect();

        Ok(validated_models)
    }

    pub async fn store_benchmark_report(&self, report: &BenchmarkReport) -> Result<()> {
        // Store benchmark report in memory
        let mut reports = self.benchmark_reports.write().await;
        reports.insert(report.report_id, report.clone());

        // Update historical benchmarks for each model in the report
        let mut historical = self.historical_benchmarks.write().await;
        for result in &report.benchmark_results {
            historical
                .entry(result.model_id)
                .or_insert_with(Vec::new)
                .push(result.clone());
        }

        Ok(())
    }

    pub async fn store_evaluation_result(&self, result: &ModelEvaluationResult) -> Result<()> {
        // Store evaluation result in memory
        let mut results = self.evaluation_results.write().await;
        results.insert(result.evaluation_id, result.clone());

        // Update model performance data
        let mut performance = self.model_performance.write().await;

        // Create or update performance entry for this model
        let model_perf = performance
            .entry(result.model_spec.id)
            .or_insert_with(Vec::new);

        // Calculate overall performance score
        let overall_score = result.evaluation_metrics.overall_score;

        // Determine strengths and weaknesses based on capability scores
        let mut strengths = Vec::new();
        let mut weaknesses = Vec::new();

        for cap_score in &result.evaluation_metrics.capability_scores {
            if cap_score.score >= 0.8 {
                strengths.push(format!("{:?}", cap_score.capability));
            } else if cap_score.score < 0.6 {
                weaknesses.push(format!("{:?}", cap_score.capability));
            }
        }

        let perf_entry = ModelPerformance {
            model_id: result.model_spec.id,
            model_name: result.model_spec.name.clone(),
            performance_score: overall_score,
            strengths,
            weaknesses,
        };

        model_perf.push(perf_entry);

        Ok(())
    }

    pub async fn get_model_performance(&self) -> Result<Vec<ModelPerformance>> {
        // Retrieve all model performance data
        let performance_data = self.model_performance.read().await;

        // Aggregate performance data - return latest performance for each model
        let mut aggregated_performance = Vec::new();

        for (model_id, performances) in performance_data.iter() {
            if let Some(latest_perf) = performances.last() {
                let mut perf = latest_perf.clone();
                // If we have multiple entries, calculate average performance
                if performances.len() > 1 {
                    let avg_score = performances
                        .iter()
                        .map(|p| p.performance_score)
                        .sum::<f64>()
                        / performances.len() as f64;
                    perf.performance_score = avg_score;
                }
                aggregated_performance.push(perf);
            }
        }

        // Sort by performance score (highest first)
        aggregated_performance.sort_by(|a, b| {
            b.performance_score
                .partial_cmp(&a.performance_score)
                .unwrap()
        });

        Ok(aggregated_performance)
    }

    pub async fn get_model_confidence(&self, model_id: Uuid) -> Result<f64> {
        // Calculate confidence based on historical performance data
        let performance_data = self.model_performance.read().await;

        if let Some(performances) = performance_data.get(&model_id) {
            if performances.is_empty() {
                return Ok(0.5); // Default confidence for new models
            }

            // Calculate confidence based on:
            // 1. Consistency of performance scores
            // 2. Number of evaluations
            // 3. Trend stability

            let scores: Vec<f64> = performances.iter().map(|p| p.performance_score).collect();

            let avg_score = scores.iter().sum::<f64>() / scores.len() as f64;

            // Calculate standard deviation for consistency
            let variance = scores
                .iter()
                .map(|score| (score - avg_score).powi(2))
                .sum::<f64>()
                / scores.len() as f64;
            let std_dev = variance.sqrt();

            // Consistency factor (lower std_dev = higher confidence)
            let consistency_factor = 1.0 - (std_dev / 2.0).min(1.0);

            // Volume factor (more evaluations = higher confidence)
            let volume_factor = (performances.len() as f64 / 10.0).min(1.0);

            // Trend factor (recent performance stability)
            let trend_factor = if performances.len() >= 3 {
                let recent_scores: Vec<f64> = performances
                    .iter()
                    .rev()
                    .take(3)
                    .map(|p| p.performance_score)
                    .collect();
                let recent_avg = recent_scores.iter().sum::<f64>() / recent_scores.len() as f64;
                let recent_std_dev = recent_scores
                    .iter()
                    .map(|score| (score - recent_avg).powi(2))
                    .sum::<f64>()
                    / recent_scores.len() as f64;
                1.0 - (recent_std_dev / 2.0).min(1.0)
            } else {
                0.5 // Neutral for models with few evaluations
            };

            // Combine factors with weights
            let confidence =
                (consistency_factor * 0.4) + (volume_factor * 0.3) + (trend_factor * 0.3);

            Ok(confidence.max(0.1).min(1.0))
        } else {
            Ok(0.1) // Very low confidence for unknown models
        }
    }

    pub async fn get_historical_performance(&self, model_id: Uuid) -> Result<Vec<BenchmarkResult>> {
        // Retrieve historical benchmark results for the specified model
        let historical_data = self.historical_benchmarks.read().await;

        if let Some(results) = historical_data.get(&model_id) {
            // Sort by timestamp (most recent first) and return copy
            let mut sorted_results = results.clone();
            sorted_results.sort_by(|a, b| {
                // TODO: Implement proper timestamp-based sorting with the following requirements:
                // 1. Timestamp integration: Add timestamp field to BenchmarkResult structure
                //    - Add timestamp field to BenchmarkResult data structure
                //    - Handle timestamp generation and management
                //    - Implement timestamp validation and quality assurance
                // 2. Sorting optimization: Implement efficient timestamp-based sorting
                //    - Implement timestamp-based sorting algorithms
                //    - Handle sorting performance optimization and caching
                //    - Implement sorting validation and quality assurance
                // 3. Historical data management: Manage historical benchmark data with timestamps
                //    - Track benchmark execution timestamps and history
                //    - Handle historical data storage and retrieval optimization
                //    - Implement historical data validation and integrity
                // 4. Performance analytics: Analyze benchmark performance trends over time
                //    - Generate performance trend analysis based on timestamps
                //    - Handle performance analytics and reporting
                //    - Ensure timestamp-based sorting meets performance and accuracy standards
                b.score.partial_cmp(&a.score).unwrap()
            });

            Ok(sorted_results)
        } else {
            Ok(vec![])
        }
    }

    /// Add a model to the active models list
    pub async fn add_active_model(&self, model: ModelSpecification) -> Result<()> {
        let mut active_models = self.active_models.write().await;
        // Remove if already exists to avoid duplicates
        active_models.retain(|m| m.id != model.id);
        active_models.push(model);
        Ok(())
    }

    /// Remove a model from the active models list
    pub async fn remove_active_model(&self, model_id: Uuid) -> Result<()> {
        let mut active_models = self.active_models.write().await;
        active_models.retain(|m| m.id != model_id);
        Ok(())
    }

    /// Get a specific active model by ID
    pub async fn get_active_model(&self, model_id: Uuid) -> Result<Option<ModelSpecification>> {
        let active_models = self.active_models.read().await;
        Ok(active_models.iter().find(|m| m.id == model_id).cloned())
    }

    /// Get performance summary for all models
    pub async fn get_performance_summary(&self) -> Result<PerformanceSummary> {
        let performances = self.get_model_performance().await?;

        if performances.is_empty() {
            return Ok(PerformanceSummary {
                overall_performance: 0.0,
                performance_trend: PerformanceTrend::Stable,
                top_performers: vec![],
                improvement_areas: vec![],
            });
        }

        // Calculate overall performance
        let overall_performance = performances
            .iter()
            .map(|p| p.performance_score)
            .sum::<f64>()
            / performances.len() as f64;

        // Determine performance trend (simplified)
        let trend = PerformanceTrend::Stable; // Would need more sophisticated analysis

        // Get top performers (top 3)
        let top_performers = performances.iter().take(3).cloned().collect();

        // Identify improvement areas (models with score < 0.7)
        let improvement_areas = performances
            .iter()
            .filter(|p| p.performance_score < 0.7)
            .map(|p| ImprovementArea {
                area: format!("{} performance", p.model_name),
                current_score: p.performance_score,
                target_score: 0.8,
                improvement_potential: 0.8 - p.performance_score,
            })
            .collect();

        Ok(PerformanceSummary {
            overall_performance,
            performance_trend: trend,
            top_performers,
            improvement_areas,
        })
    }
}
