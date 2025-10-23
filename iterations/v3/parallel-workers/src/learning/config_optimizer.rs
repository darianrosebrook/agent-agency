//! Configuration optimizer for parallel worker system

use crate::types::{WorkerSpecialty, TaskPattern};
use crate::learning::pattern_analyzer::PatternAnalyzer;
use crate::learning::metrics_collector::ExecutionRecord;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Configuration optimization event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationEvent {
    WorkerCountOptimized {
        task_pattern: TaskPattern,
        old_count: usize,
        new_count: usize,
        expected_improvement: f32,
    },
    TimeoutOptimized {
        task_pattern: TaskPattern,
        old_timeout: Duration,
        new_timeout: Duration,
        expected_improvement: f32,
    },
    QualityThresholdOptimized {
        task_pattern: TaskPattern,
        old_threshold: f32,
        new_threshold: f32,
        expected_improvement: f32,
    },
    StrategyOptimized {
        task_pattern: TaskPattern,
        old_strategy: String,
        new_strategy: String,
        expected_improvement: f32,
    },
}

/// Type of configuration optimization
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OptimizationType {
    WorkerCount,
    Timeout,
    QualityThreshold,
    Strategy,
    All,
}

/// Configuration recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigRecommendation {
    pub task_pattern: TaskPattern,
    pub optimization_type: OptimizationType,
    pub current_value: serde_json::Value,
    pub recommended_value: serde_json::Value,
    pub expected_impact: ExpectedImpact,
    pub confidence: f32,
    pub reasoning: String,
}

/// Expected impact of a configuration change
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedImpact {
    pub performance_improvement: f32,
    pub quality_improvement: f32,
    pub resource_efficiency_improvement: f32,
    pub risk_level: RiskLevel,
}

/// Risk level for configuration changes
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

/// Configuration recommendations container
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigurationRecommendations {
    pub recommendations: Vec<ConfigRecommendation>,
    pub overall_confidence: f32,
    pub estimated_improvement: f32,
    pub generated_at: DateTime<Utc>,
}

/// Configuration optimizer
pub struct ConfigurationOptimizer {
    pattern_analyzer: Arc<PatternAnalyzer>,
    optimization_history: Arc<RwLock<Vec<OptimizationEvent>>>,
    performance_tracking: Arc<RwLock<HashMap<TaskPattern, Vec<f32>>>>,
    config_recommendations: Arc<RwLock<HashMap<TaskPattern, ConfigurationRecommendations>>>,
}

impl ConfigurationOptimizer {
    /// Create a new configuration optimizer
    pub fn new(pattern_analyzer: Arc<PatternAnalyzer>) -> Self {
        Self {
            pattern_analyzer,
            optimization_history: Arc::new(RwLock::new(Vec::new())),
            performance_tracking: Arc::new(RwLock::new(HashMap::new())),
            config_recommendations: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    /// Analyze execution records and generate configuration recommendations
    pub async fn analyze_and_recommend(
        &self,
        execution_records: Vec<ExecutionRecord>,
        current_configs: HashMap<TaskPattern, serde_json::Value>,
    ) -> anyhow::Result<ConfigurationRecommendations> {
        let mut recommendations = Vec::new();
        let mut total_confidence = 0.0;
        let mut total_improvement = 0.0;
        
        // Group records by task pattern
        let mut pattern_groups: HashMap<TaskPattern, Vec<&ExecutionRecord>> = HashMap::new();
        for record in &execution_records {
            pattern_groups.entry(TaskPattern::CompilationErrors { error_groups: vec![] }).or_default().push(record);
        }
        
        // Analyze each pattern and generate recommendations
        for (pattern, records) in &pattern_groups {
            if records.len() >= 5 { // Minimum samples for reliable analysis
                let pattern_recs = self.analyze_pattern_configuration(&pattern, &records, &current_configs).await?;
                recommendations.extend(pattern_recs);
            }
        }
        
        // Calculate overall metrics
        if !recommendations.is_empty() {
            total_confidence = recommendations.iter().map(|r| r.confidence).sum::<f32>() / recommendations.len() as f32;
            total_improvement = recommendations.iter().map(|r| r.expected_impact.performance_improvement).sum::<f32>() / recommendations.len() as f32;
        }
        
        let config_recs = ConfigurationRecommendations {
            recommendations,
            overall_confidence: total_confidence,
            estimated_improvement: total_improvement,
            generated_at: Utc::now(),
        };
        
        // Store recommendations
        {
            let mut recs = self.config_recommendations.write().await;
            for (pattern, _) in &pattern_groups {
                recs.insert(pattern.clone(), config_recs.clone());
            }
        }
        
        Ok(config_recs)
    }
    
    /// Analyze configuration for a specific pattern
    async fn analyze_pattern_configuration(
        &self,
        pattern: &TaskPattern,
        records: &[&ExecutionRecord],
        current_configs: &HashMap<TaskPattern, serde_json::Value>,
    ) -> anyhow::Result<Vec<ConfigRecommendation>> {
        let mut recommendations = Vec::new();
        
        // Analyze worker count optimization
        if let Some(worker_count_rec) = self.analyze_worker_count_optimization(pattern, records, current_configs).await? {
            recommendations.push(worker_count_rec);
        }
        
        // Analyze timeout optimization
        if let Some(timeout_rec) = self.analyze_timeout_optimization(pattern, records, current_configs).await? {
            recommendations.push(timeout_rec);
        }
        
        // Analyze quality threshold optimization
        if let Some(quality_rec) = self.analyze_quality_threshold_optimization(pattern, records, current_configs).await? {
            recommendations.push(quality_rec);
        }
        
        // Analyze strategy optimization
        if let Some(strategy_rec) = self.analyze_strategy_optimization(pattern, records, current_configs).await? {
            recommendations.push(strategy_rec);
        }
        
        Ok(recommendations)
    }
    
    /// Analyze worker count optimization
    async fn analyze_worker_count_optimization(
        &self,
        pattern: &TaskPattern,
        records: &[&ExecutionRecord],
        current_configs: &HashMap<TaskPattern, serde_json::Value>,
    ) -> anyhow::Result<Option<ConfigRecommendation>> {
        let success_rate = records.iter().filter(|r| r.outcome.is_success()).count() as f32 / records.len() as f32;
        let avg_execution_time = records.iter().map(|r| r.metrics.execution_time_ms).sum::<u64>() as f32 / records.len() as f32;
        
        // Get current worker count from config
        let current_worker_count = current_configs
            .get(pattern)
            .and_then(|v| v.get("worker_count"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1) as usize;
        
        // Calculate optimal worker count based on performance
        let optimal_count = self.calculate_optimal_worker_count(success_rate, avg_execution_time, records.len());
        
        if optimal_count != current_worker_count {
            let improvement = self.calculate_worker_count_improvement(optimal_count, current_worker_count, success_rate);
            
            return Ok(Some(ConfigRecommendation {
                task_pattern: pattern.clone(),
                optimization_type: OptimizationType::WorkerCount,
                current_value: serde_json::Value::Number(current_worker_count.into()),
                recommended_value: serde_json::Value::Number(optimal_count.into()),
                expected_impact: ExpectedImpact {
                    performance_improvement: improvement,
                    quality_improvement: 0.0, // Worker count doesn't directly affect quality
                    resource_efficiency_improvement: if optimal_count < current_worker_count { 0.1 } else { -0.05 },
                    risk_level: if (optimal_count as i32 - current_worker_count as i32).abs() > 2 { RiskLevel::Medium } else { RiskLevel::Low },
                },
                confidence: 0.8,
                reasoning: format!("Optimal worker count based on success rate {:.2} and execution time {:.0}ms", success_rate, avg_execution_time),
            }));
        }
        
        Ok(None)
    }
    
    /// Analyze timeout optimization
    async fn analyze_timeout_optimization(
        &self,
        pattern: &TaskPattern,
        records: &[&ExecutionRecord],
        current_configs: &HashMap<TaskPattern, serde_json::Value>,
    ) -> anyhow::Result<Option<ConfigRecommendation>> {
        let timeout_failures = records.iter().filter(|r| !r.outcome.is_success() && r.metrics.execution_time_ms > 30000).count();
        let total_failures = records.iter().filter(|r| !r.outcome.is_success()).count();
        
        if timeout_failures as f32 / total_failures as f32 > 0.3 { // More than 30% of failures are timeouts
            let avg_execution_time = records.iter().map(|r| r.metrics.execution_time_ms).sum::<u64>() as f32 / records.len() as f32;
            let recommended_timeout = Duration::from_millis((avg_execution_time * 1.5) as u64);
            
            let current_timeout = current_configs
                .get(pattern)
                .and_then(|v| v.get("timeout_ms"))
                .and_then(|v| v.as_u64())
                .map(|ms| Duration::from_millis(ms))
                .unwrap_or(Duration::from_secs(30));
            
            if recommended_timeout != current_timeout {
                return Ok(Some(ConfigRecommendation {
                    task_pattern: pattern.clone(),
                    optimization_type: OptimizationType::Timeout,
                    current_value: serde_json::Value::Number(serde_json::Number::from(current_timeout.as_millis() as u64)),
                    recommended_value: serde_json::Value::Number(serde_json::Number::from(recommended_timeout.as_millis() as u64)),
                    expected_impact: ExpectedImpact {
                        performance_improvement: 0.1, // Reduce timeout failures
                        quality_improvement: 0.0,
                        resource_efficiency_improvement: 0.0,
                        risk_level: RiskLevel::Low,
                    },
                    confidence: 0.7,
                    reasoning: format!("Timeout optimization based on {} timeout failures out of {} total failures", timeout_failures, total_failures),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Analyze quality threshold optimization
    async fn analyze_quality_threshold_optimization(
        &self,
        pattern: &TaskPattern,
        records: &[&ExecutionRecord],
        current_configs: &HashMap<TaskPattern, serde_json::Value>,
    ) -> anyhow::Result<Option<ConfigRecommendation>> {
        let avg_quality = records.iter().map(|r| r.metrics.quality_score).sum::<f32>() / records.len() as f32;
        let quality_variance = records.iter()
            .map(|r| (r.metrics.quality_score - avg_quality).powi(2))
            .sum::<f32>() / records.len() as f32;
        
        // If quality is consistently high, we can raise the threshold
        if avg_quality > 0.8 && quality_variance < 0.1 {
            let recommended_threshold = (avg_quality * 0.9).max(0.7);
            
            let current_threshold = current_configs
                .get(pattern)
                .and_then(|v| v.get("quality_threshold"))
                .and_then(|v| v.as_f64())
                .unwrap_or(0.5) as f32;
            
            if recommended_threshold > current_threshold {
                return Ok(Some(ConfigRecommendation {
                    task_pattern: pattern.clone(),
                    optimization_type: OptimizationType::QualityThreshold,
                    current_value: serde_json::Value::Number(serde_json::Number::from_f64(current_threshold as f64).unwrap()),
                    recommended_value: serde_json::Value::Number(serde_json::Number::from_f64(recommended_threshold as f64).unwrap()),
                    expected_impact: ExpectedImpact {
                        performance_improvement: 0.0,
                        quality_improvement: 0.05, // Slightly higher quality requirements
                        resource_efficiency_improvement: 0.0,
                        risk_level: RiskLevel::Low,
                    },
                    confidence: 0.8,
                    reasoning: format!("Quality threshold optimization based on avg quality {:.2} and variance {:.2}", avg_quality, quality_variance),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Analyze strategy optimization
    async fn analyze_strategy_optimization(
        &self,
        pattern: &TaskPattern,
        records: &[&ExecutionRecord],
        current_configs: &HashMap<TaskPattern, serde_json::Value>,
    ) -> anyhow::Result<Option<ConfigRecommendation>> {
        // Group by worker specialty to find best performing specialty
        let mut specialty_performance: HashMap<WorkerSpecialty, (usize, f32)> = HashMap::new();
        
        for record in records {
            let entry = specialty_performance.entry(record.specialty.clone()).or_insert((0, 0.0));
            entry.0 += 1;
            entry.1 += record.metrics.quality_score;
        }
        
        // Find best performing specialty
        let best_specialty = specialty_performance
            .iter()
            .max_by(|a, b| (a.1.1 / a.1.0 as f32).partial_cmp(&(b.1.1 / b.1.0 as f32)).unwrap_or(std::cmp::Ordering::Equal));
        
        if let Some((best_specialty, _)) = best_specialty {
            let current_strategy = current_configs
                .get(pattern)
                .and_then(|v| v.get("strategy"))
                .and_then(|v| v.as_str())
                .unwrap_or("default");
            
            let recommended_strategy = format!("specialization_{:?}", best_specialty).to_lowercase();
            
            if recommended_strategy != current_strategy {
                return Ok(Some(ConfigRecommendation {
                    task_pattern: pattern.clone(),
                    optimization_type: OptimizationType::Strategy,
                    current_value: serde_json::Value::String(current_strategy.to_string()),
                    recommended_value: serde_json::Value::String(recommended_strategy),
                    expected_impact: ExpectedImpact {
                        performance_improvement: 0.15, // Strategy changes can have significant impact
                        quality_improvement: 0.1,
                        resource_efficiency_improvement: 0.05,
                        risk_level: RiskLevel::Medium,
                    },
                    confidence: 0.7,
                    reasoning: format!("Strategy optimization based on best performing specialty: {:?}", best_specialty),
                }));
            }
        }
        
        Ok(None)
    }
    
    /// Calculate optimal worker count
    fn calculate_optimal_worker_count(&self, success_rate: f32, avg_execution_time: f32, sample_count: usize) -> usize {
        // Simple heuristic based on success rate and execution time
        let base_count = if success_rate < 0.7 { 2 } else { 1 };
        let time_factor = if avg_execution_time > 10000.0 { 2 } else { 1 };
        let sample_factor = if sample_count > 20 { 1 } else { 2 };
        
        (base_count * time_factor * sample_factor).min(4) // Cap at 4 workers
    }
    
    /// Calculate expected improvement from worker count change
    fn calculate_worker_count_improvement(&self, new_count: usize, old_count: usize, success_rate: f32) -> f32 {
        let count_ratio = new_count as f32 / old_count as f32;
        let efficiency_gain = if count_ratio > 1.0 {
            // More workers: potential for parallelization
            (count_ratio - 1.0) * 0.3
        } else {
            // Fewer workers: better resource efficiency
            (1.0 - count_ratio) * 0.2
        };
        
        efficiency_gain * success_rate
    }
    
    /// Apply configuration recommendations
    pub async fn apply_recommendations(
        &self,
        recommendations: &[ConfigRecommendation],
    ) -> anyhow::Result<Vec<OptimizationEvent>> {
        let mut events = Vec::new();
        
        for rec in recommendations {
            let event = match rec.optimization_type {
                OptimizationType::WorkerCount => {
                    let old_count = rec.current_value.as_u64().unwrap_or(1) as usize;
                    let new_count = rec.recommended_value.as_u64().unwrap_or(1) as usize;
                    
                    OptimizationEvent::WorkerCountOptimized {
                        task_pattern: rec.task_pattern.clone(),
                        old_count,
                        new_count,
                        expected_improvement: rec.expected_impact.performance_improvement,
                    }
                }
                OptimizationType::Timeout => {
                    let old_timeout = Duration::from_millis(rec.current_value.as_u64().unwrap_or(30000));
                    let new_timeout = Duration::from_millis(rec.recommended_value.as_u64().unwrap_or(30000));
                    
                    OptimizationEvent::TimeoutOptimized {
                        task_pattern: rec.task_pattern.clone(),
                        old_timeout,
                        new_timeout,
                        expected_improvement: rec.expected_impact.performance_improvement,
                    }
                }
                OptimizationType::QualityThreshold => {
                    let old_threshold = rec.current_value.as_f64().unwrap_or(0.5) as f32;
                    let new_threshold = rec.recommended_value.as_f64().unwrap_or(0.5) as f32;
                    
                    OptimizationEvent::QualityThresholdOptimized {
                        task_pattern: rec.task_pattern.clone(),
                        old_threshold,
                        new_threshold,
                        expected_improvement: rec.expected_impact.quality_improvement,
                    }
                }
                OptimizationType::Strategy => {
                    let old_strategy = rec.current_value.as_str().unwrap_or("default").to_string();
                    let new_strategy = rec.recommended_value.as_str().unwrap_or("default").to_string();
                    
                    OptimizationEvent::StrategyOptimized {
                        task_pattern: rec.task_pattern.clone(),
                        old_strategy,
                        new_strategy,
                        expected_improvement: rec.expected_impact.performance_improvement,
                    }
                }
                _ => continue,
            };
            
            events.push(event);
        }
        
        // Store optimization events
        {
            let mut history = self.optimization_history.write().await;
            history.extend(events.clone());
        }
        
        Ok(events)
    }
    
    /// Get optimization history
    pub async fn get_optimization_history(&self) -> Vec<OptimizationEvent> {
        let history = self.optimization_history.read().await;
        history.clone()
    }
    
    /// Get recommendations for a specific pattern
    pub async fn get_recommendations(&self, pattern: &TaskPattern) -> Option<ConfigurationRecommendations> {
        let recs = self.config_recommendations.read().await;
        recs.get(pattern).cloned()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::metrics_collector::ExecutionOutcome;

    #[tokio::test]
    async fn test_configuration_optimizer() {
        let pattern_analyzer = Arc::new(PatternAnalyzer::new(3, 0.7));
        let optimizer = ConfigurationOptimizer::new(pattern_analyzer);
        
        let records = vec![
            ExecutionRecord {
                task_id: TaskId::new(),
                worker_id: WorkerId::new(),
                specialty: WorkerSpecialty::Compilation,
                subtask_id: crate::types::SubTaskId::new(),
                metrics: ExecutionMetrics {
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    execution_time_ms: 5000,
                    cpu_usage_percent: Some(50.0),
                    memory_usage_mb: Some(100.0),
                    files_modified: 5,
                    lines_changed: 50,
                    quality_score: 0.9,
                },
                outcome: ExecutionOutcome::Success,
                timestamp: Utc::now(),
                reward: Some(0.8),
                learning_mode: crate::learning::reward::LearningMode::Exploitation,
            },
        ];
        
        let current_configs = HashMap::new();
        let result = optimizer.analyze_and_recommend(records, current_configs).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_worker_count_optimization() {
        let pattern_analyzer = Arc::new(PatternAnalyzer::new(3, 0.7));
        let optimizer = ConfigurationOptimizer::new(pattern_analyzer);
        
        let pattern = TaskPattern::Compilation;
        let records = vec![
            &ExecutionRecord {
                task_id: TaskId::new(),
                worker_id: WorkerId::new(),
                specialty: WorkerSpecialty::Compilation,
                subtask_id: crate::types::SubTaskId::new(),
                metrics: ExecutionMetrics {
                    start_time: Utc::now(),
                    end_time: Utc::now(),
                    execution_time_ms: 5000,
                    cpu_usage_percent: Some(50.0),
                    memory_usage_mb: Some(100.0),
                    files_modified: 5,
                    lines_changed: 50,
                    quality_score: 0.9,
                },
                outcome: ExecutionOutcome::Success,
                timestamp: Utc::now(),
                reward: Some(0.8),
                learning_mode: crate::learning::reward::LearningMode::Exploitation,
            },
        ];
        
        let current_configs = HashMap::new();
        let result = optimizer.analyze_worker_count_optimization(&pattern, &records, &current_configs).await;
        assert!(result.is_ok());
    }
}
