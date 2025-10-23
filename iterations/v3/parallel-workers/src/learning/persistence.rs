//! Database persistence layer for learning system

use crate::types::{WorkerId, TaskPattern};
use crate::learning::metrics_collector::{ExecutionRecord, WorkerPerformanceProfile};
use crate::learning::pattern_analyzer::{SuccessPattern, FailurePattern, OptimalConfig};
use crate::learning::config_optimizer::{ConfigurationRecommendations, OptimizationEvent};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use chrono::{DateTime, Utc};
use std::time::Duration;

/// Learning persistence trait
#[async_trait::async_trait]
pub trait LearningPersistence: Send + Sync {
    /// Store execution records
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> anyhow::Result<()>;
    
    /// Get execution records for a task pattern
    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> anyhow::Result<Vec<ExecutionRecord>>;
    
    /// Store worker performance profiles
    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> anyhow::Result<()>;
    
    /// Get worker performance profile
    async fn get_worker_profile(&self, worker_id: &WorkerId) -> anyhow::Result<Option<WorkerPerformanceProfile>>;
    
    /// Store success patterns
    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> anyhow::Result<()>;
    
    /// Get success patterns
    async fn get_success_patterns(&self) -> anyhow::Result<Vec<SuccessPattern>>;
    
    /// Store failure patterns
    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> anyhow::Result<()>;
    
    /// Get failure patterns
    async fn get_failure_patterns(&self) -> anyhow::Result<Vec<FailurePattern>>;
    
    /// Store optimal configurations
    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> anyhow::Result<()>;
    
    /// Get optimal configurations
    async fn get_optimal_configs(&self) -> anyhow::Result<Vec<OptimalConfig>>;
    
    /// Store configuration recommendations
    async fn store_config_recommendations(&self, recommendations: HashMap<TaskPattern, ConfigurationRecommendations>) -> anyhow::Result<()>;
    
    /// Get configuration recommendations
    async fn get_config_recommendations(&self, pattern: &TaskPattern) -> anyhow::Result<Option<ConfigurationRecommendations>>;
    
    /// Store optimization events
    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> anyhow::Result<()>;
    
    /// Get optimization events
    async fn get_optimization_events(&self, limit: Option<usize>) -> anyhow::Result<Vec<OptimizationEvent>>;
}

/// In-memory learning persistence (for testing)
pub struct InMemoryLearningPersistence {
    execution_records: Arc<RwLock<HashMap<TaskPattern, Vec<ExecutionRecord>>>>,
    worker_profiles: Arc<RwLock<HashMap<WorkerId, WorkerPerformanceProfile>>>,
    success_patterns: Arc<RwLock<Vec<SuccessPattern>>>,
    failure_patterns: Arc<RwLock<Vec<FailurePattern>>>,
    optimal_configs: Arc<RwLock<Vec<OptimalConfig>>>,
    config_recommendations: Arc<RwLock<HashMap<TaskPattern, ConfigurationRecommendations>>>,
    optimization_events: Arc<RwLock<Vec<OptimizationEvent>>>,
}

impl InMemoryLearningPersistence {
    /// Create a new in-memory persistence layer
    pub fn new() -> Self {
        Self {
            execution_records: Arc::new(RwLock::new(HashMap::new())),
            worker_profiles: Arc::new(RwLock::new(HashMap::new())),
            success_patterns: Arc::new(RwLock::new(Vec::new())),
            failure_patterns: Arc::new(RwLock::new(Vec::new())),
            optimal_configs: Arc::new(RwLock::new(Vec::new())),
            config_recommendations: Arc::new(RwLock::new(HashMap::new())),
            optimization_events: Arc::new(RwLock::new(Vec::new())),
        }
    }
}

#[async_trait::async_trait]
impl LearningPersistence for InMemoryLearningPersistence {
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> anyhow::Result<()> {
        let mut storage = self.execution_records.write().await;
        
        for record in records {
            let pattern = TaskPattern::CompilationErrors { error_groups: vec![] }; // TODO: Extract actual pattern from record
            storage.entry(pattern).or_default().push(record);
        }
        
        Ok(())
    }
    
    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> anyhow::Result<Vec<ExecutionRecord>> {
        let storage = self.execution_records.read().await;
        
        if let Some(records) = storage.get(pattern) {
            let mut result = records.clone();
            if let Some(limit) = limit {
                result.truncate(limit);
            }
            Ok(result)
        } else {
            Ok(Vec::new())
        }
    }
    
    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> anyhow::Result<()> {
        let mut storage = self.worker_profiles.write().await;
        storage.extend(profiles);
        Ok(())
    }
    
    async fn get_worker_profile(&self, worker_id: &WorkerId) -> anyhow::Result<Option<WorkerPerformanceProfile>> {
        let storage = self.worker_profiles.read().await;
        Ok(storage.get(worker_id).cloned())
    }
    
    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> anyhow::Result<()> {
        let mut storage = self.success_patterns.write().await;
        storage.extend(patterns);
        Ok(())
    }
    
    async fn get_success_patterns(&self) -> anyhow::Result<Vec<SuccessPattern>> {
        let storage = self.success_patterns.read().await;
        Ok(storage.clone())
    }
    
    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> anyhow::Result<()> {
        let mut storage = self.failure_patterns.write().await;
        storage.extend(patterns);
        Ok(())
    }
    
    async fn get_failure_patterns(&self) -> anyhow::Result<Vec<FailurePattern>> {
        let storage = self.failure_patterns.read().await;
        Ok(storage.clone())
    }
    
    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> anyhow::Result<()> {
        let mut storage = self.optimal_configs.write().await;
        storage.extend(configs);
        Ok(())
    }
    
    async fn get_optimal_configs(&self) -> anyhow::Result<Vec<OptimalConfig>> {
        let storage = self.optimal_configs.read().await;
        Ok(storage.clone())
    }
    
    async fn store_config_recommendations(&self, recommendations: HashMap<TaskPattern, ConfigurationRecommendations>) -> anyhow::Result<()> {
        let mut storage = self.config_recommendations.write().await;
        for (pattern, recommendation) in recommendations {
            storage.insert(pattern, recommendation);
        }
        Ok(())
    }
    
    async fn get_config_recommendations(&self, pattern: &TaskPattern) -> anyhow::Result<Option<ConfigurationRecommendations>> {
        let storage = self.config_recommendations.read().await;
        Ok(storage.get(pattern).cloned())
    }
    
    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> anyhow::Result<()> {
        let mut storage = self.optimization_events.write().await;
        storage.extend(events);
        Ok(())
    }
    
    async fn get_optimization_events(&self, limit: Option<usize>) -> anyhow::Result<Vec<OptimizationEvent>> {
        let storage = self.optimization_events.read().await;
        let mut result = storage.clone();
        if let Some(limit) = limit {
            result.truncate(limit);
        }
        Ok(result)
    }
}

/// Batch buffer for efficient data storage
pub struct BatchBuffer<T> {
    buffer: Arc<RwLock<Vec<T>>>,
    max_batch_size: usize,
    flush_interval: Duration,
    last_flush: Arc<RwLock<DateTime<Utc>>>,
}

impl<T> BatchBuffer<T> {
    /// Create a new batch buffer
    pub fn new(max_batch_size: usize, flush_interval: Duration) -> Self {
        Self {
            buffer: Arc::new(RwLock::new(Vec::new())),
            max_batch_size,
            flush_interval,
            last_flush: Arc::new(RwLock::new(Utc::now())),
        }
    }
    
    /// Add item to buffer
    pub async fn add_item(&self, item: T) -> anyhow::Result<bool> {
        let mut buffer = self.buffer.write().await;
        buffer.push(item);
        
        // Check if buffer should be flushed
        let should_flush = buffer.len() >= self.max_batch_size || self.should_flush_by_time().await;
        Ok(should_flush)
    }
    
    /// Get and clear current batch
    pub async fn take_batch(&self) -> Vec<T> {
        let mut buffer = self.buffer.write().await;
        let batch = std::mem::take(&mut *buffer);
        *self.last_flush.write().await = Utc::now();
        batch
    }
    
    /// Check if buffer should be flushed by time
    async fn should_flush_by_time(&self) -> bool {
        let last_flush = *self.last_flush.read().await;
        Utc::now() - last_flush > chrono::Duration::from_std(self.flush_interval).unwrap()
    }
    
    /// Get current buffer size
    pub async fn buffer_size(&self) -> usize {
        let buffer = self.buffer.read().await;
        buffer.len()
    }
}

/// Database learning persistence (placeholder for real implementation)
pub struct DatabaseLearningPersistence {
    // In a real implementation, this would contain database connection pools, etc.
    // For now, we'll use the in-memory implementation as a fallback
    fallback: InMemoryLearningPersistence,
}

impl DatabaseLearningPersistence {
    /// Create a new database persistence layer
    pub fn new() -> Self {
        Self {
            fallback: InMemoryLearningPersistence::new(),
        }
    }
}

#[async_trait::async_trait]
impl LearningPersistence for DatabaseLearningPersistence {
    async fn store_execution_records(&self, records: Vec<ExecutionRecord>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        // For now, use fallback
        self.fallback.store_execution_records(records).await
    }
    
    async fn get_execution_records(&self, pattern: &TaskPattern, limit: Option<usize>) -> anyhow::Result<Vec<ExecutionRecord>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_execution_records(pattern, limit).await
    }
    
    async fn store_worker_profiles(&self, profiles: HashMap<WorkerId, WorkerPerformanceProfile>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        self.fallback.store_worker_profiles(profiles).await
    }
    
    async fn get_worker_profile(&self, worker_id: &WorkerId) -> anyhow::Result<Option<WorkerPerformanceProfile>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_worker_profile(worker_id).await
    }
    
    async fn store_success_patterns(&self, patterns: Vec<SuccessPattern>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        self.fallback.store_success_patterns(patterns).await
    }
    
    async fn get_success_patterns(&self) -> anyhow::Result<Vec<SuccessPattern>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_success_patterns().await
    }
    
    async fn store_failure_patterns(&self, patterns: Vec<FailurePattern>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        self.fallback.store_failure_patterns(patterns).await
    }
    
    async fn get_failure_patterns(&self) -> anyhow::Result<Vec<FailurePattern>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_failure_patterns().await
    }
    
    async fn store_optimal_configs(&self, configs: Vec<OptimalConfig>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        self.fallback.store_optimal_configs(configs).await
    }
    
    async fn get_optimal_configs(&self) -> anyhow::Result<Vec<OptimalConfig>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_optimal_configs().await
    }
    
    async fn store_config_recommendations(&self, recommendations: HashMap<TaskPattern, ConfigurationRecommendations>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        self.fallback.store_config_recommendations(recommendations).await
    }
    
    async fn get_config_recommendations(&self, pattern: &TaskPattern) -> anyhow::Result<Option<ConfigurationRecommendations>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_config_recommendations(pattern).await
    }
    
    async fn store_optimization_events(&self, events: Vec<OptimizationEvent>) -> anyhow::Result<()> {
        // TODO: Implement actual database storage
        self.fallback.store_optimization_events(events).await
    }
    
    async fn get_optimization_events(&self, limit: Option<usize>) -> anyhow::Result<Vec<OptimizationEvent>> {
        // TODO: Implement actual database retrieval
        self.fallback.get_optimization_events(limit).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::learning::metrics_collector::ExecutionOutcome;

    #[tokio::test]
    async fn test_in_memory_persistence() {
        let persistence = InMemoryLearningPersistence::new();
        
        let record = ExecutionRecord {
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
        };
        
        let result = persistence.store_execution_records(vec![record.clone()]).await;
        assert!(result.is_ok());
        
        let pattern = record.task_id.to_string().into();
        let retrieved = persistence.get_execution_records(&pattern, None).await;
        assert!(retrieved.is_ok());
        assert_eq!(retrieved.unwrap().len(), 1);
    }
    
    #[tokio::test]
    async fn test_batch_buffer() {
        let buffer = BatchBuffer::<String>::new(5, Duration::from_secs(60));
        
        // Add items to buffer
        for i in 0..3 {
            let should_flush = buffer.add_item(format!("item_{}", i)).await;
            assert!(should_flush.is_ok());
            assert!(!should_flush.unwrap()); // Should not flush yet
        }
        
        // Add more items to trigger flush
        for i in 3..6 {
            let should_flush = buffer.add_item(format!("item_{}", i)).await;
            assert!(should_flush.is_ok());
            if i == 5 {
                assert!(should_flush.unwrap()); // Should flush at 5 items
            }
        }
        
        let batch = buffer.take_batch().await;
        assert_eq!(batch.len(), 6);
    }
}
