//! Performance data collection layer for parallel worker learning

use crate::types::{TaskId, SubTaskId, WorkerId, WorkerSpecialty, ExecutionMetrics};
use crate::learning::reward::{RewardWeights, Baseline, LearningMode, compute_reward_with_mode};
use dashmap::DashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Execution record for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub task_id: TaskId,
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub subtask_id: SubTaskId,
    pub metrics: ExecutionMetrics,
    pub outcome: ExecutionOutcome,
    pub timestamp: DateTime<Utc>,
    pub learning_mode: LearningMode,
}

/// Execution outcome classification
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ExecutionOutcome {
    Success,
    Failure,
    Timeout,
    Cancelled,
}

impl ExecutionOutcome {
    /// Check if execution was successful
    pub fn is_success(&self) -> bool {
        matches!(self, ExecutionOutcome::Success)
    }
}

/// Worker performance profile
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerPerformanceProfile {
    pub worker_id: WorkerId,
    pub specialty: WorkerSpecialty,
    pub total_executions: usize,
    pub success_rate: f32,
    pub average_execution_time: Duration,
    pub average_quality_score: f32,
    pub resource_efficiency: ResourceEfficiencyScore,
    pub specialization_strength: std::collections::HashMap<String, f32>,
    pub last_updated: DateTime<Utc>,
}

/// Resource efficiency score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceEfficiencyScore {
    pub cpu_efficiency: f32,
    pub memory_efficiency: f32,
    pub overall_efficiency: f32,
}

/// Parallel worker metrics collector
pub struct ParallelWorkerMetricsCollector {
    execution_history: DashMap<TaskId, Vec<ExecutionRecord>>,
    worker_profiles: DashMap<WorkerId, WorkerPerformanceProfile>,
    pattern_cache: Arc<RwLock<PatternCache>>,
    reward_weights: RewardWeights,
    baseline: Baseline,
}

/// Pattern cache for optimization
#[derive(Debug, Clone)]
pub struct PatternCache {
    pub successful_patterns: std::collections::HashMap<String, Vec<SuccessPattern>>,
    pub failure_patterns: std::collections::HashMap<String, Vec<FailurePattern>>,
    pub optimal_configurations: std::collections::HashMap<String, OptimalConfig>,
}

/// Success pattern
#[derive(Debug, Clone)]
pub struct SuccessPattern {
    pub task_pattern: String,
    pub worker_specialty: WorkerSpecialty,
    pub decomposition_strategy: String,
    pub average_speedup: f32,
    pub success_rate: f32,
    pub sample_size: usize,
}

/// Failure pattern
#[derive(Debug, Clone)]
pub struct FailurePattern {
    pub task_pattern: String,
    pub failure_mode: String,
    pub frequency: usize,
    pub common_causes: Vec<String>,
}

/// Optimal configuration
#[derive(Debug, Clone)]
pub struct OptimalConfig {
    pub task_pattern: String,
    pub recommended_workers: Vec<WorkerSpecialty>,
    pub optimal_subtask_count: usize,
    pub timeout_multiplier: f32,
    pub quality_thresholds: std::collections::HashMap<String, f32>,
    pub confidence: f32,
}

impl ParallelWorkerMetricsCollector {
    /// Create a new metrics collector
    pub fn new(reward_weights: RewardWeights, baseline: Baseline) -> Self {
        Self {
            execution_history: DashMap::new(),
            worker_profiles: DashMap::new(),
            pattern_cache: Arc::new(RwLock::new(PatternCache {
                successful_patterns: std::collections::HashMap::new(),
                failure_patterns: std::collections::HashMap::new(),
                optimal_configurations: std::collections::HashMap::new(),
            })),
            reward_weights,
            baseline,
        }
    }
    
    /// Record execution completion
    pub fn record_execution(&self, record: ExecutionRecord) {
        // Store in execution history
        self.execution_history
            .entry(record.task_id.clone())
            .or_insert_with(Vec::new)
            .push(record.clone());
        
        // Update worker performance profile
        self.update_worker_profile(&record);
        
        // Trigger pattern analysis if threshold reached
        if self.should_analyze_patterns() {
            tokio::spawn({
                let cache = self.pattern_cache.clone();
                let history = self.execution_history.clone();
                async move {
                    Self::analyze_patterns_async(cache, history).await;
                }
            });
        }
    }
    
    /// Get worker performance profile
    pub fn get_worker_profile(&self, worker_id: &WorkerId) -> Option<WorkerPerformanceProfile> {
        self.worker_profiles.get(worker_id).map(|p| p.clone())
    }
    
    /// Get all workers with specialty
    pub fn get_workers_by_specialty(&self, specialty: &WorkerSpecialty) -> Vec<WorkerPerformanceProfile> {
        self.worker_profiles
            .iter()
            .filter(|entry| &entry.specialty == specialty)
            .map(|entry| entry.clone())
            .collect()
    }
    
    /// Get execution history for a task
    pub fn get_task_history(&self, task_id: &TaskId) -> Option<Vec<ExecutionRecord>> {
        self.execution_history.get(task_id).map(|h| h.clone())
    }
    
    /// Calculate reward for execution
    pub fn calculate_reward(&self, record: &ExecutionRecord) -> Option<f64> {
        let rework_within_24h = self.check_rework_within_24h(&record.task_id);
        
        compute_reward_with_mode(
            &record.metrics,
            &self.reward_weights,
            &self.baseline,
            rework_within_24h,
            record.learning_mode,
        )
    }
    
    /// Update worker performance profile
    fn update_worker_profile(&self, record: &ExecutionRecord) {
        let mut profile = self.worker_profiles
            .entry(record.worker_id.clone())
            .or_insert_with(|| WorkerPerformanceProfile {
                worker_id: record.worker_id.clone(),
                specialty: record.specialty.clone(),
                total_executions: 0,
                success_rate: 0.0,
                average_execution_time: Duration::from_secs(0),
                average_quality_score: 0.0,
                resource_efficiency: ResourceEfficiencyScore {
                    cpu_efficiency: 0.0,
                    memory_efficiency: 0.0,
                    overall_efficiency: 0.0,
                },
                specialization_strength: std::collections::HashMap::new(),
                last_updated: Utc::now(),
            });
        
        // Update execution count
        profile.total_executions += 1;
        
        // Update success rate
        let success_count = self.count_successful_executions(&record.worker_id);
        profile.success_rate = success_count as f32 / profile.total_executions as f32;
        
        // Update average execution time
        let total_time = self.calculate_total_execution_time(&record.worker_id);
        profile.average_execution_time = Duration::from_millis(
            (total_time / profile.total_executions as f64) as u64
        );
        
        // Update average quality score
        let total_quality = self.calculate_total_quality_score(&record.worker_id);
        profile.average_quality_score = total_quality / profile.total_executions as f32;
        
        // Update resource efficiency
        profile.resource_efficiency = self.calculate_resource_efficiency(&record.worker_id);
        
        // Update specialization strength
        self.update_specialization_strength(&mut profile);
        
        profile.last_updated = Utc::now();
    }
    
    /// Check if rework occurred within 24 hours
    fn check_rework_within_24h(&self, task_id: &TaskId) -> bool {
        // Simple implementation - in production, you'd check database
        // for rework events within 24 hours
        false
    }
    
    /// Count successful executions for worker
    fn count_successful_executions(&self, worker_id: &WorkerId) -> usize {
        self.execution_history
            .iter()
            .flat_map(|entry| entry.value().clone())
            .filter(|record| record.worker_id == *worker_id && record.outcome.is_success())
            .count()
    }
    
    /// Calculate total execution time for worker
    fn calculate_total_execution_time(&self, worker_id: &WorkerId) -> f64 {
        self.execution_history
            .iter()
            .flat_map(|entry| entry.value().clone())
            .filter(|record| record.worker_id == *worker_id)
            .map(|record| record.metrics.execution_time_ms as f64)
            .sum()
    }
    
    /// Calculate total quality score for worker
    fn calculate_total_quality_score(&self, worker_id: &WorkerId) -> f32 {
        let scores: Vec<f32> = self.execution_history
            .iter()
            .flat_map(|entry| entry.value().clone())
            .filter(|record| record.worker_id == *worker_id)
            .map(|record| record.metrics.quality_score)
            .collect();
        
        if scores.is_empty() {
            0.0
        } else {
            scores.iter().sum::<f32>() / scores.len() as f32
        }
    }
    
    /// Calculate resource efficiency for worker
    fn calculate_resource_efficiency(&self, worker_id: &WorkerId) -> ResourceEfficiencyScore {
        let executions: Vec<_> = self.execution_history
            .iter()
            .flat_map(|entry| entry.value().clone())
            .filter(|record| record.worker_id == *worker_id)
            .collect();
        
        if executions.is_empty() {
            return ResourceEfficiencyScore {
                cpu_efficiency: 0.0,
                memory_efficiency: 0.0,
                overall_efficiency: 0.0,
            };
        }
        
        let avg_cpu = executions.iter()
            .map(|r| r.metrics.cpu_usage_percent.unwrap_or(0.0))
            .sum::<f32>() / executions.len() as f32;
        
        let avg_memory = executions.iter()
            .map(|r| r.metrics.memory_usage_mb.unwrap_or(0.0))
            .sum::<f32>() / executions.len() as f32;
        
        // Efficiency is inverse of resource usage (lower usage = higher efficiency)
        let cpu_efficiency = (100.0 - avg_cpu) / 100.0;
        let memory_efficiency = (1000.0 - avg_memory) / 1000.0; // Assuming 1GB baseline
        
        ResourceEfficiencyScore {
            cpu_efficiency: cpu_efficiency.max(0.0).min(1.0),
            memory_efficiency: memory_efficiency.max(0.0).min(1.0),
            overall_efficiency: (cpu_efficiency + memory_efficiency) / 2.0,
        }
    }
    
    /// Update specialization strength
    fn update_specialization_strength(&self, profile: &mut WorkerPerformanceProfile) {
        // Calculate strength for different task patterns
        let patterns = self.get_task_patterns_for_worker(&profile.worker_id);
        
        for (pattern, success_rate) in patterns {
            profile.specialization_strength.insert(pattern, success_rate);
        }
    }
    
    /// Get task patterns for worker
    fn get_task_patterns_for_worker(&self, worker_id: &WorkerId) -> std::collections::HashMap<String, f32> {
        let mut patterns = std::collections::HashMap::new();
        
        // Simple pattern extraction based on task descriptions
        for entry in self.execution_history.iter() {
            for record in entry.value() {
                if record.worker_id == *worker_id {
                    let pattern = self.extract_pattern_from_task(&record.task_id);
                    let success = record.outcome.is_success();
                    
                    let entry = patterns.entry(pattern).or_insert((0, 0));
                    entry.1 += 1; // total
                    if success {
                        entry.0 += 1; // successful
                    }
                }
            }
        }
        
        patterns.into_iter()
            .map(|(pattern, (success, total))| (pattern, success as f32 / total as f32))
            .collect()
    }
    
    /// Extract pattern from task
    fn extract_pattern_from_task(&self, _task_id: &TaskId) -> String {
        // Simple pattern extraction - in production, you'd analyze task content
        "general".to_string()
    }
    
    /// Check if pattern analysis should be triggered
    fn should_analyze_patterns(&self) -> bool {
        self.execution_history.len() % 100 == 0 // Analyze every 100 executions
    }
    
    /// Analyze patterns asynchronously
    async fn analyze_patterns_async(
        cache: Arc<RwLock<PatternCache>>,
        history: DashMap<TaskId, Vec<ExecutionRecord>>,
    ) {
        // Analyze execution history to identify patterns
        let mut successful_patterns = std::collections::HashMap::new();
        let mut failure_patterns = std::collections::HashMap::new();
        
        for entry in history.iter() {
            for record in entry.value() {
                let pattern = "general".to_string(); // Simplified
                
                if record.outcome.is_success() {
                    successful_patterns.entry(pattern.clone()).or_insert_with(Vec::new).push(
                        SuccessPattern {
                            task_pattern: pattern.clone(),
                            worker_specialty: record.specialty.clone(),
                            decomposition_strategy: "default".to_string(),
                            average_speedup: 1.0,
                            success_rate: 1.0,
                            sample_size: 1,
                        }
                    );
                } else {
                    failure_patterns.entry(pattern.clone()).or_insert_with(Vec::new).push(
                        FailurePattern {
                            task_pattern: pattern,
                            failure_mode: "unknown".to_string(),
                            frequency: 1,
                            common_causes: vec!["unknown".to_string()],
                        }
                    );
                }
            }
        }
        
        // Update cache
        let mut cache_guard = cache.write().await;
        cache_guard.successful_patterns = successful_patterns;
        cache_guard.failure_patterns = failure_patterns;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ExecutionMetrics;
    use chrono::Utc;

    fn create_test_metrics() -> ExecutionMetrics {
        ExecutionMetrics {
            start_time: Utc::now(),
            end_time: Utc::now(),
            execution_time_ms: 1000,
            cpu_usage_percent: Some(50.0),
            memory_usage_mb: Some(100.0),
            files_modified: 1,
            lines_changed: 10,
            quality_score: 0.8,
            tokens: Some(500),
        }
    }

    #[test]
    fn test_execution_record_creation() {
        let record = ExecutionRecord {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            subtask_id: SubTaskId::new(),
            metrics: create_test_metrics(),
            outcome: ExecutionOutcome::Success,
            timestamp: Utc::now(),
            learning_mode: LearningMode::Learn,
        };
        
        assert!(record.outcome.is_success());
    }
    
    #[test]
    fn test_metrics_collector() {
        let weights = RewardWeights::default();
        let baseline = Baseline::default();
        let collector = ParallelWorkerMetricsCollector::new(weights, baseline);
        
        let record = ExecutionRecord {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            subtask_id: SubTaskId::new(),
            metrics: create_test_metrics(),
            outcome: ExecutionOutcome::Success,
            timestamp: Utc::now(),
            learning_mode: LearningMode::Learn,
        };
        
        collector.record_execution(record.clone());
        
        // Should have worker profile
        let profile = collector.get_worker_profile(&record.worker_id);
        assert!(profile.is_some());
        
        let profile = profile.unwrap();
        assert_eq!(profile.total_executions, 1);
        assert_eq!(profile.success_rate, 1.0);
    }
    
    #[test]
    fn test_reward_calculation() {
        let weights = RewardWeights::default();
        let baseline = Baseline::default();
        let collector = ParallelWorkerMetricsCollector::new(weights, baseline);
        
        let record = ExecutionRecord {
            task_id: TaskId::new(),
            worker_id: WorkerId::new(),
            specialty: WorkerSpecialty::Compilation,
            subtask_id: SubTaskId::new(),
            metrics: create_test_metrics(),
            outcome: ExecutionOutcome::Success,
            timestamp: Utc::now(),
            learning_mode: LearningMode::Learn,
        };
        
        let reward = collector.calculate_reward(&record);
        assert!(reward.is_some());
        assert!(reward.unwrap() > 0.0);
    }
}
