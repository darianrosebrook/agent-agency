//! Cache integration utilities for application components
//!
//! Provides ready-to-use cache integrations for common patterns:
//! - API response caching
//! - Database query result caching
//! - Computation result caching
//! - LLM response caching

use super::*;
use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use sqlparser::ast::{Query, Statement, TableWithJoins, TableFactor};
use sqlparser::dialect::PostgreSqlDialect;
use sqlparser::parser::Parser;
use once_cell::sync::Lazy;
use tracing::{debug, warn, info, error};

/// API response cache integration
pub struct ApiResponseCache {
    cache: Arc<dyn Cache<String, serde_json::Value> + Send + Sync>,
    config: CacheConfig,
}

impl ApiResponseCache {
    pub fn new(cache: Arc<dyn Cache<String, serde_json::Value> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for API request
    pub fn generate_key(&self, method: &str, path: &str, query: Option<&str>, body_hash: Option<u64>) -> String {
        let mut hasher = DefaultHasher::new();
        method.hash(&mut hasher);
        path.hash(&mut hasher);

        if let Some(q) = query {
            q.hash(&mut hasher);
        }

        if let Some(bh) = body_hash {
            bh.hash(&mut hasher);
        }

        format!("api:{}:{:x}", path, hasher.finish())
    }

    /// Cache API response
    pub async fn cache_response(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body_hash: Option<u64>,
        response: serde_json::Value,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(method, path, query, body_hash);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, response, ttl).await
    }

    /// Get cached API response
    pub async fn get_cached_response(
        &self,
        method: &str,
        path: &str,
        query: Option<&str>,
        body_hash: Option<u64>
    ) -> CacheResult<serde_json::Value> {
        let key = self.generate_key(method, path, query, body_hash);
        self.cache.get(&key).await
    }
}

/// Database query result cache
pub struct DatabaseQueryCache {
    cache: Arc<dyn Cache<String, Vec<serde_json::Value>> + Send + Sync>,
    config: CacheConfig,
}

impl DatabaseQueryCache {
    pub fn new(cache: Arc<dyn Cache<String, Vec<serde_json::Value>> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for database query
    pub fn generate_key(&self, query: &str, params: &[serde_json::Value]) -> String {
        let mut hasher = DefaultHasher::new();
        query.hash(&mut hasher);

        for param in params {
            param.to_string().hash(&mut hasher);
        }

        format!("db:{:x}", hasher.finish())
    }

    /// Cache query results
    pub async fn cache_results(
        &self,
        query: &str,
        params: &[serde_json::Value],
        results: Vec<serde_json::Value>,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(query, params);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, results, ttl).await
    }

    /// Get cached query results
    pub async fn get_cached_results(
        &self,
        query: &str,
        params: &[serde_json::Value]
    ) -> CacheResult<Vec<serde_json::Value>> {
        let key = self.generate_key(query, params);
        self.cache.get(&key).await
    }

    /// Invalidate query cache by table name
    pub async fn invalidate_by_table(&self, table_name: &str) -> CacheResult<()> {
        // TODO: Implement proper query-to-table mapping for cache invalidation
        // - Create query AST parsing and analysis
        // - Build table dependency tracking system
        // - Implement automatic cache invalidation on schema changes
        // - Add support for complex queries with JOINs and subqueries
        // - Implement selective cache invalidation strategies
        // - Add cache invalidation metrics and monitoring
        // PLACEHOLDER: Using simplified table-based invalidation
        warn!("Table-based cache invalidation not fully implemented for: {}", table_name);
        Ok(())
    }
}

/// LLM response cache for expensive API calls
pub struct LlmResponseCache {
    cache: Arc<dyn Cache<String, String> + Send + Sync>,
    config: CacheConfig,
}

impl LlmResponseCache {
    pub fn new(cache: Arc<dyn Cache<String, String> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for LLM request
    pub fn generate_key(&self, model: &str, prompt: &str, temperature: Option<f32>, max_tokens: Option<u32>) -> String {
        let mut hasher = DefaultHasher::new();
        model.hash(&mut hasher);
        prompt.hash(&mut hasher);

        if let Some(temp) = temperature {
            (temp.to_bits() as u64).hash(&mut hasher);
        }

        if let Some(tokens) = max_tokens {
            tokens.hash(&mut hasher);
        }

        format!("llm:{}:{:x}", model, hasher.finish())
    }

    /// Cache LLM response
    pub async fn cache_response(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>,
        response: String,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(model, prompt, temperature, max_tokens);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, response, ttl).await
    }

    /// Get cached LLM response
    pub async fn get_cached_response(
        &self,
        model: &str,
        prompt: &str,
        temperature: Option<f32>,
        max_tokens: Option<u32>
    ) -> CacheResult<String> {
        let key = self.generate_key(model, prompt, temperature, max_tokens);
        self.cache.get(&key).await
    }

    /// Invalidate LLM cache by model
    pub async fn invalidate_by_model(&self, model: &str) -> CacheResult<()> {
        // This would require maintaining a reverse index of model to keys
        // For now, this is a placeholder
        warn!("Model-based cache invalidation not fully implemented for: {}", model);
        Ok(())
    }
}

/// Computation result cache for expensive calculations
pub struct ComputationCache<T> {
    cache: Arc<dyn Cache<String, T> + Send + Sync>,
    config: CacheConfig,
}

impl<T> ComputationCache<T>
where
    T: serde::Serialize + for<'de> serde::Deserialize<'de> + Clone + Send + Sync + 'static,
{
    pub fn new(cache: Arc<dyn Cache<String, T> + Send + Sync>, config: CacheConfig) -> Self {
        Self { cache, config }
    }

    /// Generate cache key for computation
    pub fn generate_key(&self, function_name: &str, inputs: &[serde_json::Value]) -> String {
        let mut hasher = DefaultHasher::new();
        function_name.hash(&mut hasher);

        for input in inputs {
            input.to_string().hash(&mut hasher);
        }

        format!("comp:{}:{:x}", function_name, hasher.finish())
    }

    /// Cache computation result
    pub async fn cache_result(
        &self,
        function_name: &str,
        inputs: &[serde_json::Value],
        result: T,
        ttl_seconds: Option<u64>
    ) -> CacheResult<()> {
        let key = self.generate_key(function_name, inputs);
        let ttl = ttl_seconds
            .or(Some(self.config.default_ttl_seconds))
            .map(Duration::from_secs);

        self.cache.set(key, result, ttl).await
    }

    /// Get cached computation result
    pub async fn get_cached_result(
        &self,
        function_name: &str,
        inputs: &[serde_json::Value]
    ) -> CacheResult<T> {
        let key = self.generate_key(function_name, inputs);
        self.cache.get(&key).await
    }

    /// Compute with caching (cache-aside pattern)
    pub async fn compute_with_cache<F, Fut>(
        &self,
        function_name: &str,
        inputs: &[serde_json::Value],
        computation: F,
        ttl_seconds: Option<u64>
    ) -> CacheResult<T>
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = CacheResult<T>>,
    {
        // Try cache first
        match self.get_cached_result(function_name, inputs).await {
            Ok(result) => {
                debug!("Cache hit for computation: {}", function_name);
                Ok(result)
            }
            Err(CacheError::Miss { .. }) => {
                // Compute and cache
                debug!("Cache miss for computation: {}, computing...", function_name);
                let result = computation().await?;

                // Cache the result
                if let Err(e) = self.cache_result(function_name, inputs, result.clone(), ttl_seconds).await {
                    warn!("Failed to cache computation result: {}", e);
                }

                Ok(result)
            }
            Err(e) => Err(e),
        }
    }
}

/// Cache warming utilities
pub struct CacheWarmer {
    cache_manager: Arc<CacheManager>,
}

impl CacheWarmer {
    pub fn new(cache_manager: Arc<CacheManager>) -> Self {
        Self { cache_manager }
    }

    /// Warm API response cache with common endpoints
    pub async fn warm_api_cache(&self, common_endpoints: Vec<(String, String, Option<String>)>) -> CacheResult<()> {
        if let Ok(api_cache) = self.cache_manager.get_or_create_cache::<serde_json::Value>("api_responses").await {
            let warmer = ApiResponseCache::new(api_cache, CacheConfig::default());

            for (method, path, query) in common_endpoints {
                // This would typically make actual API calls to warm the cache
                // For now, it's a placeholder
                debug!("Would warm cache for {} {}", method, path);
            }
        }

        Ok(())
    }

    /// Warm database query cache with common queries
    pub async fn warm_db_cache(&self, common_queries: Vec<(String, Vec<serde_json::Value>)>) -> CacheResult<()> {
        if let Ok(db_cache) = self.cache_manager.get_or_create_cache::<Vec<serde_json::Value>>("db_queries").await {
            let warmer = DatabaseQueryCache::new(db_cache, CacheConfig::default());

            for (query, params) in common_queries {
                // This would typically execute queries to warm the cache
                // For now, it's a placeholder
                debug!("Would warm cache for query: {}", query);
            }
        }

        Ok(())
    }

    /// Warm LLM cache with common prompts
    pub async fn warm_llm_cache(&self, common_prompts: Vec<(String, String, Option<f32>, Option<u32>)>) -> CacheResult<()> {
        if let Ok(llm_cache) = self.cache_manager.get_or_create_cache::<String>("llm_responses").await {
            let warmer = LlmResponseCache::new(llm_cache, CacheConfig::default());

            for (model, prompt, temperature, max_tokens) in common_prompts {
                // This would typically call LLM APIs to warm the cache
                // For now, it's a placeholder
                debug!("Would warm cache for {} prompt", model);
            }
        }

        Ok(())
    }
}

/// Cache performance monitor
pub struct CacheMonitor {
    cache_manager: Arc<CacheManager>,
    stats_history: Arc<RwLock<Vec<(chrono::DateTime<chrono::Utc>, HashMap<String, CacheStats>)>>>,
    max_history: usize,
}

impl CacheMonitor {
    pub fn new(cache_manager: Arc<CacheManager>, max_history: usize) -> Self {
        Self {
            cache_manager,
            stats_history: Arc::new(RwLock::new(Vec::new())),
            max_history,
        }
    }

    /// Record current cache statistics
    pub async fn record_stats(&self) -> CacheResult<()> {
        let stats = self.cache_manager.global_stats().await;
        let timestamp = chrono::Utc::now();

        let mut history = self.stats_history.write().await;
        history.push((timestamp, stats));

        // Keep only recent history
        if history.len() > self.max_history {
            history.remove(0);
        }

        Ok(())
    }

    /// Get cache hit rate over time
    pub async fn hit_rate_over_time(&self, hours: i64) -> HashMap<String, f64> {
        let history = self.stats_history.read().await;
        let cutoff = chrono::Utc::now() - chrono::Duration::hours(hours);

        let mut cache_hits = HashMap::new();
        let mut cache_total = HashMap::new();

        for (timestamp, stats) in history.iter().rev() {
            if timestamp < &cutoff {
                break;
            }

            for (cache_name, cache_stats) in stats {
                let hits = cache_hits.entry(cache_name.clone()).or_insert(0u64);
                let total = cache_total.entry(cache_name.clone()).or_insert(0u64);

                *hits += cache_stats.hits;
                *total += cache_stats.hits + cache_stats.misses;
            }
        }

        let mut hit_rates = HashMap::new();
        for (cache_name, hits) in cache_hits {
            if let Some(total) = cache_total.get(&cache_name) {
                if *total > 0 {
                    hit_rates.insert(cache_name, hits as f64 / *total as f64);
                }
            }
        }

        hit_rates
    }

    /// Get cache performance recommendations
    pub async fn get_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();
        let hit_rates = self.hit_rate_over_time(24).await;

        for (cache_name, hit_rate) in hit_rates {
            if hit_rate < 0.5 {
                recommendations.push(format!(
                    "Cache '{}' has low hit rate ({:.1}%). Consider adjusting TTL or cache size.",
                    cache_name, hit_rate * 100.0
                ));
            }

            if hit_rate > 0.95 {
                recommendations.push(format!(
                    "Cache '{}' has very high hit rate ({:.1}%). Consider increasing TTL.",
                    cache_name, hit_rate * 100.0
                ));
            }
        }

        // Check for memory usage recommendations
        let global_stats = self.cache_manager.global_stats().await;
        for (cache_name, stats) in global_stats {
            let memory_mb = stats.size_bytes as f64 / (1024.0 * 1024.0);
            if memory_mb > 400.0 { // 80% of default 512MB limit
                recommendations.push(format!(
                    "Cache '{}' is using {:.1}MB memory. Consider increasing max_memory_mb or adjusting eviction policy.",
                    cache_name, memory_mb
                ));
            }
        }

        recommendations
    }

    /// Start background monitoring
    pub async fn start_monitoring(&self, interval_seconds: u64) {
        let monitor = Arc::new(self.clone());

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(std::time::Duration::from_secs(interval_seconds));

            loop {
                interval.tick().await;

                if let Err(e) = monitor.record_stats().await {
                    warn!("Failed to record cache stats: {}", e);
                }
            }
        });

        info!("Started cache monitoring with {}s interval", interval_seconds);
    }
}

impl Clone for CacheMonitor {
    fn clone(&self) -> Self {
        Self {
            cache_manager: self.cache_manager.clone(),
            stats_history: self.stats_history.clone(),
            max_history: self.max_history,
        }
    }
}
