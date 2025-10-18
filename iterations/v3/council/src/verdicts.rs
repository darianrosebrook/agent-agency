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
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

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
        // Use direct sqlx query for proper data retrieval
        let record = sqlx::query!(
            r#"
            SELECT
                verdict_id, task_id, consensus_result, debate_session,
                created_at, accessed_at, access_count, storage_location
            FROM council_verdicts
            WHERE verdict_id = $1
            "#,
            verdict_id.0
        )
        .fetch_optional(self.db_client.pool())
        .await
        .context("Failed to query verdict from database")?;

        match record {
            Some(row) => {
                // Parse JSON fields
                let consensus_result: ConsensusResult = serde_json::from_str(&row.consensus_result)
                    .context("Failed to deserialize consensus result")?;

                let debate_session = if let Some(debate_json) = row.debate_session {
                    Some(serde_json::from_str(&debate_json)
                        .context("Failed to deserialize debate session")?)
                } else {
                    None
                };

                let verdict_record = VerdictRecord {
                    verdict_id: VerdictId(uuid::Uuid::parse_str(&row.verdict_id)?),
                    consensus_result,
                    debate_session,
                    created_at: row.created_at,
                    accessed_at: row.accessed_at,
                    access_count: row.access_count as u64,
                    storage_location: row.storage_location,
                };

                debug!("Loaded verdict {} from database", verdict_id.0);
                Ok(Some(verdict_record))
            }
            None => {
                debug!("Verdict {} not found in database", verdict_id.0);
                Ok(None)
            }
        }
    }

    async fn load_verdicts_by_task(&self, task_id: TaskId) -> Result<Vec<VerdictRecord>> {
        // Use direct sqlx query for proper data retrieval
        let records = sqlx::query!(
            r#"
            SELECT
                verdict_id, task_id, consensus_result, debate_session,
                created_at, accessed_at, access_count, storage_location
            FROM council_verdicts
            WHERE task_id = $1
            ORDER BY created_at DESC
            "#,
            task_id.0
        )
        .fetch_all(self.db_client.pool())
        .await
        .context("Failed to query verdicts by task from database")?;

        let mut verdict_records = Vec::new();

        for row in records {
            // Parse JSON fields
            let consensus_result: ConsensusResult = serde_json::from_str(&row.consensus_result)
                .context("Failed to deserialize consensus result")?;

            let debate_session = if let Some(debate_json) = row.debate_session {
                Some(serde_json::from_str(&debate_json)
                    .context("Failed to deserialize debate session")?)
            } else {
                None
            };

            let verdict_record = VerdictRecord {
                verdict_id: VerdictId(uuid::Uuid::parse_str(&row.verdict_id)?),
                consensus_result,
                debate_session,
                created_at: row.created_at,
                accessed_at: row.accessed_at,
                access_count: row.access_count as u64,
                storage_location: row.storage_location,
            };

            verdict_records.push(verdict_record);
        }

        debug!("Loaded {} verdicts for task {} from database", verdict_records.len(), task_id.0);
        Ok(verdict_records)
    }

    async fn load_verdicts_by_time_range(
        &self,
        start: chrono::DateTime<chrono::Utc>,
        end: chrono::DateTime<chrono::Utc>,
    ) -> Result<Vec<VerdictRecord>> {
        // Use direct sqlx query for proper data retrieval
        let records = sqlx::query!(
            r#"
            SELECT
                verdict_id, task_id, consensus_result, debate_session,
                created_at, accessed_at, access_count, storage_location
            FROM council_verdicts
            WHERE created_at >= $1 AND created_at <= $2
            ORDER BY created_at DESC
            "#,
            start,
            end
        )
        .fetch_all(self.db_client.pool())
        .await
        .context("Failed to query verdicts by time range from database")?;

        let mut verdict_records = Vec::new();

        for row in records {
            // Parse JSON fields
            let consensus_result: ConsensusResult = serde_json::from_str(&row.consensus_result)
                .context("Failed to deserialize consensus result")?;

            let debate_session = if let Some(debate_json) = row.debate_session {
                Some(serde_json::from_str(&debate_json)
                    .context("Failed to deserialize debate session")?)
            } else {
                None
            };

            let verdict_record = VerdictRecord {
                verdict_id: VerdictId(uuid::Uuid::parse_str(&row.verdict_id)?),
                consensus_result,
                debate_session,
                created_at: row.created_at,
                accessed_at: row.accessed_at,
                access_count: row.access_count as u64,
                storage_location: row.storage_location,
            };

            verdict_records.push(verdict_record);
        }

        debug!("Loaded {} verdicts in time range {} to {} from database",
               verdict_records.len(), start, end);
        Ok(verdict_records)
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
        // Query storage statistics from database
        let stats = sqlx::query!(
            r#"
            SELECT
                COUNT(*) as "total_verdicts!",
                COUNT(debate_session) FILTER (WHERE debate_session IS NOT NULL) as "total_debates!",
                COALESCE(SUM(pg_column_size(consensus_result) + COALESCE(pg_column_size(debate_session), 0)), 0) as "storage_size_bytes!",
                MIN(created_at) as "oldest_verdict",
                MAX(created_at) as "newest_verdict"
            FROM council_verdicts
            "#
        )
        .fetch_one(self.db_client.pool())
        .await
        .context("Failed to query storage statistics from database")?;

        Ok(StorageStats {
            total_verdicts: stats.total_verdicts as u64,
            total_debates: stats.total_debates as u64,
            storage_size_bytes: stats.storage_size_bytes as u64,
            oldest_verdict: stats.oldest_verdict,
            newest_verdict: stats.newest_verdict,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::*;
    use std::collections::HashMap;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_verdict_store_creation() {
        let store = VerdictStore::new();
        assert_eq!(store.cache.len(), 0);
    }

    // Test database utilities
    #[cfg(test)]
    mod test_utils {
        use super::*;
        use crate::database::DatabaseClient;
        use std::env;
        use sqlx::PgPool;

        /// Test database setup utility
        pub struct TestDatabase {
            pub client: DatabaseClient,
            pub pool: PgPool,
            test_prefix: String,
        }

        impl TestDatabase {
            /// Create a new test database connection
            pub async fn new() -> Result<Self> {
                let test_prefix = format!("test_{}", uuid::Uuid::new_v4().simple());

                let config = crate::database::DatabaseConfig {
                    host: env::var("TEST_DB_HOST").unwrap_or("localhost".to_string()),
                    port: env::var("TEST_DB_PORT").unwrap_or("5432".parse().unwrap()),
                    database: env::var("TEST_DB_NAME").unwrap_or("agent_agency_test".to_string()),
                    username: env::var("TEST_DB_USER").unwrap_or("postgres".to_string()),
                    password: env::var("TEST_DB_PASSWORD").unwrap_or("password".to_string()),
                    server_url: env::var("TEST_DB_URL").unwrap_or("postgresql://localhost:5432".to_string()),
                    database_url: env::var("TEST_DATABASE_URL").unwrap_or("postgresql://postgres:password@localhost:5432/agent_agency_test".to_string()),
                    pool_max: 5,
                    pool_min: 1,
                    pool_timeout_seconds: 30,
                    connection_timeout_seconds: 10,
                };

                let client = DatabaseClient::new(config).await?;
                let pool = client.pool().clone();

                // Ensure test tables exist
                Self::setup_test_tables(&client).await?;

                Ok(Self {
                    client,
                    pool,
                    test_prefix,
                })
            }

            /// Set up test tables if they don't exist
            async fn setup_test_tables(client: &DatabaseClient) -> Result<()> {
                // Create council_verdicts table if it doesn't exist
                let create_table_sql = r#"
                    CREATE TABLE IF NOT EXISTS council_verdicts (
                        verdict_id TEXT PRIMARY KEY,
                        task_id TEXT NOT NULL,
                        consensus_result JSONB NOT NULL,
                        debate_session JSONB,
                        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                        accessed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                        access_count INTEGER NOT NULL DEFAULT 0,
                        storage_location TEXT
                    );

                    CREATE INDEX IF NOT EXISTS idx_council_verdicts_task_id ON council_verdicts(task_id);
                    CREATE INDEX IF NOT EXISTS idx_council_verdicts_created_at ON council_verdicts(created_at);
                "#;

                client.execute_safe_query(create_table_sql).await?;
                Ok(())
            }

            /// Generate a unique test identifier
            pub fn unique_id(&self, suffix: &str) -> String {
                format!("{}_{}", self.test_prefix, suffix)
            }

            /// Clean up test data
            pub async fn cleanup(&self) -> Result<()> {
                // Delete all test data with our prefix
                let cleanup_sql = format!(
                    "DELETE FROM council_verdicts WHERE verdict_id LIKE '{}%'",
                    self.test_prefix
                );

                self.client.execute_safe_query(&cleanup_sql).await?;
                Ok(())
            }
        }

        impl Drop for TestDatabase {
            fn drop(&mut self) {
                // Note: cleanup should be called explicitly since async drop isn't stable
                // The test functions handle cleanup
            }
        }
    }

    // Database integration tests
    #[cfg(feature = "integration-tests")]
    mod database_integration_tests {
        use super::*;
        use crate::test_utils::TestDatabase;

        fn create_test_consensus_result(task_id: TaskId, verdict_id: VerdictId) -> ConsensusResult {
            ConsensusResult {
                task_id,
                verdict_id,
                consensus_score: 0.85,
                final_verdict: serde_json::json!({
                    "type": "approved",
                    "reasoning": "Task meets all quality criteria",
                    "confidence": 0.85
                }),
                individual_verdicts: vec![
                    serde_json::json!({
                        "judge_id": "constitutional-judge",
                        "verdict": "approved",
                        "confidence": 0.95,
                        "reasoning": "No constitutional violations"
                    }),
                    serde_json::json!({
                        "judge_id": "technical-judge",
                        "verdict": "approved",
                        "confidence": 0.88,
                        "reasoning": "Technical implementation is sound"
                    }),
                ],
                evaluation_time_ms: 1250,
                debate_rounds: 2,
                metadata: HashMap::new(),
            }
        }

        fn create_test_debate_session() -> DebateSession {
            DebateSession {
                session_id: Uuid::new_v4(),
                task_id: TaskId(Uuid::new_v4()),
                conflicting_judges: vec![
                    serde_json::json!("constitutional-judge"),
                    serde_json::json!("quality-judge"),
                ],
                rounds: vec![
                    serde_json::json!({
                        "round": 1,
                        "arguments": [
                            {"judge": "constitutional-judge", "argument": "Task violates scope constraints"},
                            {"judge": "quality-judge", "argument": "Scope constraints are acceptable given context"}
                        ]
                    }),
                ],
                status: "resolved".to_string(),
                final_consensus: Some(serde_json::json!({
                    "decision": "approved",
                    "rationale": "Quality judge arguments prevailed"
                })),
                created_at: chrono::Utc::now(),
                resolved_at: Some(chrono::Utc::now()),
            }
        }

        #[tokio::test]
        async fn test_council_verdict_storage_and_retrieval() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());
            let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id("verdict")).unwrap());
            let consensus_result = create_test_consensus_result(task_id, verdict_id);
            let debate_session = Some(create_test_debate_session());

            // Create test verdict record
            let verdict_record = VerdictRecord {
                verdict_id,
                consensus_result,
                debate_session,
                created_at: chrono::Utc::now(),
                accessed_at: chrono::Utc::now(),
                access_count: 1,
                storage_location: Some("test_location".to_string()),
            };

            // Test storing verdict
            storage.store_verdict(&verdict_record).await?;

            // Test loading verdict
            let loaded_verdict = storage.load_verdict(verdict_id).await?;
            assert!(loaded_verdict.is_some(), "Verdict should be found after storing");

            let loaded = loaded_verdict.unwrap();
            assert_eq!(loaded.verdict_id, verdict_id);
            assert_eq!(loaded.consensus_result.task_id, task_id);
            assert_eq!(loaded.consensus_result.consensus_score, 0.85);
            assert!(loaded.debate_session.is_some(), "Debate session should be present");

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdicts_by_task_loading() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());

            // Create multiple verdict records for the same task
            let mut verdict_ids = Vec::new();
            for i in 0..3 {
                let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id(&format!("verdict_{}", i))).unwrap());
                verdict_ids.push(verdict_id);

                let mut consensus_result = create_test_consensus_result(task_id, verdict_id);
                consensus_result.consensus_score = 0.8 + (i as f32 * 0.05); // Different scores

                let verdict_record = VerdictRecord {
                    verdict_id,
                    consensus_result,
                    debate_session: None,
                    created_at: chrono::Utc::now(),
                    accessed_at: chrono::Utc::now(),
                    access_count: i + 1,
                    storage_location: Some(format!("test_location_{}", i)),
                };

                storage.store_verdict(&verdict_record).await?;
            }

            // Test loading verdicts by task
            let loaded_verdicts = storage.load_verdicts_by_task(task_id).await?;
            assert_eq!(loaded_verdicts.len(), 3, "Should load all 3 verdicts for the task");

            // Verify they are ordered by creation time (most recent first)
            for i in 0..loaded_verdicts.len() - 1 {
                assert!(loaded_verdicts[i].created_at >= loaded_verdicts[i + 1].created_at,
                       "Verdicts should be ordered by creation time descending");
            }

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_deletion() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());
            let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id("delete_verdict")).unwrap());
            let consensus_result = create_test_consensus_result(task_id, verdict_id);

            let verdict_record = VerdictRecord {
                verdict_id,
                consensus_result,
                debate_session: None,
                created_at: chrono::Utc::now(),
                accessed_at: chrono::Utc::now(),
                access_count: 1,
                storage_location: None,
            };

            // Store verdict
            storage.store_verdict(&verdict_record).await?;

            // Verify it exists
            let loaded = storage.load_verdict(verdict_id).await?;
            assert!(loaded.is_some(), "Verdict should exist before deletion");

            // Delete verdict
            storage.delete_verdict(verdict_id).await?;

            // Verify it no longer exists
            let loaded_after_delete = storage.load_verdict(verdict_id).await?;
            assert!(loaded_after_delete.is_none(), "Verdict should not exist after deletion");

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_storage_statistics() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            // Get initial stats
            let initial_stats = storage.get_storage_stats().await?;
            let initial_count = initial_stats.total_verdicts;

            // Create and store test verdicts
            for i in 0..2 {
                let task_id = TaskId(Uuid::new_v4());
                let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id(&format!("stats_verdict_{}", i))).unwrap());

                let consensus_result = create_test_consensus_result(task_id, verdict_id);
                let debate_session = if i == 0 { Some(create_test_debate_session()) } else { None };

                let verdict_record = VerdictRecord {
                    verdict_id,
                    consensus_result,
                    debate_session,
                    created_at: chrono::Utc::now(),
                    accessed_at: chrono::Utc::now(),
                    access_count: 1,
                    storage_location: Some(format!("stats_test_{}", i)),
                };

                storage.store_verdict(&verdict_record).await?;
            }

            // Get updated stats
            let updated_stats = storage.get_storage_stats().await?;
            assert_eq!(updated_stats.total_verdicts, initial_count + 2,
                      "Should have 2 more verdicts");
            assert_eq!(updated_stats.total_debates, initial_count + 1,
                      "Should have 1 debate session");
            assert!(updated_stats.storage_size_bytes > 0,
                   "Storage size should be greater than 0");
            assert!(updated_stats.oldest_verdict.is_some(),
                   "Should have oldest verdict timestamp");
            assert!(updated_stats.newest_verdict.is_some(),
                   "Should have newest verdict timestamp");

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_time_range_loading() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let base_time = chrono::Utc::now();
            let task_id = TaskId(Uuid::new_v4());

            // Create verdicts at different times
            for i in 0..3 {
                let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id(&format!("time_verdict_{}", i))).unwrap());
                let consensus_result = create_test_consensus_result(task_id, verdict_id);

                // Create verdict with different creation times
                let created_at = base_time + chrono::Duration::hours(i as i64);

                let verdict_record = VerdictRecord {
                    verdict_id,
                    consensus_result,
                    debate_session: None,
                    created_at,
                    accessed_at: created_at,
                    access_count: 1,
                    storage_location: Some(format!("time_test_{}", i)),
                };

                storage.store_verdict(&verdict_record).await?;
            }

            // Test loading verdicts in time range
            let start_time = base_time - chrono::Duration::minutes(1);
            let end_time = base_time + chrono::Duration::hours(2);

            let time_range_verdicts = storage.load_verdicts_by_time_range(start_time, end_time).await?;
            assert_eq!(time_range_verdicts.len(), 2,
                      "Should load verdicts within time range (first 2 hours)");

            // Test with narrower time range
            let narrow_start = base_time + chrono::Duration::hours(1);
            let narrow_end = base_time + chrono::Duration::hours(2);
            let narrow_verdicts = storage.load_verdicts_by_time_range(narrow_start, narrow_end).await?;
            assert_eq!(narrow_verdicts.len(), 1,
                      "Should load only 1 verdict in narrow time range");

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_nonexistent_loading() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            // Test loading non-existent verdict
            let nonexistent_id = VerdictId(Uuid::parse_str(&test_db.unique_id("nonexistent")).unwrap());
            let loaded = storage.load_verdict(nonexistent_id).await?;
            assert!(loaded.is_none(), "Loading non-existent verdict should return None");

            // Test loading verdicts for non-existent task
            let nonexistent_task_id = TaskId(Uuid::new_v4());
            let task_verdicts = storage.load_verdicts_by_task(nonexistent_task_id).await?;
            assert!(task_verdicts.is_empty(),
                   "Loading verdicts for non-existent task should return empty vector");

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_audit_trail() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());
            let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id("audit_verdict")).unwrap());
            let consensus_result = create_test_consensus_result(task_id, verdict_id);

            let verdict_record = VerdictRecord {
                verdict_id,
                consensus_result,
                debate_session: None,
                created_at: chrono::Utc::now(),
                accessed_at: chrono::Utc::now(),
                access_count: 1,
                storage_location: None,
            };

            // Store verdict (should create audit trail entry)
            storage.store_verdict(&verdict_record).await?;

            // Verify audit trail was created
            let audit_count = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM audit_trail WHERE entity_type = 'council_verdict' AND entity_id = $1",
                verdict_id.0
            )
            .fetch_one(test_db.pool())
            .await?;

            assert_eq!(audit_count.unwrap_or(0), 1,
                      "Should have created one audit trail entry for verdict creation");

            // Delete verdict (should create another audit trail entry)
            storage.delete_verdict(verdict_id).await?;

            let audit_count_after_delete = sqlx::query_scalar!(
                "SELECT COUNT(*) FROM audit_trail WHERE entity_type = 'council_verdict' AND entity_id = $1",
                verdict_id.0
            )
            .fetch_one(test_db.pool())
            .await?;

            assert_eq!(audit_count_after_delete.unwrap_or(0), 2,
                      "Should have two audit trail entries after create and delete");

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_transaction_integrity() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());

            // Test successful transaction - store multiple verdicts atomically
            let mut verdict_ids = Vec::new();
            for i in 0..3 {
                let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id(&format!("tx_verdict_{}", i))).unwrap());
                verdict_ids.push(verdict_id);

                let consensus_result = create_test_consensus_result(task_id, verdict_id);
                let verdict_record = VerdictRecord {
                    verdict_id,
                    consensus_result,
                    debate_session: None,
                    created_at: chrono::Utc::now(),
                    accessed_at: chrono::Utc::now(),
                    access_count: 1,
                    storage_location: Some(format!("tx_test_{}", i)),
                };

                storage.store_verdict(&verdict_record).await?;
            }

            // Verify all verdicts were stored
            for verdict_id in &verdict_ids {
                let loaded = storage.load_verdict(*verdict_id).await?;
                assert!(loaded.is_some(), "Verdict {} should exist", verdict_id.0);
            }

            // Test load by task returns all verdicts
            let task_verdicts = storage.load_verdicts_by_task(task_id).await?;
            assert_eq!(task_verdicts.len(), 3, "Should load all 3 verdicts for the task");

            // Test deletion transaction integrity
            for verdict_id in &verdict_ids {
                storage.delete_verdict(*verdict_id).await?;
            }

            // Verify all were deleted
            for verdict_id in &verdict_ids {
                let loaded = storage.load_verdict(*verdict_id).await?;
                assert!(loaded.is_none(), "Verdict {} should be deleted", verdict_id.0);
            }

            // Verify task loading returns empty
            let task_verdicts_after_delete = storage.load_verdicts_by_task(task_id).await?;
            assert!(task_verdicts_after_delete.is_empty(), "Should have no verdicts after deletion");

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_concurrent_access() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());
            let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id("concurrent_verdict")).unwrap());
            let consensus_result = create_test_consensus_result(task_id, verdict_id);

            let verdict_record = VerdictRecord {
                verdict_id,
                consensus_result,
                debate_session: None,
                created_at: chrono::Utc::now(),
                accessed_at: chrono::Utc::now(),
                access_count: 1,
                storage_location: Some("concurrent_test".to_string()),
            };

            // Store verdict
            storage.store_verdict(&verdict_record).await?;

            // Test concurrent reads (should work fine)
            let mut handles = Vec::new();
            for _ in 0..5 {
                let storage_clone = DatabaseVerdictStorage::new(Arc::new(test_db.client.clone()));
                let verdict_id_clone = verdict_id;
                let handle = tokio::spawn(async move {
                    storage_clone.load_verdict(verdict_id_clone).await
                });
                handles.push(handle);
            }

            // Wait for all concurrent reads to complete
            for handle in handles {
                let result = handle.await??;
                assert!(result.is_some(), "Concurrent read should succeed");
                assert_eq!(result.unwrap().verdict_id, verdict_id);
            }

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_error_handling() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            // Test loading non-existent verdict
            let nonexistent_id = VerdictId(Uuid::parse_str(&test_db.unique_id("error_test")).unwrap());
            let result = storage.load_verdict(nonexistent_id).await?;
            assert!(result.is_none(), "Loading non-existent verdict should return None");

            // Test deleting non-existent verdict (should not error)
            storage.delete_verdict(nonexistent_id).await?;

            // Test loading verdicts for non-existent task
            let nonexistent_task = TaskId(Uuid::new_v4());
            let task_verdicts = storage.load_verdicts_by_task(nonexistent_task).await?;
            assert!(task_verdicts.is_empty(), "Loading verdicts for non-existent task should return empty");

            // Test time range with no results
            let past_time = chrono::Utc::now() - chrono::Duration::days(365);
            let future_time = chrono::Utc::now() + chrono::Duration::days(365);
            let time_range_verdicts = storage.load_verdicts_by_time_range(past_time, future_time).await?;
            // This might return existing verdicts from other tests, so we just ensure it doesn't error

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_data_consistency() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());
            let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id("consistency_verdict")).unwrap());
            let mut consensus_result = create_test_consensus_result(task_id, verdict_id);

            // Modify consensus result to test data consistency
            consensus_result.consensus_score = 0.95;
            consensus_result.final_verdict = serde_json::json!({"type": "approved", "confidence": 0.95});

            let verdict_record = VerdictRecord {
                verdict_id,
                consensus_result: consensus_result.clone(),
                debate_session: Some(create_test_debate_session()),
                created_at: chrono::Utc::now(),
                accessed_at: chrono::Utc::now(),
                access_count: 5, // Test access count preservation
                storage_location: Some("consistency_test".to_string()),
            };

            // Store verdict
            storage.store_verdict(&verdict_record).await?;

            // Load and verify data consistency
            let loaded = storage.load_verdict(verdict_id).await?;
            assert!(loaded.is_some(), "Verdict should be loaded");

            let loaded_record = loaded.unwrap();
            assert_eq!(loaded_record.verdict_id, verdict_id);
            assert_eq!(loaded_record.consensus_result.task_id, task_id);
            assert_eq!(loaded_record.consensus_result.consensus_score, 0.95);
            assert_eq!(loaded_record.access_count, 5);
            assert!(loaded_record.debate_session.is_some(), "Debate session should be preserved");

            // Verify JSON serialization/deserialization consistency
            let loaded_consensus = &loaded_record.consensus_result;
            assert_eq!(loaded_consensus.final_verdict["type"], "approved");
            assert_eq!(loaded_consensus.final_verdict["confidence"], 0.95);

            // Clean up
            test_db.cleanup().await?;

            Ok(())
        }

        #[tokio::test]
        async fn test_council_verdict_performance_and_load() -> Result<()> {
            let test_db = TestDatabase::new().await?;
            let storage = DatabaseVerdictStorage::new(Arc::new(test_db.client));

            let task_id = TaskId(Uuid::new_v4());

            // Performance test: Store multiple verdicts quickly
            let start_time = std::time::Instant::now();
            let mut verdict_ids = Vec::new();

            for i in 0..100 {
                let verdict_id = VerdictId(Uuid::parse_str(&test_db.unique_id(&format!("perf_verdict_{}", i))).unwrap());
                verdict_ids.push(verdict_id);

                let consensus_result = create_test_consensus_result(task_id, verdict_id);
                let verdict_record = VerdictRecord {
                    verdict_id,
                    consensus_result,
                    debate_session: if i % 10 == 0 { Some(create_test_debate_session()) } else { None },
                    created_at: chrono::Utc::now(),
                    accessed_at: chrono::Utc::now(),
                    access_count: i as u64,
                    storage_location: Some(format!("perf_test_{}", i)),
                };

                storage.store_verdict(&verdict_record).await?;
            }

            let store_duration = start_time.elapsed();
            println!("Stored 100 verdicts in {:?}", store_duration);

            // Load performance test
            let load_start = std::time::Instant::now();
            let mut load_count = 0;

            for verdict_id in &verdict_ids {
                let loaded = storage.load_verdict(*verdict_id).await?;
                assert!(loaded.is_some(), "Verdict should exist");
                load_count += 1;
            }

            let load_duration = load_start.elapsed();
            println!("Loaded {} verdicts in {:?}", load_count, load_duration);

            // Task-based loading performance test
            let task_load_start = std::time::Instant::now();
            let task_verdicts = storage.load_verdicts_by_task(task_id).await?;
            let task_load_duration = task_load_start.elapsed();

            println!("Loaded {} verdicts by task in {:?}", task_verdicts.len(), task_load_duration);
            assert_eq!(task_verdicts.len(), 100, "Should load all verdicts for the task");

            // Time range performance test
            let time_start = chrono::Utc::now() - chrono::Duration::hours(1);
            let time_end = chrono::Utc::now() + chrono::Duration::hours(1);

            let time_load_start = std::time::Instant::now();
            let time_verdicts = storage.load_verdicts_by_time_range(time_start, time_end).await?;
            let time_load_duration = time_load_start.elapsed();

            println!("Loaded {} verdicts by time range in {:?}", time_verdicts.len(), time_load_duration);

            // Statistics performance test
            let stats_start = std::time::Instant::now();
            let stats = storage.get_storage_stats().await?;
            let stats_duration = stats_start.elapsed();

            println!("Retrieved storage stats in {:?}", stats_duration);
            assert_eq!(stats.total_verdicts, 100, "Should have 100 total verdicts");
            assert!(stats.total_debates >= 10, "Should have at least 10 debates");

            // Performance assertions
            assert!(store_duration.as_millis() < 5000, "Storing 100 verdicts should take less than 5 seconds");
            assert!(load_duration.as_millis() < 3000, "Loading 100 verdicts should take less than 3 seconds");
            assert!(task_load_duration.as_millis() < 1000, "Task-based loading should be fast");
            assert!(stats_duration.as_millis() < 500, "Statistics query should be fast");

            // Cleanup
            test_db.cleanup().await?;

            Ok(())
        }
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
