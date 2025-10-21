# Agent Agency Caching System

Enterprise-grade multi-level caching system providing memory, Redis, and CDN caching with intelligent invalidation, cache warming, and comprehensive performance monitoring.

## Features

- **Multi-Level Caching**: Memory (LRU) → Redis → CDN hierarchy
- **Intelligent Invalidation**: Tag-based, time-based, and write-through invalidation
- **Cache Warming**: Proactive loading of hot data
- **Performance Monitoring**: Hit rates, memory usage, and optimization recommendations
- **Enterprise Ready**: Production-tested with circuit breaker integration
- **Type Safe**: Generic caching with compile-time type checking

## Quick Start

```rust
use agent_agency_caching::*;

// Configure caching
let config = CacheConfig {
    max_memory_mb: 512,
    default_ttl_seconds: 3600,
    enable_compression: true,
    redis_url: Some("redis://localhost:6379".to_string()),
    ..Default::default()
};

// Create cache manager
let cache_manager = Arc::new(CacheManager::new(config));

// Create typed cache
let api_cache = cache_manager
    .get_or_create_cache::<serde_json::Value>("api_responses")
    .await?;

// Use API response cache
let response_cache = ApiResponseCache::new(api_cache, config);
response_cache.cache_response("GET", "/api/users", None, None, response_data, Some(300)).await?;
```

## Cache Types

### 1. Memory Cache (LRU)
Fast in-process caching with configurable eviction policies.

```rust
let memory_cache = MemoryCache::<String>::new(config);
memory_cache.set("key".to_string(), "value".to_string(), Some(Duration::from_secs(300))).await?;
```

### 2. Redis Cache
Distributed caching with persistence and clustering support.

```rust
let redis_cache = RedisCache::<serde_json::Value>::new(config)?;
redis_cache.set("api_data".to_string(), json_data, Some(Duration::from_secs(600))).await?;
```

### 3. Multi-Level Cache
Combines memory and Redis with automatic fallback and promotion.

```rust
let multi_cache = MultiLevelCache::<String>::new(config)?;
// Automatically uses memory first, then Redis
let value = multi_cache.get(&"key".to_string()).await?;
```

## Integration Utilities

### API Response Caching
```rust
let api_cache = ApiResponseCache::new(cache, config);
// Cache GET /api/users?limit=10
api_cache.cache_response("GET", "/api/users", Some("limit=10"), None, response, Some(300)).await?;
// Retrieve from cache
let cached = api_cache.get_cached_response("GET", "/api/users", Some("limit=10"), None).await?;
```

### Database Query Caching
```rust
let db_cache = DatabaseQueryCache::new(cache, config);
// Cache SELECT results
db_cache.cache_results("SELECT * FROM users WHERE active = $1", &[true.into()], results, Some(600)).await?;
// Get cached results
let cached_results = db_cache.get_cached_results("SELECT * FROM users WHERE active = $1", &[true.into()]).await?;
```

### LLM Response Caching
```rust
let llm_cache = LlmResponseCache::new(cache, config);
// Cache expensive LLM calls
llm_cache.cache_response("gpt-4", "Explain quantum physics", Some(0.7), Some(500), response, Some(3600)).await?;
// Reuse cached responses
let cached = llm_cache.get_cached_response("gpt-4", "Explain quantum physics", Some(0.7), Some(500)).await?;
```

### Computation Result Caching
```rust
let comp_cache = ComputationCache::new(cache, config);
// Cache expensive computations
let result = comp_cache.compute_with_cache(
    "fibonacci",
    &[serde_json::json!(20)],
    || async { fibonacci(20).await },
    Some(300)
).await?;
```

## Cache Warming

Proactively load frequently accessed data:

```rust
let warmer = CacheWarmer::new(cache_manager);

// Warm common API endpoints
warmer.warm_api_cache(vec![
    ("GET".to_string(), "/api/health".to_string(), None),
    ("GET".to_string(), "/api/config".to_string(), None),
]).await?;

// Warm common database queries
warmer.warm_db_cache(vec![
    ("SELECT * FROM users WHERE active = true".to_string(), vec![true.into()]),
]).await?;
```

## Performance Monitoring

Track cache performance and get optimization recommendations:

```rust
let monitor = CacheMonitor::new(cache_manager, 1000);

// Start background monitoring
monitor.start_monitoring(60).await; // Every 60 seconds

// Get hit rates over time
let hit_rates = monitor.hit_rate_over_time(24).await; // Last 24 hours

// Get optimization recommendations
let recommendations = monitor.get_recommendations().await;
for rec in recommendations {
    println!("💡 {}", rec);
}
```

## Configuration

```rust
let config = CacheConfig {
    max_memory_mb: 512,                    // Memory limit
    default_ttl_seconds: 3600,             // Default TTL
    enable_compression: true,              // Compress large values
    enable_serialization: true,            // JSON serialization
    eviction_policy: EvictionPolicy::Lru, // LRU eviction
    enable_metrics: true,                  // Performance metrics
    redis_url: Some("redis://localhost:6379".to_string()),
    redis_cluster: false,                  // Redis cluster mode
    cdn_enabled: false,                    // CDN integration
    cdn_base_url: None,                    // CDN base URL
};
```

## Cache Invalidation Strategies

### 1. Time-Based (TTL)
Automatic expiration based on time.

### 2. Tag-Based
Invalidate related cache entries by tags:

```rust
multi_cache.invalidate_by_tags(&["user".to_string(), "profile".to_string()]).await?;
```

### 3. Write-Through
Automatically invalidate related keys on write operations.

### 4. Manual
Explicit invalidation when needed:

```rust
cache.delete(&"specific_key".to_string()).await?;
cache.clear().await?; // Clear all
```

## CDN Integration

For static content caching (future enhancement):

```rust
let config = CacheConfig {
    cdn_enabled: true,
    cdn_base_url: Some("https://cdn.example.com".to_string()),
    ..Default::default()
};
```

## Performance Benchmarks

Typical performance characteristics:

- **Memory Cache**: ~1μs read/write latency
- **Redis Cache**: ~100μs read/write latency (local Redis)
- **Cache Hit Rate**: Target 80%+ for optimal performance
- **Memory Usage**: Configurable limits with intelligent eviction

## Production Considerations

### Health Checks
```rust
// Check cache health
let stats = cache.stats().await?;
if stats.hit_rate() < 0.5 {
    warn!("Cache hit rate below 50%: {:.1}%", stats.hit_rate() * 100.0);
}
```

### Monitoring Integration
```rust
// Export metrics to Prometheus
let hit_rate = stats.hits as f64 / (stats.hits + stats.misses) as f64;
metrics::gauge!("cache_hit_rate", hit_rate);
metrics::gauge!("cache_memory_usage_mb", stats.size_bytes as f64 / (1024.0 * 1024.0));
```

### Circuit Breaker Integration
```rust
// Wrap Redis operations with circuit breaker
let result = circuit_breaker.execute(|| redis_cache.get(&key)).await?;
```

## Architecture

```
┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   Application   │────│  Memory Cache   │────│   Redis Cache   │
│                 │    │    (Fast)       │    │ (Distributed)   │
└─────────────────┘    └─────────────────┘    └─────────────────┘
                                │                        │
                                └────────────────────────┘
                                         │
                                ┌─────────────────┐
                                │     CDN Cache   │
                                │   (Static)      │
                                └─────────────────┘
```

## Migration from Single Cache

Existing single-cache implementations can be migrated gradually:

```rust
// Before
let cache = MemoryCache::<String>::new(config);

// After
let multi_cache = MultiLevelCache::<String>::new(config)?;
// Same API, enhanced performance
```

## Troubleshooting

### Common Issues

1. **Low Hit Rate**
   - Increase TTL values
   - Review cache key generation
   - Check for cache invalidation issues

2. **Memory Pressure**
   - Reduce `max_memory_mb`
   - Adjust eviction policy
   - Monitor memory usage trends

3. **Redis Connection Issues**
   - Verify Redis URL and credentials
   - Check network connectivity
   - Enable circuit breaker protection

### Debug Logging

Enable detailed logging:

```rust
tracing_subscriber::fmt()
    .with_env_filter("agent_agency_caching=debug")
    .init();
```

## Contributing

See the main Agent Agency repository for contribution guidelines.

## License

MIT License - see main repository for details.
