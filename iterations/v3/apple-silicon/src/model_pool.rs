//! Model instance pooling for concurrent inference
//!
//! Provides a pool of reusable Core ML model instances to support
//! concurrent inference requests with bounded resource usage.
//!
//! @author @darianrosebrook

use std::collections::VecDeque;
use std::sync::{Arc, Mutex, Condvar};
use std::time::Duration;
use anyhow::{Result, bail};

/// Configuration for model pool
#[derive(Debug, Clone)]
pub struct ModelPoolConfig {
    /// Maximum number of model instances to pool
    pub max_instances: usize,
    /// Timeout for acquiring a model from the pool
    pub acquire_timeout_ms: u64,
    /// Maximum number of concurrent inferences per instance
    pub max_concurrent_per_instance: usize,
}

impl Default for ModelPoolConfig {
    fn default() -> Self {
        Self {
            max_instances: 4,
            acquire_timeout_ms: 5000,
            max_concurrent_per_instance: 2,
        }
    }
}

/// Statistics for model pool usage
#[derive(Debug, Clone, Default)]
pub struct ModelPoolStats {
    /// Total acquire requests
    pub total_acquires: u64,
    /// Successful acquires
    pub successful_acquires: u64,
    /// Failed acquires (timeout)
    pub failed_acquires: u64,
    /// Current active models
    pub active_models: usize,
    /// Total inferences run
    pub total_inferences: u64,
    /// Average pool wait time in ms
    pub average_wait_time_ms: f64,
}

/// Model pool state
pub struct ModelPool {
    config: ModelPoolConfig,
    available: Arc<Mutex<VecDeque<usize>>>,
    condvar: Arc<Condvar>,
    stats: Arc<Mutex<ModelPoolStats>>,
}

impl ModelPool {
    /// Create a new model pool with default configuration
    pub fn new() -> Self {
        Self::with_config(ModelPoolConfig::default())
    }

    /// Create a new model pool with custom configuration
    pub fn with_config(config: ModelPoolConfig) -> Self {
        let mut available = VecDeque::new();
        for i in 0..config.max_instances {
            available.push_back(i);
        }

        let max_instances = config.max_instances;
        Self {
            config,
            available: Arc::new(Mutex::new(available)),
            condvar: Arc::new(Condvar::new()),
            stats: Arc::new(Mutex::new(ModelPoolStats {
                active_models: max_instances,
                ..Default::default()
            })),
        }
    }

    /// Acquire a model instance from the pool
    ///
    /// Blocks until a model is available or timeout occurs.
    ///
    /// # Returns
    /// `Ok(model_id)` if a model was acquired
    pub fn acquire(&self) -> Result<usize> {
        let mut stats = self.stats.lock().unwrap();
        stats.total_acquires += 1;

        let timeout = Duration::from_millis(self.config.acquire_timeout_ms);
        let mut available = self.available.lock().unwrap();

        let start = std::time::Instant::now();
        loop {
            if let Some(model_id) = available.pop_front() {
                let wait_time_ms = start.elapsed().as_millis() as f64;
                stats.successful_acquires += 1;
                stats.average_wait_time_ms =
                    (stats.average_wait_time_ms * 0.9) + (wait_time_ms * 0.1);
                drop(stats);
                drop(available);
                return Ok(model_id);
            }

            let result = self.condvar.wait_timeout(available, timeout).unwrap();
            available = result.0;

            if result.1.timed_out() {
                let mut stats = self.stats.lock().unwrap();
                stats.failed_acquires += 1;
                bail!("Model pool acquire timeout after {}ms", self.config.acquire_timeout_ms);
            }

            if start.elapsed() > timeout {
                let mut stats = self.stats.lock().unwrap();
                stats.failed_acquires += 1;
                bail!("Model pool acquire timeout");
            }
        }
    }

    /// Release a model instance back to the pool
    pub fn release(&self, model_id: usize) -> Result<()> {
        let mut available = self.available.lock().unwrap();
        if model_id >= self.config.max_instances {
            bail!("Invalid model ID: {}", model_id);
        }
        available.push_back(model_id);
        drop(available);
        self.condvar.notify_one();
        Ok(())
    }

    /// Record an inference operation
    pub fn record_inference(&self) {
        if let Ok(mut stats) = self.stats.lock() {
            stats.total_inferences += 1;
        }
    }

    /// Get current statistics
    pub fn stats(&self) -> Result<ModelPoolStats> {
        let stats = self.stats.lock().unwrap();
        Ok(stats.clone())
    }

    /// Get summary of model pool status
    pub fn summary(&self) -> Result<String> {
        let stats = self.stats.lock().unwrap();
        Ok(format!(
            "ModelPool Stats:\n  Total Acquires: {}\n  Successful: {}\n  Failed: {}\n  Active Models: {}\n  Total Inferences: {}\n  Avg Wait: {:.2}ms",
            stats.total_acquires,
            stats.successful_acquires,
            stats.failed_acquires,
            stats.active_models,
            stats.total_inferences,
            stats.average_wait_time_ms,
        ))
    }
}

impl Default for ModelPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_pool_creation() {
        let pool = ModelPool::new();
        let stats = pool.stats().unwrap();
        assert_eq!(stats.total_acquires, 0);
        assert_eq!(stats.active_models, 4);
    }

    #[test]
    fn test_model_acquire_and_release() {
        let pool = ModelPool::new();
        let model_id = pool.acquire().unwrap();
        assert!(model_id < 4);
        
        let stats = pool.stats().unwrap();
        assert_eq!(stats.successful_acquires, 1);

        pool.release(model_id).unwrap();
    }

    #[test]
    fn test_pool_exhaustion() {
        let config = ModelPoolConfig {
            max_instances: 2,
            acquire_timeout_ms: 100,
            max_concurrent_per_instance: 1,
        };
        let pool = ModelPool::with_config(config);

        let model1 = pool.acquire().unwrap();
        let model2 = pool.acquire().unwrap();

        // Pool is exhausted, next acquire should timeout
        let result = pool.acquire();
        assert!(result.is_err());

        pool.release(model1).unwrap();
        pool.release(model2).unwrap();
    }

    #[test]
    fn test_record_inference() {
        let pool = ModelPool::new();
        pool.record_inference();
        pool.record_inference();
        
        let stats = pool.stats().unwrap();
        assert_eq!(stats.total_inferences, 2);
    }
}
