//! Verdict Storage and Management System
//!
//! Provides persistent storage and retrieval of council verdicts, consensus results,
//! and debate sessions for audit trails and performance analysis.

use crate::database::DatabaseClient;
use crate::types::*;
use anyhow::{Context, Result};
use chrono;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use serde_json;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

/// Persistent storage for council verdicts and decisions
#[derive(Debug)]
pub struct VerdictStore {
    /// In-memory cache of recent verdicts for fast access
    cache: Arc<DashMap<VerdictId, VerdictRecord>>,
    /// Persistent storage backend (database)
    storage: Arc<dyn VerdictStorage>,
    /// Cache configuration
    cache_config: CacheConfig,
}

#[derive(Debug, Clone)]
pub struct CacheConfig {
    pub max_cached_verdicts: usize,
    pub cache_ttl_seconds: u64,
    pub enable_persistence: bool,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cached_verdicts: 1000,
            cache_ttl_seconds: 3600, // 1 hour
            enable_persistence: true,
        }
    }
}

/// Verdict record with metadata and storage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictRecord {
    pub verdict_id: VerdictId,
    pub consensus_result: ConsensusResult,
    pub debate_session: Option<DebateSession>,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub accessed_at: chrono::DateTime<chrono::Utc>,
    pub access_count: u64,
    pub storage_location: Option<String>,
}

/// Storage backend trait for verdict persistence
pub trait VerdictStorage: Send + Sync + std::fmt::Debug {
    fn store_verdict(&self, record: &VerdictRecord) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;
    fn load_verdict(&self, verdict_id: VerdictId) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<VerdictRecord>>> + Send>>;
    fn load_verdicts_by_task(&self, task_id: TaskId) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VerdictRecord>>> + Send>>;
    fn load_verdicts_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VerdictRecord>>> + Send>>;
    fn delete_verdict(&self, verdict_id: VerdictId) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>;
    fn get_storage_stats(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<StorageStats>> + Send>>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_verdicts: u64,
    pub total_debates: u64,
    pub storage_size_bytes: u64,
    pub oldest_verdict: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_verdict: Option<chrono::DateTime<chrono::Utc>>,
}

impl VerdictStore {
    /// Create a new verdict store
    pub fn new() -> Self {
        Self::with_storage(Arc::new(MemoryVerdictStorage::new()), CacheConfig::default())
    }

    /// Create a new verdict store with custom storage backend
    pub fn with_storage(storage: Arc<dyn VerdictStorage>, cache_config: CacheConfig) -> Self {
        Self {
            cache: Arc::new(DashMap::new()),
            storage,
            cache_config,
        }
    }

    /// Store a consensus result and associated debate session
    pub async fn store_consensus(
        &self,
        consensus_result: ConsensusResult,
        debate_session: Option<DebateSession>,
    ) -> Result<VerdictId> {
        let verdict_id = consensus_result.verdict_id;
        let now = chrono::Utc::now();

        let record = VerdictRecord {
            verdict_id,
            consensus_result,
            debate_session,
            created_at: now,
            accessed_at: now,
            access_count: 0,
            storage_location: None,
        };

        // Store in cache
        self.cache.insert(verdict_id, record.clone());

        // Persist to storage if enabled
        if self.cache_config.enable_persistence {
            if let Err(e) = self.storage.store_verdict(&record).await {
                error!("Failed to persist verdict {}: {}", verdict_id, e);
                // Don't fail the operation, just log the error
            }
        }

        // Clean up cache if needed
        self.cleanup_cache().await;

        info!("Stored verdict {} for task {}", verdict_id, record.consensus_result.task_id);
        Ok(verdict_id)
    }

    /// Retrieve a verdict by ID
    pub async fn get_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>> {
        // Try cache first
        if let Some(mut record) = self.cache.get_mut(&verdict_id) {
            record.accessed_at = chrono::Utc::now();
            record.access_count += 1;
            debug!("Retrieved verdict {} from cache", verdict_id);
            return Ok(Some(record.clone()));
        }

        // Try persistent storage
        if self.cache_config.enable_persistence {
            if let Some(mut record) = self.storage.load_verdict(verdict_id).await? {
                record.accessed_at = chrono::Utc::now();
                record.access_count += 1;

                // Add to cache
                self.cache.insert(verdict_id, record.clone());

                debug!("Retrieved verdict {} from storage", verdict_id);
                return Ok(Some(record));
            }
        }

        debug!("Verdict {} not found", verdict_id);
        Ok(None)
    }

    /// Get all verdicts for a specific task
    pub async fn get_verdicts_for_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>> {
        if self.cache_config.enable_persistence {
            self.storage.load_verdicts_by_task(task_id).await
        } else {
            // Search cache only
            let mut results = Vec::new();
            for entry in self.cache.iter() {
                if entry.value().consensus_result.task_id == task_id {
                    results.push(entry.value().clone());
                }
            }
            Ok(results)
        }
    }

    /// Get verdicts within a time range
    pub async fn get_verdicts_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<VerdictRecord>> {
        if self.cache_config.enable_persistence {
            self.storage.load_verdicts_by_time_range(start, end).await
        } else {
            // Search cache only
            let mut results = Vec::new();
            for entry in self.cache.iter() {
                let created_at = entry.value().created_at;
                if created_at >= start && created_at <= end {
                    results.push(entry.value().clone());
                }
            }
            Ok(results)
        }
    }

    /// Delete a verdict (for testing or cleanup)
    pub async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()> {
        // Remove from cache
        self.cache.remove(&verdict_id);

        // Remove from persistent storage
        if self.cache_config.enable_persistence {
            self.storage.delete_verdict(verdict_id).await?;
        }

        debug!("Deleted verdict {}", verdict_id);
        Ok(())
    }

    /// Get storage statistics
    pub async fn get_stats(&self) -> Result<VerdictStoreStats> {
        let cache_stats = CacheStats {
            cached_verdicts: self.cache.len(),
            max_cached_verdicts: self.cache_config.max_cached_verdicts,
            cache_ttl_seconds: self.cache_config.cache_ttl_seconds,
        };

        let storage_stats = if self.cache_config.enable_persistence {
            Some(self.storage.get_storage_stats().await?)
        } else {
            None
        };

        Ok(VerdictStoreStats {
            cache: cache_stats,
            storage: storage_stats,
        })
    }

    /// Clean up cache based on TTL and size limits
    async fn cleanup_cache(&self) {
        let now = chrono::Utc::now();
        let ttl = chrono::Duration::seconds(self.cache_config.cache_ttl_seconds as i64);

        // Remove expired entries
        self.cache.retain(|_, record| {
            now.signed_duration_since(record.accessed_at) < ttl
        });

        // If still over limit, remove least recently accessed
        if self.cache.len() > self.cache_config.max_cached_verdicts {
            let mut entries: Vec<_> = self.cache.iter().collect();
            entries.sort_by_key(|entry| entry.value().accessed_at);
            
            let to_remove = entries.len() - self.cache_config.max_cached_verdicts;
            for entry in entries.into_iter().take(to_remove) {
                self.cache.remove(entry.key());
            }
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerdictStoreStats {
    pub cache: CacheStats,
    pub storage: Option<StorageStats>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheStats {
    pub cached_verdicts: usize,
    pub max_cached_verdicts: usize,
    pub cache_ttl_seconds: u64,
}

/// In-memory storage implementation for testing
#[derive(Debug)]
pub struct MemoryVerdictStorage {
    verdicts: Arc<RwLock<std::collections::HashMap<VerdictId, VerdictRecord>>>,
}

impl MemoryVerdictStorage {
    pub fn new() -> Self {
        Self {
            verdicts: Arc::new(RwLock::new(std::collections::HashMap::new())),
        }
    }
}

impl VerdictStorage for MemoryVerdictStorage {
    fn store_verdict(&self, record: &VerdictRecord) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> {
        let verdicts = self.verdicts.clone();
        Box::pin(async move {
            let mut verdicts = verdicts.write().await;
            verdicts.insert(record.verdict_id, record.clone());
            Ok(())
        })
    }

    fn load_verdict(&self, verdict_id: VerdictId) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Option<VerdictRecord>>> + Send>> {
        let verdicts = self.verdicts.clone();
        Box::pin(async move {
            let verdicts = verdicts.read().await;
            Ok(verdicts.get(&verdict_id).cloned())
        })
    }

    fn load_verdicts_by_task(&self, task_id: TaskId) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VerdictRecord>>> + Send>> {
        let verdicts = self.verdicts.clone();
        Box::pin(async move {
            let verdicts = verdicts.read().await;
            Ok(verdicts
                .values()
                .filter(|record| record.consensus_result.task_id == task_id)
                .cloned()
                .collect())
        })
    }

    fn load_verdicts_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Vec<VerdictRecord>>> + Send>> {
        let verdicts = self.verdicts.clone();
        Box::pin(async move {
            let verdicts = verdicts.read().await;
            Ok(verdicts
                .values()
                .filter(|record| {
                    let created_at = record.created_at;
                    created_at >= start && created_at <= end
                })
                .cloned()
                .collect())
        })
    }

    fn delete_verdict(&self, verdict_id: VerdictId) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>> {
        let verdicts = self.verdicts.clone();
        Box::pin(async move {
            let mut verdicts = verdicts.write().await;
            verdicts.remove(&verdict_id);
            Ok(())
        })
    }

    fn get_storage_stats(&self) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<StorageStats>> + Send>> {
        let verdicts = self.verdicts.clone();
        Box::pin(async move {
            let verdicts = verdicts.read().await;
            let total_verdicts = verdicts.len() as u64;

            let (oldest, newest) = if total_verdicts > 0 {
                let mut timestamps: Vec<_> = verdicts.values().map(|r| r.created_at).collect();
                timestamps.sort();
                (Some(timestamps[0]), Some(timestamps[timestamps.len() - 1]))
            } else {
                (None, None)
            };

            Ok(StorageStats {
                total_verdicts,
                total_debates: verdicts.values().filter(|r| r.debate_session.is_some()).count() as u64,
                storage_size_bytes: total_verdicts * 1024, // Rough estimate
                oldest_verdict: oldest,
                newest_verdict: newest,
            })
        })
    }
}

/// Database storage implementation for verdict records
#[derive(Debug)]
pub struct DatabaseVerdictStorage {
    /// Database client for executing queries
    db_client: Arc<DatabaseClient>,
}

impl DatabaseVerdictStorage {
    /// Create new database verdict storage with existing database client
    pub fn new(db_client: Arc<DatabaseClient>) -> Self {
        Self { db_client }
    }
}

#[async_trait]
impl VerdictStorage for DatabaseVerdictStorage {
    async fn store_verdict(&self, record: &VerdictRecord) -> Result<()> {
        // Serialize the verdict record to JSON
        let consensus_json = serde_json::to_string(&record.consensus_result)
            .context("Failed to serialize consensus result")?;
        let debate_json = record.debate_session.as_ref()
            .map(|ds| serde_json::to_string(ds))
            .transpose()
            .context("Failed to serialize debate session")?;

        let storage_location = record.storage_location.clone()
            .unwrap_or_else(|| format!("verdict_{}", record.verdict_id.0));

        // Prepare parameters for database insertion
        let params = vec![
            serde_json::Value::String(record.verdict_id.0.to_string()),
            serde_json::Value::String(record.consensus_result.task_id.0.to_string()),
            serde_json::Value::String(consensus_json),
            debate_json.map(serde_json::Value::String).unwrap_or(serde_json::Value::Null),
            serde_json::Value::String(record.created_at.to_rfc3339()),
            serde_json::Value::String(record.accessed_at.to_rfc3339()),
            serde_json::Value::Number(record.access_count.into()),
            serde_json::Value::String(storage_location),
        ];

        // Insert into database using parameterized query
        let query = r#"
            INSERT INTO council_verdicts (
                verdict_id, task_id, consensus_result, debate_session,
                created_at, accessed_at, access_count, storage_location
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            ON CONFLICT (verdict_id) DO UPDATE SET
                accessed_at = EXCLUDED.accessed_at,
                access_count = EXCLUDED.access_count
        "#;

        self.db_client.execute_parameterized_query(query, params).await
            .context("Failed to store verdict in database")?;

        info!("Stored verdict {} in database", record.verdict_id.0);
        Ok(())
    }

    async fn load_verdict(&self, verdict_id: VerdictId) -> Result<Option<VerdictRecord>> {
        let params = vec![serde_json::Value::String(verdict_id.0.to_string())];

        let query = r#"
            SELECT verdict_id, task_id, consensus_result, debate_session,
                   created_at, accessed_at, access_count, storage_location
            FROM council_verdicts
            WHERE verdict_id = $1
        "#;

        // For now, we'll use a simpler approach since we need to handle the result
        // TODO: Implement comprehensive database verdict loading with the following requirements:
        // 1. SQL query optimization: Use optimized SQL queries for verdict retrieval
        //    - Implement sqlx::query_as! macros for type-safe query execution
        //    - Use prepared statements and parameterized queries for security
        //    - Optimize query performance with proper indexing strategies
        //    - Handle query result mapping and type conversion efficiently
        // 2. Verdict data retrieval: Retrieve complete verdict data from database
        //    - Load verdict records with all associated metadata and evidence
        //    - Handle verdict relationships and foreign key constraints
        //    - Support verdict versioning and historical data retrieval
        //    - Implement verdict data validation and integrity checks
        // 3. Error handling and recovery: Implement robust error handling for database operations
        //    - Handle database connection failures and retry logic
        //    - Implement proper transaction management and rollback
        //    - Provide meaningful error messages and logging for debugging
        //    - Support database migration and schema evolution
        // 4. Performance optimization: Optimize verdict loading performance and scalability
        //    - Implement caching strategies for frequently accessed verdicts
        //    - Use connection pooling and database resource management
        //    - Support batch loading and lazy loading of verdict data
        //    - Monitor query performance and implement query optimization
        // For this placeholder, we'll return None to indicate not implemented
        warn!("Database verdict loading not fully implemented yet - returning None for {}", verdict_id.0);
        Ok(None)
    }

    async fn load_verdicts_by_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>> {
        let params = vec![serde_json::Value::String(task_id.0.to_string())];

        let query = r#"
            SELECT verdict_id, task_id, consensus_result, debate_session,
                   created_at, accessed_at, access_count, storage_location
            FROM council_verdicts
            WHERE task_id = $1
            ORDER BY created_at DESC
        "#;

        // TODO: Implement comprehensive task-based verdict loading with the following requirements:
        // 1. Task-verdict relationship mapping: Map verdicts to their associated tasks
        //    - Query verdict records by task ID with proper indexing
        //    - Handle one-to-many relationships between tasks and verdicts
        //    - Support verdict ordering and prioritization by task
        //    - Implement efficient task-verdict lookup and retrieval
        // 2. Verdict aggregation and filtering: Aggregate verdicts by task criteria
        //    - Filter verdicts by task status, priority, and completion state
        //    - Aggregate verdict statistics and metrics per task
        //    - Support verdict sorting and pagination for large task sets
        //    - Handle verdict conflicts and resolution within tasks
        // 3. Performance optimization: Optimize task-based verdict queries
        //    - Implement database indexing for task-verdict relationships
        //    - Use query optimization and result caching strategies
        //    - Support batch loading and lazy loading of task verdicts
        //    - Monitor query performance and implement optimizations
        // 4. Data integrity validation: Ensure data consistency for task verdicts
        //    - Validate task-verdict relationships and referential integrity
        //    - Handle orphaned verdicts and missing task references
        //    - Implement data cleanup and maintenance procedures
        //    - Support audit trails for verdict-task associations
        warn!("Database task-based verdict loading not fully implemented yet - returning empty for {}", task_id.0);
        Ok(vec![])
    }

    async fn load_verdicts_by_time_range(
        &self,
        _start: chrono::DateTime<chrono::Utc>,
        _end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<VerdictRecord>> {
        // TODO: Implement database query with the following requirements:
        // 1. Query construction: Construct database queries for time-based verdict retrieval
        //    - Build SQL queries to fetch verdicts within time range
        //    - Handle query optimization and performance
        //    - Implement proper query security and injection prevention
        // 2. Data retrieval: Retrieve verdict records within specified time range
        //    - Execute database queries and fetch multiple results
        //    - Handle database connection and transaction management
        //    - Implement proper error handling and timeout management
        // 3. Data processing: Process and validate retrieved verdict data
        //    - Convert database rows to verdict record structures
        //    - Handle data type conversions and validation
        //    - Implement proper data decoding and decompression
        // 4. Result formatting: Format and return retrieved verdict records
        //    - Validate data integrity and completeness
        //    - Handle missing or corrupted data
        //    - Implement proper result formatting and return
        Ok(Vec::new())
    }

    async fn delete_verdict(&self, verdict_id: VerdictId) -> Result<()> {
        let params = vec![serde_json::Value::String(verdict_id.0.to_string())];

        let query = "DELETE FROM council_verdicts WHERE verdict_id = $1";

        let result = self.db_client.execute_parameterized_query(query, params).await
            .context("Failed to delete verdict from database")?;

        if result.rows_affected() == 0 {
            warn!("No verdict found with ID {} to delete", verdict_id.0);
        } else {
            info!("Deleted verdict {} from database", verdict_id.0);
        }

        Ok(())
    }

    async fn get_storage_stats(&self) -> Result<StorageStats> {
        let query = r#"
            SELECT
                COUNT(*) as total_verdicts,
                COUNT(debate_session) FILTER (WHERE debate_session IS NOT NULL) as total_debates,
                COALESCE(SUM(pg_column_size(consensus_result) + pg_column_size(debate_session)), 0) as storage_size,
                MIN(created_at) as oldest_verdict,
                MAX(created_at) as newest_verdict
            FROM council_verdicts
        "#;

        // TODO: Implement comprehensive storage statistics collection with the following requirements:
        // 1. Statistics aggregation: Aggregate verdict storage statistics from database
        //    - Query total verdict counts and storage metrics from database
        //    - Calculate storage size, access patterns, and usage statistics
        //    - Aggregate debate session counts and performance metrics
        //    - Handle statistics calculation for large datasets efficiently
        // 2. Historical data analysis: Analyze historical verdict and debate data
        //    - Track verdict creation dates and identify oldest/newest records
        //    - Calculate verdict lifecycle statistics and retention metrics
        //    - Analyze debate session durations and outcome distributions
        //    - Implement time-based statistics and trending analysis
        // 3. Performance metrics: Calculate storage and access performance metrics
        //    - Measure query performance and response times for verdict operations
        //    - Track storage utilization and efficiency metrics
        //    - Monitor database performance and optimization opportunities
        //    - Implement performance benchmarking and alerting
        // 4. Data quality monitoring: Monitor verdict data quality and integrity
        //    - Validate verdict data completeness and consistency
        //    - Detect data anomalies and quality issues in storage
        //    - Implement data cleanup and maintenance procedures
        //    - Provide data quality metrics and health monitoring
        warn!("Database storage statistics not fully implemented yet - returning defaults");
        Ok(StorageStats {
            total_verdicts: 0,
            total_debates: 0,
            storage_size_bytes: 0,
            oldest_verdict: None,
            newest_verdict: None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_verdict_store_creation() {
        let store = VerdictStore::new();
        assert_eq!(store.cache.len(), 0);
    }

    #[tokio::test]
    async fn test_store_and_retrieve_verdict() {
        let store = VerdictStore::new();
        
        let task_id = Uuid::new_v4();
        let verdict_id = Uuid::new_v4();
        
        let consensus_result = ConsensusResult {
            task_id,
            verdict_id,
            final_verdict: FinalVerdict::Accepted {
                confidence: 0.9,
                summary: "Test verdict".to_string(),
            },
            individual_verdicts: std::collections::HashMap::new(),
            consensus_score: 0.9,
            debate_rounds: 0,
            evaluation_time_ms: 100,
            timestamp: chrono::Utc::now(),
        };

        let stored_id = store.store_consensus(consensus_result.clone(), None).await.unwrap();
        assert_eq!(stored_id, verdict_id);

        let retrieved = store.get_verdict(verdict_id).await.unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().consensus_result.task_id, task_id);
    }

    #[tokio::test]
    async fn test_verdict_not_found() {
        let store = VerdictStore::new();
        let verdict_id = Uuid::new_v4();
        
        let result = store.get_verdict(verdict_id).await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let config = CacheConfig {
            max_cached_verdicts: 2,
            cache_ttl_seconds: 1,
            enable_persistence: false,
        };
        
        let store = VerdictStore::with_storage(Arc::new(MemoryVerdictStorage::new()), config);
        
        // Store 3 verdicts (exceeds cache limit)
        for i in 0..3 {
            let task_id = Uuid::new_v4();
            let verdict_id = Uuid::new_v4();
            
            let consensus_result = ConsensusResult {
                task_id,
                verdict_id,
                final_verdict: FinalVerdict::Accepted {
                    confidence: 0.9,
                    summary: format!("Test verdict {}", i),
                },
                individual_verdicts: std::collections::HashMap::new(),
                consensus_score: 0.9,
                debate_rounds: 0,
                evaluation_time_ms: 100,
                timestamp: chrono::Utc::now(),
            };

            store.store_consensus(consensus_result, None).await.unwrap();
        }

        // Cache should be cleaned up to max_cached_verdicts
        assert!(store.cache.len() <= 2);
    }
}
