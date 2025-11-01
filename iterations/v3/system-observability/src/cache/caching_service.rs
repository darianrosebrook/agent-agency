//! High-level caching service for inference result caching
//!
//! Provides a service layer around CacheBackend for managing inference result caching
//! with automatic serialization, key generation, and cache invalidation.

use async_trait::async_trait;
use std::sync::Arc;
use std::time::Duration;
use serde::{Deserialize, Serialize};
use sha2::{Sha256, Digest};
use crate::cache::{CacheBackend, CacheError};

/// High-level caching service for inference results
#[async_trait]
pub trait CachingService: Send + Sync {
    /// Cache an inference result
    async fn cache_inference_result(
        &self,
        cache_key: &InferenceCacheKey,
        result: &InferenceCacheValue,
        ttl: Option<Duration>,
    ) -> Result<(), CachingServiceError>;

    /// Retrieve a cached inference result
    async fn get_cached_result(
        &self,
        cache_key: &InferenceCacheKey,
    ) -> Result<Option<InferenceCacheValue>, CachingServiceError>;

    /// Invalidate cached results for a model
    async fn invalidate_model_cache(&self, model_id: &str) -> Result<usize, CachingServiceError>;

    /// Clear all cached results
    async fn clear_cache(&self) -> Result<(), CachingServiceError>;

    /// Get cache statistics
    async fn get_cache_stats(&self) -> Result<CacheStats, CachingServiceError>;
}

/// Cache key for inference results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceCacheKey {
    /// Model identifier
    pub model_id: String,
    
    /// Input prompt or data hash
    pub input_hash: String,
    
    /// Parameter signature (temperature, top_p, etc.)
    pub parameter_signature: String,
}

impl InferenceCacheKey {
    /// Generate a cache key from model and input
    pub fn from_input(model_id: &str, input: &str, parameters: &serde_json::Value) -> Self {
        // Hash the input for consistent key generation
        let mut hasher = Sha256::new();
        hasher.update(input.as_bytes());
        let input_hash = format!("{:x}", hasher.finalize());

        // Create parameter signature
        let parameter_signature = serde_json::to_string(parameters)
            .unwrap_or_else(|_| "default".to_string());

        Self {
            model_id: model_id.to_string(),
            input_hash,
            parameter_signature,
        }
    }

    /// Generate cache key string
    pub fn to_key_string(&self) -> String {
        format!("inference:{}:{}:{}", self.model_id, self.input_hash, self.parameter_signature)
    }
}

/// Cached inference result value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InferenceCacheValue {
    /// Cached output data
    pub output: serde_json::Value,
    
    /// Metadata about the cached result
    pub metadata: CacheMetadata,
    
    /// Cached timestamp
    pub cached_at: chrono::DateTime<chrono::Utc>,
}

/// Metadata about cached result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMetadata {
    /// Model version used
    pub model_version: String,
    
    /// Cache hit count
    pub hit_count: u64,
    
    /// Original inference latency (ms)
    pub original_latency_ms: u64,
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    /// Total cache entries
    pub total_entries: usize,
    
    /// Total cache hits
    pub total_hits: u64,
    
    /// Total cache misses
    pub total_misses: u64,
    
    /// Hit rate (0.0-1.0)
    pub hit_rate: f64,
    
    /// Average cache age (seconds)
    pub avg_age_seconds: f64,
}

/// Caching service implementation using CacheBackend
pub struct DefaultCachingService {
    /// Cache backend
    cache_backend: Arc<dyn CacheBackend>,
    
    /// Default TTL for cached results
    default_ttl: Duration,
    
    /// Cache statistics
    stats: Arc<tokio::sync::RwLock<CacheStats>>,
}

impl DefaultCachingService {
    /// Create a new caching service
    pub fn new(cache_backend: Arc<dyn CacheBackend>, default_ttl: Duration) -> Self {
        Self {
            cache_backend,
            default_ttl,
            stats: Arc::new(tokio::sync::RwLock::new(CacheStats {
                total_entries: 0,
                total_hits: 0,
                total_misses: 0,
                hit_rate: 0.0,
                avg_age_seconds: 0.0,
            })),
        }
    }
}

#[async_trait]
impl CachingService for DefaultCachingService {
    async fn cache_inference_result(
        &self,
        cache_key: &InferenceCacheKey,
        result: &InferenceCacheValue,
        ttl: Option<Duration>,
    ) -> Result<(), CachingServiceError> {
        let key = cache_key.to_key_string();
        let value = serde_json::to_string(result)
            .map_err(|e| CachingServiceError::SerializationError(e.to_string()))?;

        self.cache_backend
            .set(&key, &value, Some(ttl.unwrap_or(self.default_ttl)))
            .await
            .map_err(|e| CachingServiceError::CacheError(e))?;

        // Update stats
        let mut stats = self.stats.write().await;
        stats.total_entries += 1;

        Ok(())
    }

    async fn get_cached_result(
        &self,
        cache_key: &InferenceCacheKey,
    ) -> Result<Option<InferenceCacheValue>, CachingServiceError> {
        let key = cache_key.to_key_string();
        
        match self.cache_backend.get(&key).await {
            Ok(Some(value_str)) => {
                // Update hit stats
                let mut stats = self.stats.write().await;
                stats.total_hits += 1;
                stats.hit_rate = if stats.total_hits + stats.total_misses > 0 {
                    stats.total_hits as f64 / (stats.total_hits + stats.total_misses) as f64
                } else {
                    0.0
                };

                // Deserialize cached value
                let cached_value: InferenceCacheValue = serde_json::from_str(&value_str)
                    .map_err(|e| CachingServiceError::DeserializationError(e.to_string()))?;

                Ok(Some(cached_value))
            }
            Ok(None) => {
                // Update miss stats
                let mut stats = self.stats.write().await;
                stats.total_misses += 1;
                stats.hit_rate = if stats.total_hits + stats.total_misses > 0 {
                    stats.total_hits as f64 / (stats.total_hits + stats.total_misses) as f64
                } else {
                    0.0
                };

                Ok(None)
            }
            Err(e) => Err(CachingServiceError::CacheError(e)),
        }
    }

    async fn invalidate_model_cache(&self, model_id: &str) -> Result<usize, CachingServiceError> {
        // PLACEHOLDER: In a real implementation, this would:
        // 1. List all keys matching pattern "inference:{model_id}:*"
        // 2. Delete each matching key
        // 3. Return count of deleted keys
        
        // For now, return 0 as we don't have pattern-based deletion
        Ok(0)
    }

    async fn clear_cache(&self) -> Result<(), CachingServiceError> {
        // PLACEHOLDER: In a real implementation, this would:
        // 1. List all keys matching pattern "inference:*"
        // 2. Delete all matching keys
        
        // Reset stats
        let mut stats = self.stats.write().await;
        stats.total_entries = 0;
        stats.total_hits = 0;
        stats.total_misses = 0;
        stats.hit_rate = 0.0;

        Ok(())
    }

    async fn get_cache_stats(&self) -> Result<CacheStats, CachingServiceError> {
        let stats = self.stats.read().await;
        Ok(stats.clone())
    }
}

/// Caching service errors
#[derive(Debug, thiserror::Error)]
pub enum CachingServiceError {
    #[error("Cache backend error: {0}")]
    CacheError(CacheError),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Deserialization error: {0}")]
    DeserializationError(String),

    #[error("Invalid cache key: {0}")]
    InvalidKey(String),
}


