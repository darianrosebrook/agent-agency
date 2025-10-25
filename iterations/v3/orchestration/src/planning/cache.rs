//! LLM Response Caching for Performance Optimization

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Cache entry for LLM responses
#[derive(Debug, Clone)]
pub struct LLMCacheEntry {
    pub response: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub ttl_seconds: u64,
}

impl LLMCacheEntry {
    pub fn new(response: String, ttl_seconds: u64) -> Self {
        Self {
            response,
            timestamp: chrono::Utc::now(),
            ttl_seconds,
        }
    }

    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now();
        let age = now.signed_duration_since(self.timestamp);
        age.num_seconds() as u64 >= self.ttl_seconds
    }
}

/// Optimized LLM Client with caching capabilities
#[derive(Clone)]
pub struct CachedLLMClient {
    inner: Arc<dyn super::llm_client::LLMClient>,
    cache: Arc<RwLock<HashMap<String, LLMCacheEntry>>>,
    cache_ttl_seconds: u64,
}

impl CachedLLMClient {
    pub fn new(inner: Arc<dyn super::llm_client::LLMClient>, cache_ttl_seconds: u64) -> Self {
        Self {
            inner,
            cache: Arc::new(RwLock::new(HashMap::new())),
            cache_ttl_seconds,
        }
    }

    /// Generate response with caching optimization
    pub async fn generate_cached(&self, prompt: &str) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        // Create cache key from prompt (using hash for privacy)
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        prompt.hash(&mut hasher);
        let cache_key = format!("llm_{:x}", hasher.finish());

        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(&cache_key) {
                if !entry.is_expired() {
                    tracing::debug!("Cache hit for prompt hash: {}", &cache_key[..8]);
                    return Ok(entry.response.clone());
                } else {
                    tracing::debug!("Cache expired for prompt hash: {}", &cache_key[..8]);
                }
            }
        }

        // Cache miss or expired - generate new response
        tracing::debug!("Cache miss for prompt hash: {} - generating new response", &cache_key[..8]);

        let messages = vec![
            super::llm_client::Message {
                role: super::llm_client::MessageRole::User,
                content: prompt.to_string(),
            }
        ];

        let request = super::llm_client::GenerationRequest {
            messages,
            temperature: Some(0.7),
            max_tokens: Some(1000),
            stop_sequences: None,
        };

        let response = self.inner.generate(request).await?;

        // Cache the response
        let cache_entry = LLMCacheEntry::new(response.clone(), self.cache_ttl_seconds);
        {
            let mut cache = self.cache.write().await;
            cache.insert(cache_key, cache_entry);
        }

        Ok(response)
    }

    /// Get cache statistics for monitoring
    pub async fn cache_stats(&self) -> (usize, usize) {
        let cache = self.cache.read().await;
        let total_entries = cache.len();
        let expired_entries = cache.values().filter(|entry| entry.is_expired()).count();
        (total_entries, expired_entries)
    }

    /// Clear expired cache entries
    pub async fn clean_expired(&self) {
        let mut cache = self.cache.write().await;
        let initial_count = cache.len();
        cache.retain(|_, entry| !entry.is_expired());
        let final_count = cache.len();

        if initial_count != final_count {
            tracing::debug!("Cleaned {} expired cache entries", initial_count - final_count);
        }
    }

    /// Clear all cache entries
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        let cleared_count = cache.len();
        cache.clear();
        tracing::debug!("Cleared {} cache entries", cleared_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_hit() {
        // Test cache hit functionality
        let mock_client = Arc::new(MockLLMClient::new("test response"));
        let cached_client = CachedLLMClient::new(mock_client, 300);

        // First call should cache
        let result1 = cached_client.generate_cached("test prompt").await.unwrap();
        assert_eq!(result1, "test response");

        // Second call should use cache
        let result2 = cached_client.generate_cached("test prompt").await.unwrap();
        assert_eq!(result2, "test response");
    }

    #[tokio::test]
    async fn test_cache_expiration() {
        // Test cache expiration
        let mock_client = Arc::new(MockLLMClient::new("fresh response"));
        let cached_client = CachedLLMClient::new(mock_client, 0); // 0 TTL = immediate expiration

        // First call
        let result1 = cached_client.generate_cached("test prompt").await.unwrap();

        // Second call should generate new response due to expiration
        let result2 = cached_client.generate_cached("test prompt").await.unwrap();

        assert_eq!(result1, "fresh response");
        assert_eq!(result2, "fresh response");
    }

    // Mock LLM client for testing
    struct MockLLMClient {
        response: String,
    }

    impl MockLLMClient {
        fn new(response: &str) -> Self {
            Self {
                response: response.to_string(),
            }
        }
    }

    #[async_trait::async_trait]
    impl super::llm_client::LLMClient for MockLLMClient {
        async fn generate(&self, _request: super::llm_client::GenerationRequest) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
            Ok(self.response.clone())
        }
    }
}
