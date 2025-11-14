//! Vector Search Cache Management
//!
//! Handles LRU caching and persistent cache storage for vector search operations.

use crate::research_types::*;
use anyhow::{Context, Result};
use lru::LruCache;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};
use tracing::debug;

use super::vector_metrics::VectorSearchMetrics;

const PERSISTENT_CACHE_ENV_KEY: &str = "AA_VECTOR_CACHE_DIR";
const PERSISTENT_CACHE_LIMIT_ENV_KEY: &str = "AA_VECTOR_CACHE_LIMIT";
const DEFAULT_PERSISTENT_CACHE_DIR: &str = "cache/vector_search";
const DEFAULT_PERSISTENT_CACHE_LIMIT: usize = 10_000;

/// Persistent embedding record for disk storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersistentEmbeddingRecord {
    pub embedding: Vec<f32>,
    pub last_updated: i64,
}

/// Cache manager for vector search operations
pub struct CacheManager {
    search_cache: RwLock<LruCache<String, Vec<KnowledgeEntry>>>,
    embedding_cache: RwLock<LruCache<String, Vec<f32>>>,
    persistent_cache_dir: PathBuf,
    persistent_cache_lock: Arc<Mutex<()>>,
    metrics: Arc<RwLock<VectorSearchMetrics>>,
}

impl CacheManager {
    /// Create a new cache manager with default settings
    pub async fn new(search_cache_size: usize, embedding_cache_size: usize) -> Result<Self> {
        let persistent_cache_dir = Self::resolve_persistent_cache_dir();
        Self::new_with_cache_dir(
            search_cache_size,
            embedding_cache_size,
            persistent_cache_dir,
        )
        .await
    }

    /// Create a new cache manager with custom cache directory
    pub async fn new_with_cache_dir(
        search_cache_size: usize,
        embedding_cache_size: usize,
        cache_dir: impl Into<PathBuf>,
    ) -> Result<Self> {
        let persistent_cache_dir = cache_dir.into();

        // Ensure cache directory exists
        tokio::fs::create_dir_all(&persistent_cache_dir)
            .await
            .unwrap_or_else(|e| debug!("Failed to create cache directory: {}", e));

        Ok(Self {
            search_cache: RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(search_cache_size).unwrap(),
            )),
            embedding_cache: RwLock::new(LruCache::new(
                std::num::NonZeroUsize::new(embedding_cache_size).unwrap(),
            )),
            persistent_cache_dir,
            persistent_cache_lock: Arc::new(Mutex::new(())),
            metrics: Arc::new(RwLock::new(VectorSearchMetrics::default())),
        })
    }

    /// Get cached search results
    pub async fn get_search_cache(&self, key: &str) -> Option<Vec<KnowledgeEntry>> {
        let mut cache = self.search_cache.write().await;
        cache.get(key).cloned()
    }

    /// Put search results in cache
    pub async fn put_search_cache(&self, key: String, results: Vec<KnowledgeEntry>) {
        let mut cache = self.search_cache.write().await;
        cache.put(key, results);
    }

    /// Get cached embedding
    pub async fn get_embedding_cache(&self, key: &str) -> Option<Vec<f32>> {
        let mut cache = self.embedding_cache.write().await;
        cache.get(key).cloned()
    }

    /// Put embedding in cache
    pub async fn put_embedding_cache(&self, key: String, embedding: Vec<f32>) {
        let mut cache = self.embedding_cache.write().await;
        cache.put(key, embedding);
    }

    /// Clear all caches
    pub async fn clear_all_caches(&self) -> Result<()> {
        let mut search_cache = self.search_cache.write().await;
        let mut embedding_cache = self.embedding_cache.write().await;

        search_cache.clear();
        embedding_cache.clear();

        // Also clear persistent cache
        let _lock = self.persistent_cache_lock.lock().await;
        let cache_file = self.cache_file_path();
        if cache_file.exists() {
            tokio::fs::remove_file(&cache_file).await?;
        }

        Ok(())
    }

    /// Get cache statistics
    pub async fn get_cache_stats(&self) -> (usize, usize) {
        let search_cache = self.search_cache.read().await;
        let embedding_cache = self.embedding_cache.read().await;
        (search_cache.len(), embedding_cache.len())
    }

    /// Store embedding in persistent cache
    pub async fn store_embedding_persistent(
        &self,
        text_hash: String,
        embedding: Vec<f32>,
    ) -> Result<()> {
        let _lock = self.persistent_cache_lock.lock().await;

        let mut persistent_cache = self.read_persistent_cache().await?;
        let record = PersistentEmbeddingRecord {
            embedding,
            last_updated: chrono::Utc::now().timestamp(),
        };

        persistent_cache.insert(text_hash, record);
        self.write_persistent_cache(&persistent_cache).await
    }

    /// Retrieve embedding from persistent cache
    pub async fn retrieve_embedding_persistent(&self, text_hash: &str) -> Result<Option<Vec<f32>>> {
        let _lock = self.persistent_cache_lock.lock().await;

        let persistent_cache = self.read_persistent_cache().await?;
        Ok(persistent_cache
            .get(text_hash)
            .map(|record| record.embedding.clone()))
    }

    fn resolve_persistent_cache_dir() -> PathBuf {
        std::env::var(PERSISTENT_CACHE_ENV_KEY)
            .ok()
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from(DEFAULT_PERSISTENT_CACHE_DIR))
    }

    fn cache_file_path(&self) -> PathBuf {
        // TODO: Implement comprehensive collection-specific cache file path parameterization
        //       Currently uses generic name; should implement comprehensive parameterization that accepts collection name as parameter, generates collection-specific cache file paths, and supports multiple concurrent collections with proper name sanitization.
        //
        // COMPLETION CHECKLIST:
        // [ ] Primary functionality implemented
        // [ ] API/data structures defined & stable
        // [ ] Error handling + validation aligned with error taxonomy
        // [ ] Tests: Unit ≥80% branch coverage (≥50% mutation if enabled)
        // [ ] Integration tests for external systems/contracts
        // [ ] Documentation: public API + system behavior
        // [ ] Performance/profiled against SLA (CPU/mem/latency throughput)
        // [ ] Security posture reviewed (inputs, authz, sandboxing)
        // [ ] Observability: logs (debug), metrics (SLO-aligned), tracing
        // [ ] Configurability and feature flags defined if relevant
        // [ ] Failure-mode cards documented (degradation paths)
        //
        // ACCEPTANCE CRITERIA:
        // - Collection name is accepted as parameter
        // - Collection-specific cache file paths are generated
        // - Multiple concurrent collections are supported
        // - Collection name sanitization handles file system constraints
        //
        // DEPENDENCIES:
        // - Collection name parameter handling (Required)
        // - File path generation utilities (Required)
        // - Name sanitization utilities (Required)
        //
        // ESTIMATED EFFORT: 6-8 hours (medium confidence)
        // PRIORITY: Medium
        // BLOCKING: No
        //
        // GOVERNANCE:
        // - CAWS Tier: 2 (cache file management functionality)
        // - Change Budget: ~150 LOC
        // - Reviewer Requirements: File system and cache management expertise
        let file_name = "vector_search_embeddings.json".to_string();
        self.persistent_cache_dir.join(file_name)
    }

    fn persistent_cache_limit(&self) -> usize {
        std::env::var(PERSISTENT_CACHE_LIMIT_ENV_KEY)
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|limit| *limit > 0)
            .unwrap_or(DEFAULT_PERSISTENT_CACHE_LIMIT)
    }

    async fn read_persistent_cache(&self) -> Result<HashMap<String, PersistentEmbeddingRecord>> {
        let path = self.cache_file_path();
        match tokio::fs::read(&path).await {
            Ok(bytes) if !bytes.is_empty() => {
                let cache =
                    serde_json::from_slice::<HashMap<String, PersistentEmbeddingRecord>>(&bytes)
                        .context("Failed to deserialize persistent embedding cache")?;
                Ok(cache)
            }
            Ok(_) => Ok(HashMap::new()),
            Err(err) if err.kind() == ErrorKind::NotFound => Ok(HashMap::new()),
            Err(err) => Err(err.into()),
        }
    }

    async fn write_persistent_cache(
        &self,
        cache: &HashMap<String, PersistentEmbeddingRecord>,
    ) -> Result<()> {
        let path = self.cache_file_path();
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }

        let serialized =
            serde_json::to_vec(cache).context("Failed to serialize persistent embedding cache")?;
        let tmp_path = path.with_extension("tmp");

        tokio::fs::write(&tmp_path, &serialized).await?;
        tokio::fs::rename(&tmp_path, &path).await?;
        Ok(())
    }

    fn prune_persistent_cache(&self, cache: &mut HashMap<String, PersistentEmbeddingRecord>) {
        let limit = self.persistent_cache_limit();
        if cache.len() <= limit {
            return;
        }

        let mut entries: Vec<_> = cache
            .iter()
            .map(|(key, record)| (key.clone(), record.last_updated))
            .collect();
        entries.sort_by_key(|(_, timestamp)| *timestamp);

        let remove_count = cache.len() - limit;
        for (key, _) in entries.into_iter().take(remove_count) {
            cache.remove(&key);
        }
    }
}
