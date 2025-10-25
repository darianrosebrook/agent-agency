//! Resource management and prediction
//!
//! This module provides resource usage analysis, prediction,
//! and optimization capabilities based on historical data.

use anyhow::Result;
use chrono::{DateTime, Utc};
use std::time::Duration;

use crate::types::{ResourceTrend, ResourceUsageMetrics, ResourcePrediction};
use super::storage::*;
use super::types::*;

/// Resource manager for optimizing resource allocation
pub struct ResourceManager {
    storage: Box<dyn LearningSignalStorage>,
}

impl ResourceManager {
    /// Create a new resource manager
    pub fn new(storage: Box<dyn LearningSignalStorage>) -> Self {
        Self { storage }
    }

    /// Analyze resource usage patterns from historical data
    pub async fn analyze_resource_usage_patterns(
        &self,
        historical_data: &HistoricalResourceData,
    ) -> Result<ResourceUsagePatterns> {
        let mut cpu_usage = Vec::new();
        let mut memory_usage = Vec::new();
        let mut io_usage = Vec::new();

        // Extract resource usage over time
        for entry in &historical_data.entries {
            cpu_usage.push(entry.resource_usage.cpu_percent);
            memory_usage.push(entry.resource_usage.memory_mb as f32);
            io_usage.push(entry.resource_usage.io_bytes_per_sec as f32);
        }

        // Calculate patterns
        let cpu_pattern = self.calculate_resource_pattern(&cpu_usage, "CPU")?;
        let memory_pattern = self.calculate_resource_pattern(&memory_usage, "Memory")?;
        let io_pattern = self.calculate_resource_pattern(&io_usage, "IO")?;

        // Analyze seasonal patterns
        let seasonal_patterns = self.analyze_seasonal_patterns(historical_data)?;

        // Detect anomalies
        let anomaly_patterns = self.detect_resource_anomalies(historical_data)?;

        Ok(ResourceUsagePatterns {
            cpu_pattern,
            memory_pattern,
            io_pattern,
            seasonal_patterns,
            anomaly_patterns,
        })
    }

    /// Calculate resource usage pattern from data points
    fn calculate_resource_pattern(&self, data: &[f32], resource_type: &str) -> Result<ResourcePattern> {
        if data.is_empty() {
            return Ok(ResourcePattern {
                average: 0.0,
                peak: 0.0,
                trend: "insufficient_data".to_string(),
                confidence: 0.0,
            });
        }

        let average = data.iter().sum::<f32>() / data.len() as f32;
        let peak = data.iter().fold(0.0, |max, &val| if val > max { val } else { max });

        // Simple trend analysis (could be more sophisticated)
        let trend = if data.len() >= 2 {
            let first_half_avg = data[..data.len()/2].iter().sum::<f32>() / (data.len()/2) as f32;
            let second_half_avg = data[data.len()/2..].iter().sum::<f32>() / (data.len()/2) as f32;

            if second_half_avg > first_half_avg * 1.1 {
                "increasing"
            } else if second_half_avg < first_half_avg * 0.9 {
                "decreasing"
            } else {
                "stable"
            }
        } else {
            "stable"
        };

        Ok(ResourcePattern {
            average,
            peak,
            trend: trend.to_string(),
            confidence: 0.8, // Simplified confidence calculation
        })
    }

    /// Analyze seasonal patterns in resource usage
    fn analyze_seasonal_patterns(&self, historical_data: &HistoricalResourceData) -> Result<Vec<SeasonalPattern>> {
        // Simplified seasonal analysis
        let mut patterns = Vec::new();

        // Check for daily patterns
        if historical_data.entries.len() > 24 {
            patterns.push(SeasonalPattern {
                pattern_type: "daily".to_string(),
                description: "Resource usage shows daily patterns".to_string(),
                impact: "Schedule resource-intensive tasks during off-peak hours".to_string(),
                confidence: 0.7,
            });
        }

        // Check for complexity-based patterns
        let complex_tasks: Vec<_> = historical_data.entries.iter()
            .filter(|e| matches!(e.task_complexity, TaskComplexity::Complex | TaskComplexity::VeryComplex))
            .collect();

        if !complex_tasks.is_empty() {
            let avg_complex_cpu = complex_tasks.iter()
                .map(|e| e.resource_usage.cpu_percent)
                .sum::<f32>() / complex_tasks.len() as f32;

            patterns.push(SeasonalPattern {
                pattern_type: "complexity-based".to_string(),
                description: format!("Complex tasks average {:.1}% CPU usage", avg_complex_cpu),
                impact: "Allocate additional resources for complex task processing".to_string(),
                confidence: 0.8,
            });
        }

        Ok(patterns)
    }

    /// Detect resource usage anomalies
    fn detect_resource_anomalies(&self, historical_data: &HistoricalResourceData) -> Result<Vec<ResourceAnomaly>> {
        let mut anomalies = Vec::new();

        if historical_data.entries.len() < 10 {
            return Ok(anomalies); // Need minimum data for anomaly detection
        }

        // Calculate baseline statistics
        let cpu_values: Vec<f32> = historical_data.entries.iter()
            .map(|e| e.resource_usage.cpu_percent)
            .collect();

        let cpu_avg = cpu_values.iter().sum::<f32>() / cpu_values.len() as f32;
        let cpu_std = (cpu_values.iter()
            .map(|v| (v - cpu_avg).powi(2))
            .sum::<f32>() / cpu_values.len() as f32)
            .sqrt();

        // Detect CPU anomalies (values more than 2 standard deviations from mean)
        for (i, entry) in historical_data.entries.iter().enumerate() {
            let cpu_deviation = (entry.resource_usage.cpu_percent - cpu_avg).abs();
            if cpu_deviation > cpu_std * 2.0 {
                anomalies.push(ResourceAnomaly {
                    timestamp: entry.timestamp,
                    resource_type: "CPU".to_string(),
                    deviation: cpu_deviation / cpu_avg, // Percentage deviation
                    description: format!(
                        "CPU usage {:.1}% is {:.1} standard deviations from mean",
                        entry.resource_usage.cpu_percent,
                        cpu_deviation / cpu_std
                    ),
                    severity: if cpu_deviation > cpu_std * 3.0 { "high" } else { "medium" }.to_string(),
                });
            }
        }

        Ok(anomalies)
    }

    /// Generate resource usage predictions
    pub async fn generate_resource_predictions(
        &self,
        historical_data: &HistoricalResourceData,
        trends: &[ResourceTrend],
    ) -> Result<Vec<ResourcePrediction>> {
        let mut predictions = Vec::new();

        if historical_data.entries.is_empty() {
            return Ok(predictions);
        }

        // Predict CPU requirements
        let cpu_prediction = self.predict_cpu_requirements(historical_data)?;
        predictions.push(cpu_prediction);

        // Predict memory requirements
        let memory_prediction = self.predict_memory_requirements(historical_data)?;
        predictions.push(memory_prediction);

        // Predict IO requirements
        let io_prediction = self.predict_io_requirements(historical_data)?;
        predictions.push(io_prediction);

        Ok(predictions)
    }

    /// Predict CPU requirements based on historical data
    fn predict_cpu_requirements(&self, historical_data: &HistoricalResourceData) -> Result<ResourcePrediction> {
        let recent_cpu: Vec<f32> = historical_data.entries.iter()
            .rev()
            .take(10)
            .map(|e| e.resource_usage.cpu_percent)
            .collect();

        let avg_cpu = recent_cpu.iter().sum::<f32>() / recent_cpu.len() as f32;
        let peak_cpu = recent_cpu.iter().fold(0.0, |max, &val| if val > max { val } else { max });

        // Simple prediction: add 10% buffer
        let predicted_cpu = (avg_cpu * 1.1).min(100.0);

        Ok(ResourcePrediction {
            resource_type: "CPU".to_string(),
            predicted_usage_percent: predicted_cpu,
            confidence: 0.75,
            time_horizon_hours: 24,
            risk_factors: vec!["variable_workload".to_string()],
        })
    }

    /// Predict memory requirements based on historical data
    fn predict_memory_requirements(&self, historical_data: &HistoricalResourceData) -> Result<ResourcePrediction> {
        let recent_memory: Vec<f32> = historical_data.entries.iter()
            .rev()
            .take(10)
            .map(|e| e.resource_usage.memory_mb as f32)
            .collect();

        let avg_memory = recent_memory.iter().sum::<f32>() / recent_memory.len() as f32;
        let predicted_memory = avg_memory * 1.15; // Add 15% buffer for memory

        Ok(ResourcePrediction {
            resource_type: "Memory".to_string(),
            predicted_usage_percent: (predicted_memory / 8192.0 * 100.0).min(100.0), // Assuming 8GB system
            confidence: 0.8,
            time_horizon_hours: 24,
            risk_factors: vec!["memory_intensive_tasks".to_string()],
        })
    }

    /// Predict IO requirements based on historical data
    fn predict_io_requirements(&self, historical_data: &HistoricalResourceData) -> Result<ResourcePrediction> {
        let recent_io: Vec<f32> = historical_data.entries.iter()
            .rev()
            .take(10)
            .map(|e| e.resource_usage.io_bytes_per_sec as f32 / 1_000_000.0) // Convert to MB/s
            .collect();

        let avg_io = recent_io.iter().sum::<f32>() / recent_io.len() as f32;

        Ok(ResourcePrediction {
            resource_type: "IO".to_string(),
            predicted_usage_percent: (avg_io / 100.0 * 100.0).min(100.0), // Assuming 100MB/s max
            confidence: 0.7,
            time_horizon_hours: 24,
            risk_factors: vec!["data_processing_workload".to_string()],
        })
    }

    /// Generate resource allocation recommendations
    pub async fn generate_resource_allocation_recommendations(
        &self,
        task_spec: &crate::types::TaskSpec,
    ) -> Result<PredictedResourceRequirements> {
        // Get historical data for similar tasks
        let historical_data = self.storage.perform_comprehensive_historical_resource_lookup(task_spec).await?;

        if historical_data.entries.is_empty() {
            // Return default recommendations if no historical data
            return Ok(PredictedResourceRequirements {
                cpu_percent: 50.0,
                memory_mb: 2048,
                io_bytes_per_sec: 10_000_000,
                estimated_duration_ms: 30_000,
                confidence: 0.5,
                risk_factors: vec!["no_historical_data".to_string()],
            });
        }

        // Analyze patterns and generate predictions
        let patterns = self.analyze_resource_usage_patterns(&historical_data).await?;
        let trends = self.storage.analyze_resource_usage_trends(&historical_data).await?;
        let predictions = self.generate_resource_predictions(&historical_data, &trends).await?;

        // Calculate recommended allocation
        let cpu_prediction = predictions.iter()
            .find(|p| p.resource_type == "CPU")
            .map(|p| p.predicted_usage_percent)
            .unwrap_or(50.0);

        let memory_prediction = predictions.iter()
            .find(|p| p.resource_type == "Memory")
            .map(|p| p.predicted_usage_percent)
            .unwrap_or(25.0);

        let io_prediction = predictions.iter()
            .find(|p| p.resource_type == "IO")
            .map(|p| p.predicted_usage_percent)
            .unwrap_or(10.0);

        // Estimate duration based on historical data
        let avg_duration = historical_data.entries.iter()
            .map(|e| e.duration_ms)
            .sum::<u64>() / historical_data.entries.len() as u64;

        let confidence = predictions.iter()
            .map(|p| p.confidence)
            .sum::<f32>() / predictions.len() as f32;

        let mut risk_factors = Vec::new();
        for prediction in &predictions {
            risk_factors.extend(prediction.risk_factors.clone());
        }

        Ok(PredictedResourceRequirements {
            cpu_percent: cpu_prediction,
            memory_mb: ((memory_prediction / 100.0) * 8192.0) as u32, // Convert percentage to MB
            io_bytes_per_sec: ((io_prediction / 100.0) * 100_000_000.0) as u64, // Convert percentage to bytes/sec
            estimated_duration_ms: avg_duration,
            confidence,
            risk_factors,
        })
    }
}
