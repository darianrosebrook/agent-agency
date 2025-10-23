//! Pattern recognition engine for learning optimal configurations

use crate::types::{WorkerSpecialty, TaskPattern};
use crate::learning::metrics_collector::ExecutionRecord;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Pattern cache for storing learned patterns
#[derive(Debug, Clone)]
pub struct PatternCache {
    success_patterns: HashMap<TaskPattern, SuccessPattern>,
    failure_patterns: HashMap<TaskPattern, FailurePattern>,
    optimal_configs: HashMap<TaskPattern, OptimalConfig>,
    last_updated: DateTime<Utc>,
}

impl Default for PatternCache {
    fn default() -> Self {
        Self {
            success_patterns: HashMap::new(),
            failure_patterns: HashMap::new(),
            optimal_configs: HashMap::new(),
            last_updated: Utc::now(),
        }
    }
}

/// Success pattern for a task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuccessPattern {
    pub task_pattern: TaskPattern,
    pub worker_specialty: WorkerSpecialty,
    pub avg_execution_time: Duration,
    pub avg_quality_score: f32,
    pub success_rate: f32,
    pub resource_efficiency: f32,
    pub sample_count: usize,
    pub last_observed: DateTime<Utc>,
}

/// Failure pattern for a task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailurePattern {
    pub task_pattern: TaskPattern,
    pub common_failure_modes: Vec<String>,
    pub failure_rate: f32,
    pub avg_failure_time: Duration,
    pub sample_count: usize,
    pub last_observed: DateTime<Utc>,
}

/// Optimal configuration for a task type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimalConfig {
    pub task_pattern: TaskPattern,
    pub recommended_worker_count: usize,
    pub recommended_timeout: Duration,
    pub recommended_quality_threshold: f32,
    pub confidence: f32,
    pub last_updated: DateTime<Utc>,
}

/// Pattern analyzer for learning from execution history
pub struct PatternAnalyzer {
    cache: Arc<RwLock<PatternCache>>,
    min_samples: usize,
    confidence_threshold: f32,
}

impl PatternAnalyzer {
    /// Create a new pattern analyzer
    pub fn new(min_samples: usize, confidence_threshold: f32) -> Self {
        Self {
            cache: Arc::new(RwLock::new(PatternCache::default())),
            min_samples,
            confidence_threshold,
        }
    }
    
    /// Analyze execution records to identify patterns
    pub async fn analyze_execution_records(&self, records: Vec<ExecutionRecord>) -> anyhow::Result<()> {
        let mut cache = self.cache.write().await;
        
        // Group records by task pattern
        let mut pattern_groups: HashMap<TaskPattern, Vec<&ExecutionRecord>> = HashMap::new();
        for record in &records {
            pattern_groups.entry(TaskPattern::CompilationErrors { error_groups: vec![] }).or_default().push(record);
        }
        
        // Analyze each pattern group
        for (pattern, group_records) in pattern_groups {
            if group_records.len() >= self.min_samples {
                self.analyze_pattern_group(&mut cache, pattern, group_records).await?;
            }
        }
        
        cache.last_updated = Utc::now();
        Ok(())
    }
    
    /// Analyze a specific pattern group
    async fn analyze_pattern_group(
        &self,
        cache: &mut PatternCache,
        pattern: TaskPattern,
        records: Vec<&ExecutionRecord>,
    ) -> anyhow::Result<()> {
        // Calculate success metrics
        let success_count = records.iter().filter(|r| r.outcome.is_success()).count();
        let success_rate = success_count as f32 / records.len() as f32;
        
        let avg_execution_time = records
            .iter()
            .map(|r| r.metrics.execution_time_ms)
            .sum::<u64>() as f32 / records.len() as f32;
        
        let avg_quality_score = records
            .iter()
            .map(|r| r.metrics.quality_score)
            .sum::<f32>() / records.len() as f32;
        
        // Group by worker specialty
        let mut specialty_groups: HashMap<WorkerSpecialty, Vec<&ExecutionRecord>> = HashMap::new();
        for record in &records {
            specialty_groups.entry(record.specialty.clone()).or_default().push(record);
        }
        
        // Find best performing specialty
        let mut best_specialty = None;
        let mut best_performance = 0.0;
        
        for (specialty, specialty_records) in specialty_groups {
            let specialty_success_rate = specialty_records.iter().filter(|r| r.outcome.is_success()).count() as f32 / specialty_records.len() as f32;
            let specialty_quality = specialty_records.iter().map(|r| r.metrics.quality_score).sum::<f32>() / specialty_records.len() as f32;
            let performance = specialty_success_rate * specialty_quality;
            
            if performance > best_performance {
                best_performance = performance;
                best_specialty = Some(specialty);
            }
        }
        
        // Update success pattern
        if let Some(specialty) = best_specialty {
            let success_pattern = SuccessPattern {
                task_pattern: pattern.clone(),
                worker_specialty: specialty,
                avg_execution_time: Duration::from_millis(avg_execution_time as u64),
                avg_quality_score: avg_quality_score,
                success_rate,
                resource_efficiency: self.calculate_resource_efficiency(&records),
                sample_count: records.len(),
                last_observed: Utc::now(),
            };
            
            cache.success_patterns.insert(pattern.clone(), success_pattern);
        }
        
        // Analyze failure patterns
        if success_rate < 0.8 {
            let failure_pattern = FailurePattern {
                task_pattern: pattern.clone(),
                common_failure_modes: self.identify_failure_modes(&records),
                failure_rate: 1.0 - success_rate,
                avg_failure_time: Duration::from_millis(avg_execution_time as u64),
                sample_count: records.len(),
                last_observed: Utc::now(),
            };
            
            cache.failure_patterns.insert(pattern, failure_pattern);
        }
        
        Ok(())
    }
    
    /// Calculate resource efficiency score
    fn calculate_resource_efficiency(&self, records: &[&ExecutionRecord]) -> f32 {
        let avg_cpu = records.iter().map(|r| r.metrics.cpu_usage_percent.unwrap_or(0.0)).sum::<f32>() / records.len() as f32;
        let avg_memory = records.iter().map(|r| r.metrics.memory_usage_mb.unwrap_or(0.0)).sum::<f32>() / records.len() as f32;
        
        // Higher efficiency = lower resource usage for same quality
        let quality_weight = 0.7;
        let resource_weight = 0.3;
        
        let avg_quality = records.iter().map(|r| r.metrics.quality_score).sum::<f32>() / records.len() as f32;
        let resource_score = 1.0 - ((avg_cpu + avg_memory) / 200.0).min(1.0);
        
        avg_quality * quality_weight + resource_score * resource_weight
    }
    
    /// Identify common failure modes
    fn identify_failure_modes(&self, records: &[&ExecutionRecord]) -> Vec<String> {
        let mut failure_modes = Vec::new();
        
        // Count different types of failures
        let mut timeout_count = 0;
        let mut quality_failures = 0;
        let mut resource_failures = 0;
        
        for record in records {
            if !record.outcome.is_success() {
                if record.metrics.execution_time_ms > 30000 {
                    timeout_count += 1;
                }
                if record.metrics.quality_score < 0.5 {
                    quality_failures += 1;
                }
                if record.metrics.cpu_usage_percent.unwrap_or(0.0) > 90.0 {
                    resource_failures += 1;
                }
            }
        }
        
        let total_failures = records.len() - records.iter().filter(|r| r.outcome.is_success()).count();
        
        if total_failures > 0 {
            if timeout_count as f32 / total_failures as f32 > 0.3 {
                failure_modes.push("Timeout".to_string());
            }
            if quality_failures as f32 / total_failures as f32 > 0.3 {
                failure_modes.push("Quality".to_string());
            }
            if resource_failures as f32 / total_failures as f32 > 0.3 {
                failure_modes.push("Resource".to_string());
            }
        }
        
        failure_modes
    }
    
    /// Get success pattern for a task pattern
    pub async fn get_success_pattern(&self, pattern: &TaskPattern) -> Option<SuccessPattern> {
        let cache = self.cache.read().await;
        cache.success_patterns.get(pattern).cloned()
    }
    
    /// Get failure pattern for a task pattern
    pub async fn get_failure_pattern(&self, pattern: &TaskPattern) -> Option<FailurePattern> {
        let cache = self.cache.read().await;
        cache.failure_patterns.get(pattern).cloned()
    }
    
    /// Get optimal configuration for a task pattern
    pub async fn get_optimal_config(&self, pattern: &TaskPattern) -> Option<OptimalConfig> {
        let cache = self.cache.read().await;
        cache.optimal_configs.get(pattern).cloned()
    }
    
    /// Update optimal configuration based on success patterns
    pub async fn update_optimal_config(&self, pattern: &TaskPattern) -> anyhow::Result<()> {
        let mut cache = self.cache.write().await;
        
        if let Some(success_pattern) = cache.success_patterns.get(pattern) {
            let optimal_config = OptimalConfig {
                task_pattern: pattern.clone(),
                recommended_worker_count: self.calculate_optimal_worker_count(success_pattern),
                recommended_timeout: success_pattern.avg_execution_time * 2, // 2x safety margin
                recommended_quality_threshold: success_pattern.avg_quality_score * 0.9, // 90% of average
                confidence: success_pattern.success_rate,
                last_updated: Utc::now(),
            };
            
            cache.optimal_configs.insert(pattern.clone(), optimal_config);
        }
        
        Ok(())
    }
    
    /// Calculate optimal worker count based on success pattern
    fn calculate_optimal_worker_count(&self, success_pattern: &SuccessPattern) -> usize {
        // Simple heuristic: more workers for complex tasks, fewer for simple ones
        let complexity_factor = match success_pattern.task_pattern {
            TaskPattern::CompilationErrors { .. } => 1,
            TaskPattern::RefactoringOperations { .. } => 2,
            TaskPattern::TestingGaps { .. } => 1,
            TaskPattern::DocumentationNeeds { .. } => 1,
        };
        
        // Adjust based on success rate and resource efficiency
        let efficiency_factor = if success_pattern.resource_efficiency > 0.8 { 1 } else { 2 };
        
        (complexity_factor * efficiency_factor).min(4) // Cap at 4 workers
    }
    
    /// Get all learned patterns
    pub async fn get_all_patterns(&self) -> (Vec<SuccessPattern>, Vec<FailurePattern>, Vec<OptimalConfig>) {
        let cache = self.cache.read().await;
        (
            cache.success_patterns.values().cloned().collect(),
            cache.failure_patterns.values().cloned().collect(),
            cache.optimal_configs.values().cloned().collect(),
        )
    }
    
    /// Clear old patterns
    pub async fn clear_old_patterns(&self, max_age: Duration) -> anyhow::Result<()> {
        let mut cache = self.cache.write().await;
        let cutoff = Utc::now() - chrono::Duration::from_std(max_age).unwrap();
        
        // Remove old success patterns
        cache.success_patterns.retain(|_, pattern| pattern.last_observed > cutoff);
        
        // Remove old failure patterns
        cache.failure_patterns.retain(|_, pattern| pattern.last_observed > cutoff);
        
        // Remove old optimal configs
        cache.optimal_configs.retain(|_, config| config.last_updated > cutoff);
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::metrics_collector::ExecutionOutcome;

    #[tokio::test]
    async fn test_pattern_analyzer() {
        let analyzer = PatternAnalyzer::new(3, 0.7);
        
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
        
        let result = analyzer.analyze_execution_records(records).await;
        assert!(result.is_ok());
    }
    
    #[tokio::test]
    async fn test_success_pattern_retrieval() {
        let analyzer = PatternAnalyzer::new(3, 0.7);
        
        // Test with empty cache
        let pattern = TaskPattern::Compilation;
        let result = analyzer.get_success_pattern(&pattern).await;
        assert!(result.is_none());
    }
    
    #[test]
    fn test_resource_efficiency_calculation() {
        let analyzer = PatternAnalyzer::new(3, 0.7);
        
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
        
        let efficiency = analyzer.calculate_resource_efficiency(&records);
        assert!(efficiency > 0.0);
        assert!(efficiency <= 1.0);
    }
}
