//! Response caching for judge evaluations

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

// Import types from judge_types module
use super::judge_types::{JudgeVerdict, RiskAssessment, RiskLevel, RiskFactor, RiskFactorType, RiskSeverity};

/// Cache entry for judge responses
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub verdict: JudgeVerdict,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl_seconds: u64,
    pub spec_hash: String,
}

impl CacheEntry {
    pub fn new(
        verdict: JudgeVerdict,
        ttl_seconds: u64,
        spec_hash: String,
    ) -> Self {
        Self {
            verdict,
            timestamp: chrono::Utc::now(),
            ttl_seconds,
            spec_hash,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        age.num_seconds() as u64 >= self.ttl_seconds
    }

    pub fn time_to_live_seconds(&self) -> i64 {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        let ttl = self.ttl_seconds as i64;
        let age_seconds = age.num_seconds();
        ttl.saturating_sub(age_seconds)
    }
}

/// Response cache for judge evaluations
#[derive(Debug)]
pub struct ResponseCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    default_ttl_seconds: u64,
    max_entries: usize,
}

impl ResponseCache {
    pub fn new(default_ttl_seconds: u64, max_entries: usize) -> Self {
        Self {
            cache: Arc::new(RwLock::new(HashMap::new())),
            default_ttl_seconds,
            max_entries,
        }
    }

    /// Get cached verdict for a specification
    pub async fn get(&self, spec_id: Uuid, title: &str, description: &str) -> Option<JudgeVerdict> {
        let cache_key = self.generate_cache_key(spec_id, title, description);

        let cache = self.cache.read().await;
        if let Some(entry) = cache.get(&cache_key) {
            if !entry.is_expired() {
                return Some(entry.verdict.clone());
            }
        }

        None
    }

    /// Store verdict in cache
    pub async fn put(
        &self,
        spec_id: Uuid,
        title: &str,
        description: &str,
        verdict: JudgeVerdict,
        ttl_seconds: Option<u64>,
    ) {
        let cache_key = self.generate_cache_key(spec_id, title, description);
        let ttl = ttl_seconds.unwrap_or(self.default_ttl_seconds);

        // Create spec hash for content-based caching
        let spec_hash = self.generate_spec_hash(title, description);

        let entry = CacheEntry::new(verdict, ttl, spec_hash);

        let mut cache = self.cache.write().await;

        // Evict if at capacity (simple LRU-like behavior)
        if cache.len() >= self.max_entries {
            // Remove expired entries first
            let expired_keys: Vec<String> = cache.iter()
                .filter(|(_, entry)| entry.is_expired())
                .map(|(key, _)| key.clone())
                .collect();

            for key in expired_keys {
                cache.remove(&key);
            }

            // If still at capacity, remove oldest entry
            if cache.len() >= self.max_entries {
                if let Some(oldest_key) = cache.iter()
                    .min_by_key(|(_, entry)| entry.timestamp)
                    .map(|(key, _)| key.clone()) {
                    cache.remove(&oldest_key);
                }
            }
        }

        cache.insert(cache_key, entry);
    }

    /// Check if specification is cached
    pub async fn is_cached(&self, spec_id: Uuid, title: &str, description: &str) -> bool {
        let cache_key = self.generate_cache_key(spec_id, title, description);
        let cache = self.cache.read().await;
        cache.get(&cache_key).map_or(false, |entry| !entry.is_expired())
    }

    /// Get cache statistics
    pub async fn stats(&self) -> CacheStats {
        let cache = self.cache.read().await;

        let total_entries = cache.len();
        let expired_entries = cache.values().filter(|entry| entry.is_expired()).count();
        let active_entries = total_entries - expired_entries;

        let oldest_entry = cache.values()
            .min_by_key(|entry| entry.timestamp)
            .map(|entry| entry.timestamp);

        let newest_entry = cache.values()
            .max_by_key(|entry| entry.timestamp)
            .map(|entry| entry.timestamp);

        CacheStats {
            total_entries,
            active_entries,
            expired_entries,
            oldest_entry,
            newest_entry,
        }
    }

    /// Clear all cache entries
    pub async fn clear(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
    }

    /// Clean expired entries
    pub async fn clean_expired(&self) {
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        let final_count = cache.len();

        if initial_count != final_count {
            tracing::debug!("Cleaned {} expired cache entries", initial_count - final_count);
        }
    }

    /// Generate cache key from specification components
    fn generate_cache_key(&self, spec_id: Uuid, title: &str, description: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        spec_id.hash(&mut hasher);
        title.hash(&mut hasher);
        description.hash(&mut hasher);

        format!("judge_{:x}", hasher.finish())
    }

    /// Generate content hash for specification
    fn generate_spec_hash(&self, title: &str, description: &str) -> String {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        title.hash(&mut hasher);
        description.hash(&mut hasher);

        format!("{:x}", hasher.finish())
    }
}

impl Default for ResponseCache {
    fn default() -> Self {
        Self::new(3600, 1000) // 1 hour TTL, 1000 max entries
    }
}

/// Cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub total_entries: usize,
    pub active_entries: usize,
    pub expired_entries: usize,
    pub oldest_entry: Option<chrono::DateTime<chrono::Utc>>,
    pub newest_entry: Option<chrono::DateTime<chrono::Utc>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::judge_types::JudgeVerdict;

    #[tokio::test]
    async fn test_cache_hit() {
        let cache = ResponseCache::new(300, 100);
        let spec_id = Uuid::new_v4();
        let verdict = JudgeVerdict::Approve {
            confidence: 0.9,
            reasoning: "Test approval".to_string(),
            quality_score: 0.85,
            risk_assessment: Default::default(),
        };

        // Store verdict
        cache.put(spec_id, "Test Title", "Test Description", verdict.clone(), None).await;

        // Retrieve verdict
        let cached = cache.get(spec_id, "Test Title", "Test Description").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap(), verdict);
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = ResponseCache::new(300, 100);
        let spec_id = Uuid::new_v4();

        let cached = cache.get(spec_id, "Test Title", "Test Description").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        let cache = ResponseCache::new(0, 100); // 0 TTL = immediate expiration
        let spec_id = Uuid::new_v4();
        let verdict = JudgeVerdict::Approve {
            confidence: 0.9,
            reasoning: "Test approval".to_string(),
            quality_score: 0.85,
            risk_assessment: Default::default(),
        };

        // Store verdict
        cache.put(spec_id, "Test Title", "Test Description", verdict, None).await;

        // Should be expired immediately
        let cached = cache.get(spec_id, "Test Title", "Test Description").await;
        assert!(cached.is_none());
    }

    #[tokio::test]
    async fn test_cache_cleanup() {
        let cache = ResponseCache::new(0, 100);
        let spec_id = Uuid::new_v4();
        let verdict = JudgeVerdict::Approve {
            confidence: 0.9,
            reasoning: "Test approval".to_string(),
            quality_score: 0.85,
            risk_assessment: Default::default(),
        };

        // Store verdict
        cache.put(spec_id, "Test Title", "Test Description", verdict, None).await;

        // Clean expired entries
        cache.clean_expired().await;

        // Check stats
        let stats = cache.stats().await;
        assert_eq!(stats.expired_entries, 0); // Should be cleaned
    }
}
