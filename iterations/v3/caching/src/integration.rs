//! Simple cache integration utilities
//!
//! Provides basic cache integration for common patterns.

use super::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use tracing::{debug, info};

/// Simple API response cache
pub struct ApiResponseCache {
    cache: SimpleMemoryCache<String>,
}

impl ApiResponseCache {
    /// Create a new API response cache
    pub fn new() -> Self {
        Self {
            cache: SimpleMemoryCache::new(CacheConfig::default()),
        }
    }

    /// Get a cached response
    pub async fn get(&self, key: &str) -> CacheResult<Option<String>> {
        self.cache.get(key).await
    }

    /// Cache a response
    pub async fn set(&self, key: String, response: String) -> CacheResult<()> {
        self.cache.set(key, response).await
    }
}

impl Default for ApiResponseCache {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple key generation utilities
pub struct CacheKey;

impl CacheKey {
    /// Generate a cache key from a string
    pub fn from_string(s: &str) -> String {
        let mut hasher = DefaultHasher::new();
        s.hash(&mut hasher);
        format!("{:x}", hasher.finish())
    }

    /// Generate a cache key from multiple components
    pub fn from_components(components: &[&str]) -> String {
        let combined = components.join(":");
        Self::from_string(&combined)
    }
}
