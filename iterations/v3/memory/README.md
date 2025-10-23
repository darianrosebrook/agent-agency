# Agent Agency Memory Management System

Enterprise-grade memory management for Rust applications with comprehensive monitoring, object pooling, leak detection, and performance optimization.

## Features

### Memory Monitoring
- **Global Allocator Tracking**: Monitors all heap allocations and deallocations
- **Memory Pressure Detection**: Low, Moderate, High, and Critical pressure levels
- **Real-time Statistics**: Current usage, peak usage, allocation counts
- **Automated GC Triggers**: Memory pressure-based garbage collection

### Object Pooling
- **Generic Object Pools**: Type-safe pooling for any expensive resource
- **Automatic Lifecycle**: Objects returned to pool on drop
- **Size Limits**: Configurable maximum pool sizes
- **Health Checking**: Optional health validation for pooled objects

### Smart Caching
- **Memory-Aware Caches**: Size and memory limits with LRU eviction
- **TTL Support**: Time-based expiration
- **Pressure-Aware**: Adjusts behavior based on memory pressure
- **Integration Ready**: Works with existing cache implementations

### Leak Detection
- **Allocation Tracking**: Monitors allocation patterns over time
- **Trend Analysis**: Detects memory growth trends
- **Snapshot Comparison**: Compare memory states across time periods
- **Alert Generation**: Automatic alerts for potential leaks

### Performance Monitoring
- **Memory Trends**: Increasing, Decreasing, or Stable trends
- **Peak Usage Tracking**: Historical peak memory usage
- **Average Calculations**: Rolling average memory usage
- **Performance Metrics**: Response times under memory pressure

## Quick Start

```rust
use agent_agency_memory::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize memory management
    let config = MemoryManagementConfig::default();
    let memory_manager = Arc::new(MemoryManager::new(config));
    memory_manager.initialize().await?;

    // Create an object pool for expensive resources
    memory_manager.create_pool(
        "database_connections",
        || create_database_connection(),
        20
    ).await;

    // Use pooled objects
    if let Some(mut conn) = memory_manager.get_from_pool::<DatabaseConnection>("database_connections").await {
        let result = conn.get().query("SELECT * FROM users").await?;
        // Connection automatically returned to pool
    }

    // Monitor memory usage
    let stats = memory_manager.get_memory_stats();
    println!("Memory usage: {} MB", stats.allocated_bytes / (1024 * 1024));

    Ok(())
}
```

## Configuration

```rust
let config = MemoryManagementConfig {
    monitor_config: MemoryLimitConfig {
        max_heap_mb: 1024,        // 1GB heap limit
        max_stack_mb: 8,          // 8MB per thread
        warning_threshold_mb: 768, // Warning at 75%
        critical_threshold_mb: 896, // Critical at 87.5%
        enable_gc_pressure: true,
        gc_pressure_threshold_mb: 800,
        monitoring_interval_ms: 5000, // Check every 5 seconds
    },
    enable_object_pooling: true,
    database_connection_pool_size: 20,
    llm_client_pool_size: 10,
    enable_leak_detection: true,
    leak_detection_threshold_mb: 100,
};
```

## Integration Examples

### Database Connection Pooling

```rust
use agent_agency_memory::integration::DatabaseConnectionPool;

let pool = DatabaseConnectionPool::new(20, "postgresql://...".to_string());
let mut connection = pool.get_connection().await;

// Use connection
let result = connection.query("SELECT * FROM users").await?;
// Automatically returned to pool when dropped
```

### LLM Client Pooling

```rust
use agent_agency_memory::integration::LlmClientPool;

let pool = LlmClientPool::new(10, "sk-...".to_string(), "gpt-4".to_string());
let mut client = pool.get_client().await;

// Use client
let response = client.generate("Hello, world!").await?;
// Automatically returned to pool when dropped
```

### Memory-Aware Caching

```rust
use agent_agency_memory::integration::SmartCache;

let mut cache = SmartCache::new(
    memory_manager,
    1000, // max entries
    50,   // max memory MB
    300,  // TTL seconds
);

// Cache automatically manages memory pressure
cache.insert("user:123".to_string(), user_data);
if let Some(data) = cache.get(&"user:123".to_string()) {
    // Use cached data
}
```

### Memory Pressure Handling

```rust
use agent_agency_memory::integration::MemoryPressureManager;

let pressure_manager = MemoryPressureManager::new(memory_manager);
pressure_manager.monitor_and_respond().await;

// Automatically handles memory pressure events
```

## Memory Pressure Levels

| Level | Description | Actions |
|-------|-------------|---------|
| **Low** | Normal operation | No special actions |
| **Moderate** | Increased usage | Reduce cache sizes |
| **High** | Approaching limits | Force GC, reduce caches |
| **Critical** | Near limits | Reject requests, aggressive GC |

## Performance Characteristics

- **Memory Overhead**: ~1-2% for monitoring infrastructure
- **Allocation Tracking**: O(1) atomic operations per allocation
- **Pool Operations**: O(1) for borrow/return
- **Cache Operations**: O(1) average case with LRU eviction
- **Leak Detection**: O(n) for trend analysis

## Production Deployment

### Environment Variables

```bash
# Memory limits
MEMORY_MAX_HEAP_MB=2048
MEMORY_MAX_STACK_MB=16
MEMORY_WARNING_THRESHOLD_MB=1536
MEMORY_CRITICAL_THRESHOLD_MB=1792

# Pool sizes
MEMORY_DB_POOL_SIZE=50
MEMORY_LLM_POOL_SIZE=20
MEMORY_HTTP_POOL_SIZE=100

# Monitoring
MEMORY_MONITORING_INTERVAL_MS=10000
MEMORY_LEAK_THRESHOLD_MB=200
```

### Health Checks

```rust
// Memory health endpoint
#[get("/health/memory")]
async fn memory_health(memory_manager: web::Data<MemoryManager>) -> impl Responder {
    let stats = memory_manager.get_memory_stats();
    let pressure = memory_manager.get_memory_pressure();

    let status = match pressure {
        MemoryPressure::Critical => "unhealthy",
        MemoryPressure::High => "degraded",
        _ => "healthy",
    };

    web::Json(serde_json::json!({
        "status": status,
        "memory_mb": stats.allocated_bytes / (1024 * 1024),
        "peak_mb": stats.peak_usage_bytes / (1024 * 1024),
        "pressure": format!("{:?}", pressure),
    }))
}
```

## Monitoring and Observability

### Metrics

- `memory_allocated_bytes`: Current allocated memory
- `memory_peak_bytes`: Peak memory usage
- `memory_pressure_level`: Current pressure level (0-3)
- `memory_allocation_count`: Total allocations
- `memory_pool_size`: Current pool sizes
- `memory_pool_borrowed`: Borrowed objects per pool

### Logs

```
INFO - Memory pressure changed to High
WARN - Memory limit exceeded: 950 MB used, 1000 MB limit
ERROR - Memory pressure critical, rejecting requests
INFO - Forced garbage collection due to memory pressure
```

## Best Practices

### Object Pooling
- Use for expensive resources (DB connections, HTTP clients, LLM clients)
- Set pool sizes based on expected concurrent usage
- Implement health checks for pooled objects
- Monitor pool utilization and adjust sizes

### Memory Limits
- Set realistic limits based on container memory allocation
- Use warning thresholds to trigger proactive actions
- Enable GC pressure triggers for automatic cleanup
- Monitor trends to adjust limits over time

### Leak Detection
- Enable in development and staging environments
- Set appropriate thresholds for your application
- Use snapshot comparisons to identify leak sources
- Integrate alerts with monitoring systems

### Performance Monitoring
- Track memory trends during normal operation
- Monitor cache hit rates and memory pressure correlation
- Use performance baselines for regression detection
- Implement circuit breakers for memory-critical operations

## Troubleshooting

### High Memory Usage
1. Check allocation patterns with memory snapshots
2. Analyze object pool utilization
3. Review cache sizes and TTL settings
4. Look for memory leaks in trend analysis

### Pool Exhaustion
1. Increase pool sizes based on usage patterns
2. Implement request queuing for peak loads
3. Add health checks to detect stale connections
4. Monitor pool stats for optimization opportunities

### Memory Pressure Events
1. Review cache invalidation strategies
2. Implement selective cache clearing
3. Add request throttling under pressure
4. Consider horizontal scaling options

## Examples

See `examples/` directory for comprehensive usage examples:

- `comprehensive_usage.rs`: Full system demonstration
- Database connection pooling patterns
- Memory-aware caching strategies
- Pressure response automation

## License

Licensed under the same terms as the Agent Agency project.
