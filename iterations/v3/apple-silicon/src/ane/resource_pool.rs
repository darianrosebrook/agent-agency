//! Resource pool for ANE memory and concurrency management
//!
//! This module provides admission control, memory accounting, and concurrency
//! limits using semaphores and atomic operations for thread-safe resource management.

use crate::ane::errors::{ANEError, Result};
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Resource admission permit
/// 
/// This struct holds a semaphore permit that must be kept alive for the
/// duration of the resource usage. When dropped, the permit is automatically
/// returned to the pool.
#[derive(Debug)]
pub struct Admission {
    _permit: tokio::sync::OwnedSemaphorePermit,
    mem_cost_mb: usize,
    pool: Arc<Pool>,
}

impl Admission {
    /// Get the memory cost of this admission
    pub fn memory_cost_mb(&self) -> usize {
        self.mem_cost_mb
    }
}

impl Drop for Admission {
    fn drop(&mut self) {
        // Automatically release memory when admission is dropped
        self.pool.release_mem(self.mem_cost_mb);
    }
}

/// Resource pool for managing ANE resources
/// 
/// Provides admission control for memory and concurrency limits using
/// semaphores and atomic memory accounting.
#[derive(Debug)]
pub struct Pool {
    /// Semaphore for concurrency control
    inner: Arc<tokio::sync::Semaphore>,
    /// Total memory pool size in MB
    mem_total_mb: usize,
    /// Currently used memory in MB (protected by mutex)
    mem_used_mb: Mutex<usize>,
    /// Pool configuration
    config: PoolConfig,
    /// Pool statistics
    stats: Mutex<PoolStats>,
}

/// Pool statistics for monitoring
#[derive(Debug, Clone)]
pub struct PoolStats {
    pub total_admissions: u64,
    pub active_admissions: u64,
    pub peak_memory_usage_mb: usize,
    pub total_memory_allocated_mb: u64,
    pub admission_failures: u64,
    pub last_admission_time: Option<Instant>,
}

impl Default for PoolStats {
    fn default() -> Self {
        Self {
            total_admissions: 0,
            active_admissions: 0,
            peak_memory_usage_mb: 0,
            total_memory_allocated_mb: 0,
            admission_failures: 0,
            last_admission_time: None,
        }
    }
}

impl Pool {
    /// Create a new resource pool
    /// 
    /// # Arguments
    /// * `max_concurrent` - Maximum number of concurrent operations
    /// * `mem_total_mb` - Total memory pool size in MB
    pub fn new(max_concurrent: usize, mem_total_mb: usize) -> Self {
        Self {
            inner: Arc::new(tokio::sync::Semaphore::new(max_concurrent)),
            mem_total_mb,
            mem_used_mb: Mutex::new(0),
            config: PoolConfig {
                max_concurrent,
                mem_total_mb,
            },
            stats: Mutex::new(PoolStats::default()),
        }
    }

    /// Request admission to the resource pool
    /// 
    /// # Arguments
    /// * `mem_cost_mb` - Memory cost in MB for this operation
    /// 
    /// # Returns
    /// * `Ok(Admission)` - Admission permit if resources are available
    /// * `Err(ANEError::ResourceLimit)` - If resources are not available
    pub async fn admit(&self, mem_cost_mb: usize) -> Result<Admission> {
        // Check memory availability first (fast path)
        {
            let mut used = self.mem_used_mb.lock();
            if *used + mem_cost_mb > self.mem_total_mb {
                let mut stats = self.stats.lock();
                stats.admission_failures += 1;
                return Err(ANEError::ResourceLimit(
                    format!("Insufficient memory: {} MB requested, {} MB available", 
                           mem_cost_mb, self.mem_total_mb - *used)
                ));
            }
        }

        // Acquire semaphore permit (this may block)
        let permit = self.inner.clone()
            .acquire_owned()
            .await
            .map_err(|_| ANEError::Internal("Semaphore closed"))?;

        // Double-check memory availability after acquiring permit
        {
            let mut used = self.mem_used_mb.lock();
            if *used + mem_cost_mb > self.mem_total_mb {
                // Release the permit we just acquired
                drop(permit);
                let mut stats = self.stats.lock();
                stats.admission_failures += 1;
                return Err(ANEError::ResourceLimit(
                    format!("Insufficient memory: {} MB requested, {} MB available", 
                           mem_cost_mb, self.mem_total_mb - *used)
                ));
            }

            // Reserve memory
            *used += mem_cost_mb;
            
            // Update statistics
            let mut stats = self.stats.lock();
            stats.total_admissions += 1;
            stats.active_admissions += 1;
            stats.total_memory_allocated_mb += mem_cost_mb as u64;
            stats.last_admission_time = Some(Instant::now());
            
            if *used > stats.peak_memory_usage_mb {
                stats.peak_memory_usage_mb = *used;
            }
        }

        Ok(Admission {
            _permit: permit,
            mem_cost_mb,
            pool: Arc::new(Pool {
                inner: self.inner.clone(),
                mem_total_mb: self.mem_total_mb,
                mem_used_mb: Mutex::new(*self.mem_used_mb.lock()),
                config: self.config().clone(),
                stats: Mutex::new(self.stats.lock().clone()),
            }),
        })
    }

    /// Release memory (called automatically when Admission is dropped)
    fn release_mem(&self, mem_cost_mb: usize) {
        let mut used = self.mem_used_mb.lock();
        *used = used.saturating_sub(mem_cost_mb);
        
        let mut stats = self.stats.lock();
        stats.active_admissions = stats.active_admissions.saturating_sub(1);
    }

    /// Get current pool statistics
    pub fn stats(&self) -> PoolStats {
        let stats = self.stats.lock();
        let used = *self.mem_used_mb.lock();
        
        PoolStats {
            total_admissions: stats.total_admissions,
            active_admissions: stats.active_admissions,
            peak_memory_usage_mb: stats.peak_memory_usage_mb.max(used),
            total_memory_allocated_mb: stats.total_memory_allocated_mb,
            admission_failures: stats.admission_failures,
            last_admission_time: stats.last_admission_time,
        }
    }

    /// Get current memory usage
    pub fn memory_usage_mb(&self) -> usize {
        *self.mem_used_mb.lock()
    }

    /// Get available memory
    pub fn available_memory_mb(&self) -> usize {
        self.mem_total_mb - self.memory_usage_mb()
    }

    /// Get memory utilization percentage
    pub fn memory_utilization_percent(&self) -> f32 {
        let used = self.memory_usage_mb();
        (used as f32 / self.mem_total_mb as f32) * 100.0
    }

    /// Check if pool is under memory pressure
    pub fn is_under_pressure(&self, threshold_percent: f32) -> bool {
        self.memory_utilization_percent() > threshold_percent
    }

    /// Get pool configuration
    pub fn config(&self) -> PoolConfig {
        PoolConfig {
            max_concurrent: self.inner.available_permits(),
            mem_total_mb: self.mem_total_mb,
        }
    }
}

/// Pool configuration
#[derive(Debug, Clone)]
pub struct PoolConfig {
    pub max_concurrent: usize,
    pub mem_total_mb: usize,
}

/// Resource pool builder for configuration
#[derive(Debug, Clone)]
pub struct PoolBuilder {
    max_concurrent: Option<usize>,
    mem_total_mb: Option<usize>,
}

impl PoolBuilder {
    /// Create a new pool builder
    pub fn new() -> Self {
        Self {
            max_concurrent: None,
            mem_total_mb: None,
        }
    }

    /// Set maximum concurrent operations
    pub fn max_concurrent(mut self, max: usize) -> Self {
        self.max_concurrent = Some(max);
        self
    }

    /// Set total memory pool size in MB
    pub fn memory_total_mb(mut self, mb: usize) -> Self {
        self.mem_total_mb = Some(mb);
        self
    }

    /// Build the resource pool
    pub fn build(self) -> Result<Pool> {
        let max_concurrent = self.max_concurrent
            .ok_or_else(|| ANEError::ConfigurationError("max_concurrent not set".to_string()))?;
        let mem_total_mb = self.mem_total_mb
            .ok_or_else(|| ANEError::ConfigurationError("mem_total_mb not set".to_string()))?;

        if max_concurrent == 0 {
            return Err(ANEError::ConfigurationError("max_concurrent must be > 0".to_string()));
        }
        if mem_total_mb == 0 {
            return Err(ANEError::ConfigurationError("mem_total_mb must be > 0".to_string()));
        }

        Ok(Pool::new(max_concurrent, mem_total_mb))
    }
}

impl Default for PoolBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::time::timeout;

    #[tokio::test]
    async fn test_pool_creation() {
        let pool = Pool::new(4, 1024);
        assert_eq!(pool.memory_usage_mb(), 0);
        assert_eq!(pool.available_memory_mb(), 1024);
        assert_eq!(pool.memory_utilization_percent(), 0.0);
    }

    #[tokio::test]
    async fn test_admission_success() {
        let pool = Pool::new(2, 512);
        
        let admission = pool.admit(256).await.unwrap();
        assert_eq!(admission.memory_cost_mb(), 256);
        assert_eq!(pool.memory_usage_mb(), 256);
        assert_eq!(pool.available_memory_mb(), 256);
        
        // Admission should be automatically released when dropped
        drop(admission);
        assert_eq!(pool.memory_usage_mb(), 0);
        assert_eq!(pool.available_memory_mb(), 512);
    }

    #[tokio::test]
    async fn test_admission_failure() {
        let pool = Pool::new(2, 512);
        
        // First admission should succeed
        let _admission1 = pool.admit(400).await.unwrap();
        
        // Second admission should fail due to insufficient memory
        let result = pool.admit(200).await;
        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), ANEError::ResourceLimit(_)));
    }

    #[tokio::test]
    async fn test_concurrency_limit() {
        let pool = Pool::new(2, 1024);
        
        // First two admissions should succeed
        let _admission1 = pool.admit(100).await.unwrap();
        let _admission2 = pool.admit(100).await.unwrap();
        
        // Third admission should block (we'll timeout to avoid hanging)
        let result = timeout(Duration::from_millis(100), pool.admit(100)).await;
        assert!(result.is_err()); // Should timeout
    }

    #[tokio::test]
    async fn test_pool_statistics() {
        let pool = Pool::new(4, 1024);
        
        let stats_before = pool.stats();
        assert_eq!(stats_before.total_admissions, 0);
        assert_eq!(stats_before.active_admissions, 0);
        
        let _admission = pool.admit(256).await.unwrap();
        let stats_after = pool.stats();
        assert_eq!(stats_after.total_admissions, 1);
        assert_eq!(stats_after.active_admissions, 1);
        assert_eq!(stats_after.total_memory_allocated_mb, 256);
        
        drop(_admission);
        let stats_final = pool.stats();
        assert_eq!(stats_final.active_admissions, 0);
    }

    #[tokio::test]
    async fn test_memory_pressure() {
        let pool = Pool::new(4, 1000);
        
        assert!(!pool.is_under_pressure(50.0));
        
        let _admission = pool.admit(600).await.unwrap();
        assert!(pool.is_under_pressure(50.0));
        assert!(!pool.is_under_pressure(70.0));
    }

    #[tokio::test]
    async fn test_pool_builder() {
        let pool = PoolBuilder::new()
            .max_concurrent(8)
            .memory_total_mb(2048)
            .build()
            .unwrap();
            
        assert_eq!(pool.mem_total_mb, 2048);
    }

    #[tokio::test]
    async fn test_pool_builder_validation() {
        // Test missing max_concurrent
        let result = PoolBuilder::new()
            .memory_total_mb(1024)
            .build();
        assert!(result.is_err());
        
        // Test missing memory_total_mb
        let result = PoolBuilder::new()
            .max_concurrent(4)
            .build();
        assert!(result.is_err());
        
        // Test zero max_concurrent
        let result = PoolBuilder::new()
            .max_concurrent(0)
            .memory_total_mb(1024)
            .build();
        assert!(result.is_err());
        
        // Test zero memory_total_mb
        let result = PoolBuilder::new()
            .max_concurrent(4)
            .memory_total_mb(0)
            .build();
        assert!(result.is_err());
    }
}
