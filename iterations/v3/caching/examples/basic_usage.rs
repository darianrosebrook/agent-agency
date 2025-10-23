//! Example usage of the multi-level caching system

use agent_agency_caching::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!(" Agent Agency Caching System Demo");

    // Configure caching
    let cache_config = CacheConfig {
        max_memory_mb: 128, // Smaller for demo
        default_ttl_seconds: 300, // 5 minutes
        enable_compression: false,
        enable_serialization: true,
        eviction_policy: EvictionPolicy::Lru,
        enable_metrics: true,
        redis_url: None, // Using memory-only for demo
        redis_cluster: false,
        cdn_enabled: false,
        cdn_base_url: None,
    };

    // Create cache manager
    let cache_manager = Arc::new(CacheManager::new(cache_config.clone()));

    // Example 1: API Response Caching
    println!("\n API Response Caching");
    let api_cache = cache_manager
        .get_or_create_cache::<serde_json::Value>("api_responses")
        .await?;
    let api_response_cache = ApiResponseCache::new(api_cache, cache_config.clone());

    // Cache an API response
    let response_data = serde_json::json!({
        "users": [
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"}
        ],
        "total": 2
    });

    api_response_cache
        .cache_response("GET", "/api/users", Some("limit=10"), None, response_data.clone(), Some(300))
        .await?;

    // Retrieve from cache
    let cached_response = api_response_cache
        .get_cached_response("GET", "/api/users", Some("limit=10"), None)
        .await?;

    println!(" Cached API response: {}", cached_response);

    // Example 2: Database Query Caching
    println!("\n🗄️ Database Query Caching");
    let db_cache = cache_manager
        .get_or_create_cache::<Vec<serde_json::Value>>("db_queries")
        .await?;
    let db_query_cache = DatabaseQueryCache::new(db_cache, cache_config.clone());

    // Cache query results
    let query_results = vec![
        serde_json::json!({"id": 1, "title": "Task 1", "status": "completed"}),
        serde_json::json!({"id": 2, "title": "Task 2", "status": "pending"}),
    ];

    let query_params = vec![serde_json::json!("completed")];
    db_query_cache
        .cache_results("SELECT * FROM tasks WHERE status = $1", &query_params, query_results.clone(), Some(600))
        .await?;

    // Retrieve from cache
    let cached_results = db_query_cache
        .get_cached_results("SELECT * FROM tasks WHERE status = $1", &query_params)
        .await?;

    println!(" Cached {} query results", cached_results.len());

    // Example 3: LLM Response Caching
    println!("\n LLM Response Caching");
    let llm_cache = cache_manager
        .get_or_create_cache::<String>("llm_responses")
        .await?;
    let llm_response_cache = LlmResponseCache::new(llm_cache, cache_config.clone());

    // Cache LLM response
    let llm_response = "The capital of France is Paris. This is a well-known fact in geography.".to_string();
    llm_response_cache
        .cache_response(
            "gpt-4",
            "What is the capital of France?",
            Some(0.7),
            Some(100),
            llm_response.clone(),
            Some(3600) // 1 hour TTL
        )
        .await?;

    // Retrieve from cache
    let cached_llm_response = llm_response_cache
        .get_cached_response("gpt-4", "What is the capital of France?", Some(0.7), Some(100))
        .await?;

    println!(" Cached LLM response: {}", cached_llm_response);

    // Example 4: Computation Result Caching
    println!("\n Computation Result Caching");
    let comp_cache = cache_manager
        .get_or_create_cache::<i64>("computations")
        .await?;
    let computation_cache = ComputationCache::new(comp_cache, cache_config.clone());

    // Define an expensive computation (fibonacci for demo)
    let fibonacci = |n: i64| async move {
        if n <= 1 {
            Ok(n)
        } else {
            // Simulate expensive computation
            tokio::time::sleep(Duration::from_millis(100)).await;
            Ok(fibonacci(n-1) + fibonacci(n-2))
        }
    };

    // Compute with caching
    let inputs = vec![serde_json::json!(10)];
    let result = computation_cache
        .compute_with_cache(
            "fibonacci",
            &inputs,
            || fibonacci(10),
            Some(300)
        )
        .await?;

    println!(" Computed fibonacci(10) = {} (cached for future calls)", result);

    // Example 5: Cache Statistics and Monitoring
    println!("\n Cache Statistics");
    let monitor = CacheMonitor::new(cache_manager.clone(), 100);
    monitor.record_stats().await?;

    let hit_rates = monitor.hit_rate_over_time(1).await;
    for (cache_name, hit_rate) in hit_rates {
        println!(" {} hit rate: {:.1}%", cache_name, hit_rate * 100.0);
    }

    let recommendations = monitor.get_recommendations().await;
    if !recommendations.is_empty() {
        println!(" Recommendations:");
        for rec in recommendations {
            println!("   • {}", rec);
        }
    }

    // Example 6: Cache Warming
    println!("\n Cache Warming");
    let warmer = CacheWarmer::new(cache_manager.clone());

    // Warm with common API endpoints
    let common_endpoints = vec![
        ("GET".to_string(), "/api/health".to_string(), None),
        ("GET".to_string(), "/api/status".to_string(), None),
    ];

    warmer.warm_api_cache(common_endpoints).await?;
    println!(" Cache warming completed for common endpoints");

    // Example 7: Cache Invalidation
    println!("\n🗑️ Cache Invalidation");
    if let Ok(api_cache) = cache_manager.get_or_create_cache::<serde_json::Value>("api_responses").await {
        let multi_cache = MultiLevelCache::<serde_json::Value>::new(cache_config.clone())?;
        let invalidated = multi_cache.invalidate_by_tags(&["api".to_string()]).await?;
        println!(" Invalidated {} cache entries by tag", invalidated);
    }

    println!("\n Caching system demo completed successfully!");
    println!(" Key benefits demonstrated:");
    println!("   • Multi-level caching (memory + Redis support)");
    println!("   • Intelligent cache invalidation");
    println!("   • Performance monitoring and recommendations");
    println!("   • Cache warming for hot data");
    println!("   • Different cache types for different use cases");

    Ok(())
}
