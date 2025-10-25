//! Main verdict store implementation with caching and persistence
//!
//! This module provides the main VerdictStore implementation that combines
//! caching, persistence, and monitoring capabilities.

use super::cache::{VerdictCache, CacheManager};
use super::storage::{VerdictStorage, MemoryVerdictStorage, DatabaseVerdictStorage};
use super::types::*;
use crate::types::*;
use agent_agency_database::DatabaseClient;
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use dashmap::DashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Persistent storage for council verdicts and decisions
#[derive(Debug)]
pub struct VerdictStore {
    /// Cache manager for fast access
    cache_manager: Arc<CacheManager>,
    /// Persistent storage backend
    storage: Arc<dyn VerdictStorage>,
    /// Store statistics
    stats: Arc<RwLock<VerdictStoreStats>>,
    /// Start time for uptime tracking
    start_time: DateTime<Utc>,
}

impl VerdictStore {
    /// Create a new verdict store with in-memory storage
    pub fn new() -> Self {
        let cache_config = CacheConfig::default();
        let cache_manager = Arc::new(CacheManager::new(cache_config));

        Self {
            cache_manager,
            storage: Arc::new(MemoryVerdictStorage::default()),
            stats: Arc::new(RwLock::new(VerdictStoreStats::default())),
            start_time: Utc::now(),
        }
    }

    /// Create a verdict store with custom cache configuration
    pub fn with_cache_config(cache_config: CacheConfig) -> Self {
        let cache_manager = Arc::new(CacheManager::new(cache_config));

        Self {
            cache_manager,
            storage: Arc::new(MemoryVerdictStorage::default()),
            stats: Arc::new(RwLock::new(VerdictStoreStats::default())),
            start_time: Utc::now(),
        }
    }

    /// Create a verdict store with database storage
    pub fn with_database_storage(storage: DatabaseVerdictStorage, cache_config: CacheConfig) -> Self {
        let cache_manager = Arc::new(CacheManager::new(cache_config));

        Self {
            cache_manager,
            storage: Arc::new(storage),
            stats: Arc::new(RwLock::new(VerdictStoreStats::default())),
            start_time: Utc::now(),
        }
    }

    /// Store a verdict with caching
    pub async fn store_verdict(&self, record: VerdictRecord) -> Result<()> {
        let verdict_id = record.verdict_id;
        let start_time = std::time::Instant::now();

        // Store in persistent storage first
        self.storage.store_verdict(&record).await
            .context("Failed to store verdict in persistent storage")?;

        // Cache the verdict if caching is enabled
        if let Err(e) = self.cache_manager.verdict_cache().put(record).await {
            warn!("Failed to cache verdict {}: {}", verdict_id, e);
        }

        // Update statistics
        let duration = start_time.elapsed();
        self.update_stats(duration.as_millis() as u64, true).await;

        debug!("Stored verdict {} in {}ms", verdict_id, duration.as_millis());
        Ok(())
    }

    /// Load a verdict with cache-first strategy
    pub async fn load_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>> {
        let start_time = std::time::Instant::now();

        // Try cache first
        if let Some(record) = self.cache_manager.verdict_cache().get(verdict_id).await? {
            let duration = start_time.elapsed();
            self.update_stats(duration.as_millis() as u64, true).await;
            debug!("Cache hit for verdict {} in {}ms", verdict_id, duration.as_millis());
            return Ok(Some(record));
        }

        // Cache miss - load from storage
        debug!("Cache miss for verdict {}, loading from storage", verdict_id);
        match self.storage.load_verdict(verdict_id).await {
            Ok(Some(record)) => {
                // Cache the loaded record for future requests
                if let Err(e) = self.cache_manager.verdict_cache().put(record.clone()).await {
                    warn!("Failed to cache loaded verdict {}: {}", verdict_id, e);
                }

                let duration = start_time.elapsed();
                self.update_stats(duration.as_millis() as u64, true).await;
                debug!("Loaded verdict {} from storage in {}ms", verdict_id, duration.as_millis());
                Ok(Some(record))
            }
            Ok(None) => {
                let duration = start_time.elapsed();
                self.update_stats(duration.as_millis() as u64, false).await;
                Ok(None)
            }
            Err(e) => {
                let duration = start_time.elapsed();
                self.update_stats(duration.as_millis() as u64, false).await;
                Err(e)
            }
        }
    }

    /// Load verdicts for a specific task
    pub async fn load_verdicts_by_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>> {
        let start_time = std::time::Instant::now();

        // For task-based queries, we need to go to storage since cache is keyed by verdict ID
        let records = self.storage.load_verdicts_by_task(task_id).await?;

        // Cache the loaded records
        for record in &records {
            if let Err(e) = self.cache_manager.verdict_cache().put(record.clone()).await {
                warn!("Failed to cache verdict {}: {}", record.verdict_id, e);
            }
        }

        let duration = start_time.elapsed();
        self.update_stats(duration.as_millis() as u64, true).await;

        debug!("Loaded {} verdicts for task {} in {}ms", records.len(), task_id, duration.as_millis());
        Ok(records)
    }

    /// Load verdicts within a time range
    pub async fn load_verdicts_by_time_range(
        &self,
        start: DateTime<Utc>,
        end: DateTime<Utc>,
    ) -> Result<Vec<VerdictRecord>> {
        let start_time = std::time::Instant::now();

        let records = self.storage.load_verdicts_by_time_range(start, end).await?;

        // Cache loaded records (with size limit consideration)
        let mut cached_count = 0;
        for record in &records {
            if cached_count >= 50 { // Limit caching for bulk loads
                break;
            }
            if let Err(e) = self.cache_manager.verdict_cache().put(record.clone()).await {
                warn!("Failed to cache verdict {}: {}", record.verdict_id, e);
            }
            cached_count += 1;
        }

        let duration = start_time.elapsed();
        self.update_stats(duration.as_millis() as u64, true).await;

        debug!("Loaded {} verdicts for time range in {}ms", records.len(), duration.as_millis());
        Ok(records)
    }

    /// Delete a verdict from both cache and storage
    pub async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Remove from cache first
        let _ = self.cache_manager.verdict_cache().remove(verdict_id).await;

        // Delete from storage
        self.storage.delete_verdict(verdict_id).await?;

        let duration = start_time.elapsed();
        self.update_stats(duration.as_millis() as u64, true).await;

        debug!("Deleted verdict {} in {}ms", verdict_id, duration.as_millis());
        Ok(())
    }

    /// Get comprehensive store statistics
    pub async fn get_store_stats(&self) -> Result<VerdictStoreStats> {
        let storage_stats = self.storage.get_storage_stats().await?;
        let cache_stats = self.cache_manager.verdict_cache().stats().await?;

        let uptime_seconds = Utc::now().signed_duration_since(self.start_time).num_seconds() as u64;

        let mut stats = self.stats.read().await.clone();
        stats.storage_stats = storage_stats;
        stats.cache_stats = cache_stats;
        stats.uptime_seconds = uptime_seconds;

        Ok(stats)
    }

    /// Perform health check on the verdict store
    pub async fn health_check(&self) -> HealthCheck {
        let start_time = std::time::Instant::now();
        let mut error_message = None;

        let status = match self.storage.get_storage_stats().await {
            Ok(_) => {
                // Test cache access
                match self.cache_manager.verdict_cache().stats().await {
                    Ok(_) => StorageHealth::Healthy,
                    Err(e) => {
                        error_message = Some(format!("Cache health check failed: {}", e));
                        StorageHealth::Degraded {
                            reason: "Cache access failed".to_string(),
                        }
                    }
                }
            }
            Err(e) => {
                error_message = Some(format!("Storage health check failed: {}", e));
                StorageHealth::Unhealthy {
                    reason: "Storage access failed".to_string(),
                    critical: true,
                }
            }
        };

        let response_time_ms = start_time.elapsed().as_millis() as u64;

        HealthCheck {
            status,
            last_check: Utc::now(),
            response_time_ms,
            error_message,
        }
    }

    /// Clear cache
    pub async fn clear_cache(&self) -> Result<()> {
        self.cache_manager.verdict_cache().clear().await
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> Result<CacheStats> {
        self.cache_manager.verdict_cache().stats().await
    }

    /// Update internal statistics
    async fn update_stats(&self, operation_time_ms: u64, success: bool) {
        let mut stats = self.stats.write().await;
        stats.operations_count += 1;

        // Additional statistics could be tracked here
        // (e.g., operation latency percentiles, error rates, etc.)
    }

    /// Query verdicts with flexible parameters
    pub async fn query_verdicts(&self, query: VerdictQuery) -> Result<VerdictQueryResult> {
        let start_time = std::time::Instant::now();

        let records = match (query.task_id, query.time_range) {
            (Some(task_id), None) => self.load_verdicts_by_task(task_id).await?,
            (None, Some((start, end))) => self.load_verdicts_by_time_range(start, end).await?,
            (Some(task_id), Some((start, end))) => {
                // Load by task and filter by time
                let mut records = self.load_verdicts_by_task(task_id).await?;
                records.retain(|r| r.created_at >= start && r.created_at <= end);
                records
            }
            (None, None) => {
                return Err(anyhow::anyhow!("Query must specify either task_id or time_range"));
            }
        };

        // Apply pagination
        let total_count = records.len() as u64;
        let offset = query.offset.unwrap_or(0);
        let limit = query.limit.unwrap_or(100);

        let paginated_records = records
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();

        let has_more = (offset + paginated_records.len() as usize) < total_count as usize;

        let query_duration_ms = start_time.elapsed().as_millis() as u64;

        Ok(VerdictQueryResult {
            records: paginated_records,
            total_count,
            has_more,
            query_duration_ms,
        })
    }
}

impl Default for VerdictStoreStats {
    fn default() -> Self {
        Self {
            storage_stats: StorageStats {
                total_verdicts: 0,
                total_debates: 0,
                storage_size_bytes: 0,
                oldest_verdict: None,
                newest_verdict: None,
            },
            cache_stats: CacheStats::default(),
            uptime_seconds: 0,
            operations_count: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verdict_store_basic_operations() {
        let store = VerdictStore::new();

        let record = VerdictRecord {
            verdict_id: VerdictId(Uuid::new_v4()),
            consensus_result: ConsensusResult::Approve {
                confidence: 0.9,
                judge_count: 3,
            },
            debate_session: None,
            created_at: Utc::now(),
            accessed_at: Utc::now(),
            access_count: 0,
            storage_location: None,
        };

        // Store verdict
        store.store_verdict(record.clone()).await.unwrap();

        // Load verdict
        let loaded = store.load_verdict(record.verdict_id).await.unwrap();
        assert!(loaded.is_some());
        assert_eq!(loaded.unwrap().verdict_id, record.verdict_id);
    }

    #[tokio::test]
    async fn test_verdict_store_health_check() {
        let store = VerdictStore::new();
        let health = store.health_check().await;

        match health.status {
            StorageHealth::Healthy => {} // Expected for in-memory storage
            _ => panic!("Expected healthy status for new store"),
        }
    }
}
