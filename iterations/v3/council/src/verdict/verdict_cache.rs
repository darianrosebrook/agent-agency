//! Caching layer for verdict storage
//!
//! This module provides caching functionality for verdict records,
//! including cache management, eviction policies, and performance monitoring.

use super::types::*;
use crate::council_types::VerdictId;
use anyhow::Result;
use chrono::{DateTime, Utc, Duration};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, warn};

/// Verdict cache for fast access to frequently used records
#[derive(Debug)]
pub struct VerdictCache {
    /// Internal cache storage using DashMap for concurrent access
    cache: Arc<DashMap<VerdictId, CachedVerdict>>,
    /// Cache configuration
    config: CacheConfig,
    /// Cache statistics
    stats: Arc<RwLock<CacheStats>>,
}

/// Cached verdict with metadata
#[derive(Debug, Clone)]
struct CachedVerdict {
    record: VerdictRecord,
    inserted_at: DateTime<Utc>,
    last_accessed: DateTime<Utc>,
    access_count: u64,
}

impl VerdictCache {
    /// Create a new verdict cache
    pub fn new(config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            config,
            stats: Arc::new(RwLock::new(CacheStats::default())),
        }
    }

    /// Store a verdict in the cache
    pub async fn put(&self, record: VerdictRecord) -> Result<()> {
        let verdict_id = record.verdict_id;
        let cached_verdict = CachedVerdict {
            record,
            inserted_at: Utc::now(),
            last_accessed: Utc::now(),
            access_count: 1,
        };

        // Check cache size limit
        if self.cache.len() >= self.config.max_cached_verdicts {
            self.evict_oldest()?;
        }

        self.cache.insert(verdict_id, cached_verdict);

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_entries = self.cache.len();

        Ok(())
    }

    /// Retrieve a verdict from the cache
    pub async fn get(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>> {
        if let Some(mut cached) = self.cache.get_mut(&verdict_id) {
            // Update access statistics
            cached.last_accessed = Utc::now();
            cached.access_count += 1;

            // Update global stats
            let mut stats = self.stats.write().await;
            stats.last_access = Some(Utc::now());

            // Check if cache entry has expired
            if self.is_expired(&cached) {
                // Remove expired entry
                drop(cached);
                self.cache.remove(&verdict_id);
                return Ok(None);
            }

            return Ok(Some(cached.record.clone()));
        }

        Ok(None)
    }

    /// Remove a verdict from the cache
    pub async fn remove(&self, verdict_id: VerdictId) -> Result<bool> {
        let removed = self.cache.remove(&verdict_id).is_some();

        if removed {
            let mut stats = self.stats.write().await;
            stats.total_entries = self.cache.len();
        }

        Ok(removed)
    }

    /// Clear all entries from the cache
    pub async fn clear(&self) -> Result<()> {
        self.cache.clear();

        let mut stats = self.stats.write().await;
        *stats = CacheStats::default();

        Ok(())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> Result<CacheStats> {
        let mut stats = self.stats.read().await.clone();

        // Update current metrics
        stats.total_entries = self.cache.len();

        // Calculate hit/miss rates if we have access data
        // This would need more sophisticated tracking in a real implementation
        stats.hit_rate = 0.8; // Placeholder
        stats.miss_rate = 0.2; // Placeholder

        Ok(stats)
    }

    /// Check if a cached verdict has expired
    fn is_expired(&self, cached: &CachedVerdict) -> bool {
        let ttl_duration = Duration::seconds(self.config.cache_ttl_seconds as i64);
        Utc::now().signed_duration_since(cached.last_accessed) > ttl_duration
    }

    /// Evict the oldest cache entry
    fn evict_oldest(&self) -> Result<()> {
        let mut oldest_key = None;
        let mut oldest_time = Utc::now();

        // Find the oldest entry
        for entry in self.cache.iter() {
            if entry.value().last_accessed < oldest_time {
                oldest_time = entry.value().last_accessed;
                oldest_key = Some(*entry.key());
            }
        }

        if let Some(key) = oldest_key {
            self.cache.remove(&key);

            // Update eviction stats
            let mut stats = self.stats.write().await;
            stats.eviction_count += 1;
        }

        Ok(())
    }

    /// Clean up expired entries
    pub async fn cleanup_expired(&self) -> Result<usize> {
        let mut expired_keys = Vec::new();
        let now = Utc::now();
        let ttl_duration = Duration::seconds(self.config.cache_ttl_seconds as i64);

        // Find expired entries
        for entry in self.cache.iter() {
            if now.signed_duration_since(entry.value().last_accessed) > ttl_duration {
                expired_keys.push(*entry.key());
            }
        }

        // Remove expired entries
        for key in expired_keys {
            self.cache.remove(&key);
        }

        let removed_count = expired_keys.len();

        if removed_count > 0 {
            let mut stats = self.stats.write().await;
            stats.eviction_count += removed_count as u64;
            stats.total_entries = self.cache.len();
        }

        Ok(removed_count)
    }

    /// Get cache size information
    pub async fn size_info(&self) -> CacheSizeInfo {
        let total_entries = self.cache.len();
        let estimated_size_bytes = total_entries * std::mem::size_of::<CachedVerdict>();

        CacheSizeInfo {
            total_entries,
            estimated_size_bytes: estimated_size_bytes as u64,
            max_entries: self.config.max_cached_verdicts,
        }
    }

    /// Check if cache contains a verdict
    pub async fn contains(&self, verdict_id: VerdictId) -> bool {
        if let Some(cached) = self.cache.get(&verdict_id) {
            !self.is_expired(&cached)
        } else {
            false
        }
    }
}

/// Cache size information
#[derive(Debug, Clone)]
pub struct CacheSizeInfo {
    pub total_entries: usize,
    pub estimated_size_bytes: u64,
    pub max_entries: usize,
}

/// Cache manager for coordinating multiple caches
pub struct CacheManager {
    verdict_cache: VerdictCache,
    cleanup_interval_seconds: u64,
}

impl CacheManager {
    /// Create a new cache manager
    pub fn new(config: CacheConfig) -> Self {
        Self {
            verdict_cache: VerdictCache::new(config),
            cleanup_interval_seconds: 300, // 5 minutes
        }
    }

    /// Get the verdict cache
    pub fn verdict_cache(&self) -> &VerdictCache {
        &self.verdict_cache
    }

    /// Start background cleanup task
    pub async fn start_cleanup_task(self: Arc<Self>) {
        let cleanup_interval = std::time::Duration::from_secs(self.cleanup_interval_seconds);

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(cleanup_interval);

            loop {
                interval.tick().await;

                if let Err(e) = self.verdict_cache.cleanup_expired().await {
                    warn!("Failed to cleanup expired cache entries: {}", e);
                } else {
                    debug!("Cache cleanup completed successfully");
                }
            }
        });
    }

    /// Get combined cache statistics
    pub async fn combined_stats(&self) -> Result<CombinedCacheStats> {
        let verdict_stats = self.verdict_cache.stats().await?;

        Ok(CombinedCacheStats {
            verdict_cache: verdict_stats,
            total_caches: 1,
            uptime_seconds: 0, // Would need to track this
        })
    }
}

/// Combined statistics for all caches
#[derive(Debug, Clone)]
pub struct CombinedCacheStats {
    pub verdict_cache: CacheStats,
    pub total_caches: usize,
    pub uptime_seconds: u64,
}

impl Default for CacheStats {
    fn default() -> Self {
        Self {
            total_entries: 0,
            hit_rate: 0.0,
            miss_rate: 0.0,
            eviction_count: 0,
            last_access: None,
        }
    }
}
