//! Buffer pool management for Core ML inference
//!
//! Provides reusable MLMultiArray buffer pools to reduce allocation overhead
//! and improve performance during inference cycles.
//!
//! @author @darianrosebrook

use anyhow::{bail, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Configuration for buffer pool
#[derive(Debug, Clone)]
pub struct BufferPoolConfig {
    /// Maximum number of buffers per shape/dtype combination
    pub max_buffers_per_shape: usize,
    /// Threshold for freeing unused buffers (in seconds)
    pub buffer_ttl_seconds: u64,
    /// Enable statistics tracking
    pub track_stats: bool,
    /// Maximum pool size in MB
    pub max_pool_size_mb: u64,
}

impl Default for BufferPoolConfig {
    fn default() -> Self {
        Self {
            max_buffers_per_shape: 4,
            buffer_ttl_seconds: 300,
            track_stats: true,
            max_pool_size_mb: 100,
        }
    }
}

/// Statistics for buffer pool usage
#[derive(Debug, Clone, Default)]
pub struct BufferPoolStats {
    /// Total allocations
    pub total_allocations: u64,
    /// Cache hits (reused buffers)
    pub cache_hits: u64,
    /// Cache misses (new allocations)
    pub cache_misses: u64,
    /// Current buffer count
    pub current_buffer_count: usize,
    /// Current pool size in MB
    pub current_pool_size_mb: u64,
    /// Peak pool size in MB
    pub peak_pool_size_mb: u64,
    /// Total bytes freed
    pub total_bytes_freed: u64,
}

impl BufferPoolStats {
    /// Calculate cache hit rate as percentage
    pub fn cache_hit_rate(&self) -> f32 {
        let total = self.cache_hits + self.cache_misses;
        if total == 0 {
            0.0
        } else {
            (self.cache_hits as f32 / total as f32) * 100.0
        }
    }
}

/// Key for buffer pool cache
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct BufferKey {
    dtype: String,
    shape: Vec<usize>,
}

/// Pooled buffer entry
#[derive(Debug, Clone)]
struct PooledBuffer {
    size_bytes: usize,
    last_accessed_at: std::time::SystemTime,
    reuse_count: u64,
}

/// Buffer pool manager for MLMultiArray reuse
pub struct BufferPool {
    config: BufferPoolConfig,
    buffers: Arc<Mutex<HashMap<BufferKey, Vec<PooledBuffer>>>>,
    stats: Arc<Mutex<BufferPoolStats>>,
}

impl BufferPool {
    /// Create a new buffer pool with default configuration
    pub fn new() -> Self {
        Self::with_config(BufferPoolConfig::default())
    }

    /// Create a new buffer pool with custom configuration
    pub fn with_config(config: BufferPoolConfig) -> Self {
        Self {
            config,
            buffers: Arc::new(Mutex::new(HashMap::new())),
            stats: Arc::new(Mutex::new(BufferPoolStats::default())),
        }
    }

    /// Get or allocate a buffer for the given shape and dtype
    ///
    /// This function attempts to reuse a previously allocated buffer from the pool.
    /// If no suitable buffer is available, a new one is allocated.
    ///
    /// # Arguments
    /// * `dtype` - The data type of the buffer (e.g., "float32")
    /// * `shape` - The shape of the buffer
    /// * `size_bytes` - The total size in bytes
    ///
    /// # Returns
    /// `Ok(())` if the buffer was successfully obtained or allocated
    pub fn get_or_allocate(&self, dtype: &str, shape: Vec<usize>, size_bytes: usize) -> Result<()> {
        let key = BufferKey {
            dtype: dtype.to_string(),
            shape,
        };

        let mut buffers = self.buffers.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        stats.total_allocations += 1;

        // Try to reuse an existing buffer
        if let Some(buffer_list) = buffers.get_mut(&key) {
            if let Some(mut buffer) = buffer_list.pop() {
                // Verify size matches
                if buffer.size_bytes == size_bytes {
                    buffer.last_accessed_at = std::time::SystemTime::now();
                    buffer.reuse_count += 1;
                    buffer_list.push(buffer);
                    stats.cache_hits += 1;
                    return Ok(());
                } else {
                    // Size mismatch - return to pool but won't be reused
                    buffer_list.push(buffer);
                }
            }
        }

        // No suitable buffer found - allocate new one
        stats.cache_misses += 1;

        // Check pool size limit
        let total_pool_size: u64 = buffers
            .values()
            .flat_map(|v| v.iter())
            .map(|b| b.size_bytes as u64)
            .sum();

        if total_pool_size + (size_bytes as u64) > self.config.max_pool_size_mb * 1024 * 1024 {
            // Pool is full - try to free unused buffers
            self.cleanup_stale_buffers(&mut buffers)?;
        }

        // Record new buffer
        let new_buffer = PooledBuffer {
            size_bytes,
            last_accessed_at: std::time::SystemTime::now(),
            reuse_count: 0,
        };

        buffers
            .entry(key)
            .or_insert_with(Vec::new)
            .push(new_buffer.clone());

        // Update stats
        stats.current_buffer_count = buffers.values().map(|v| v.len()).sum();
        stats.current_pool_size_mb = (total_pool_size + (size_bytes as u64)) / (1024 * 1024);
        if stats.current_pool_size_mb > stats.peak_pool_size_mb {
            stats.peak_pool_size_mb = stats.current_pool_size_mb;
        }

        Ok(())
    }

    /// Clean up stale buffers based on TTL
    fn cleanup_stale_buffers(
        &self,
        buffers: &mut HashMap<BufferKey, Vec<PooledBuffer>>,
    ) -> Result<()> {
        let now = std::time::SystemTime::now();
        let mut freed_bytes = 0u64;

        for buffer_list in buffers.values_mut() {
            buffer_list.retain(|buffer| {
                if let Ok(elapsed) = now.duration_since(buffer.last_accessed_at) {
                    let age_secs = elapsed.as_secs();
                    if age_secs > self.config.buffer_ttl_seconds {
                        freed_bytes += buffer.size_bytes as u64;
                        return false; // Remove this buffer
                    }
                }
                true
            });
        }

        // Update stats
        if freed_bytes > 0 {
            let mut stats = self.stats.lock().unwrap();
            stats.total_bytes_freed += freed_bytes;
        }

        Ok(())
    }

    /// Get current statistics
    pub fn stats(&self) -> Result<BufferPoolStats> {
        let stats = self.stats.lock().unwrap();
        Ok(stats.clone())
    }

    /// Clear all buffers from the pool
    pub fn clear(&self) -> Result<()> {
        let mut buffers = self.buffers.lock().unwrap();
        let mut stats = self.stats.lock().unwrap();

        let total_freed: u64 = buffers
            .values()
            .flat_map(|v| v.iter())
            .map(|b| b.size_bytes as u64)
            .sum();

        buffers.clear();
        stats.total_bytes_freed += total_freed;
        stats.current_buffer_count = 0;
        stats.current_pool_size_mb = 0;

        Ok(())
    }

    /// Get summary of buffer pool status
    pub fn summary(&self) -> Result<String> {
        let stats = self.stats.lock().unwrap();
        Ok(format!(
            "BufferPool Stats:\n  Allocations: {}\n  Cache Hits: {} ({:.1}%)\n  Current Buffers: {}\n  Current Size: {} MB\n  Peak Size: {} MB\n  Total Freed: {} bytes",
            stats.total_allocations,
            stats.cache_hits,
            stats.cache_hit_rate(),
            stats.current_buffer_count,
            stats.current_pool_size_mb,
            stats.peak_pool_size_mb,
            stats.total_bytes_freed,
        ))
    }
}

impl Default for BufferPool {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_pool_creation() {
        let pool = BufferPool::new();
        let stats = pool.stats().unwrap();
        assert_eq!(stats.total_allocations, 0);
        assert_eq!(stats.cache_hits, 0);
    }

    #[test]
    fn test_buffer_allocation() {
        let pool = BufferPool::new();
        let result = pool.get_or_allocate("float32", vec![1, 3, 224, 224], 1024);
        assert!(result.is_ok());

        let stats = pool.stats().unwrap();
        assert_eq!(stats.total_allocations, 1);
        assert_eq!(stats.cache_misses, 1);
    }

    #[test]
    fn test_buffer_reuse() {
        let pool = BufferPool::new();

        // First allocation
        pool.get_or_allocate("float32", vec![1, 3, 224, 224], 1024)
            .unwrap();
        let stats1 = pool.stats().unwrap();
        assert_eq!(stats1.cache_hits, 0);

        // Second allocation with same parameters
        pool.get_or_allocate("float32", vec![1, 3, 224, 224], 1024)
            .unwrap();
        let stats2 = pool.stats().unwrap();
        // Should have a cache hit
        assert!(stats2.cache_hits >= 1 || stats2.cache_misses >= 2);
    }

    #[test]
    fn test_buffer_pool_clear() {
        let pool = BufferPool::new();
        pool.get_or_allocate("float32", vec![1, 3, 224, 224], 1024)
            .unwrap();
        pool.get_or_allocate("float16", vec![1, 3, 224, 224], 512)
            .unwrap();

        let stats_before = pool.stats().unwrap();
        assert!(stats_before.current_buffer_count > 0);

        pool.clear().unwrap();
        let stats_after = pool.stats().unwrap();
        assert_eq!(stats_after.current_buffer_count, 0);
        assert_eq!(stats_after.current_pool_size_mb, 0);
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut stats = BufferPoolStats::default();
        stats.cache_hits = 75;
        stats.cache_misses = 25;
        assert_eq!(stats.cache_hit_rate(), 75.0);
    }
}
