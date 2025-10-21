//! Comprehensive memory management example
//!
//! This example demonstrates:
//! - Memory monitoring and pressure handling
//! - Object pooling for expensive resources
//! - Memory-managed caching
//! - Memory leak detection
//! - Performance monitoring

use agent_agency_memory::*;
use std::sync::Arc;
use std::time::Duration;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🧠 Starting comprehensive memory management demo");

    // 1. Initialize memory management
    let config = MemoryManagementConfig::default();
    let memory_manager = Arc::new(MemoryManager::new(config));
    memory_manager.initialize().await?;

    println!("✅ Memory management initialized");

    // 2. Set up object pools for expensive resources
    setup_object_pools(&memory_manager).await;

    // 3. Create memory-managed caches
    let mut smart_cache = SmartCache::new(
        memory_manager.clone(),
        1000, // max entries
        50,   // max memory MB
        300,  // TTL seconds
    );

    println!("✅ Object pools and caches created");

    // 4. Demonstrate memory monitoring
    demonstrate_memory_monitoring(&memory_manager).await;

    // 5. Simulate workload with object pooling
    simulate_workload(&memory_manager).await;

    // 6. Demonstrate memory-managed caching
    demonstrate_caching(&mut smart_cache).await;

    // 7. Check for memory leaks
    check_memory_leaks(&memory_manager).await;

    // 8. Show performance metrics
    show_performance_metrics(&memory_manager).await;

    println!("🎉 Memory management demo completed successfully!");
    Ok(())
}

async fn setup_object_pools(memory_manager: &Arc<MemoryManager>) {
    println!("🔧 Setting up object pools...");

    // Database connection pool
    memory_manager.create_pool(
        "database_connections",
        || DatabaseConnection {
            id: uuid::Uuid::new_v4(),
            connection_string: "postgresql://localhost/mydb".to_string(),
            created_at: std::time::Instant::now(),
        },
        20
    ).await;

    // LLM client pool
    memory_manager.create_pool(
        "llm_clients",
        || LlmClient {
            id: uuid::Uuid::new_v4(),
            api_key: "sk-...".to_string(),
            model: "gpt-4".to_string(),
            created_at: std::time::Instant::now(),
            request_count: 0,
        },
        10
    ).await;

    // HTTP client pool
    memory_manager.create_pool(
        "http_clients",
        || HttpClient {
            id: uuid::Uuid::new_v4(),
            base_url: "https://api.example.com".to_string(),
            timeout_seconds: 30,
            created_at: std::time::Instant::now(),
            request_count: 0,
        },
        50
    ).await;

    println!("✅ Object pools created");
}

async fn demonstrate_memory_monitoring(memory_manager: &Arc<MemoryManager>) {
    println!("📊 Demonstrating memory monitoring...");

    // Wait a bit for monitoring to collect data
    tokio::time::sleep(Duration::from_secs(1)).await;

    let stats = memory_manager.get_memory_stats();
    let pressure = memory_manager.get_memory_pressure();

    println!("📈 Current Memory Stats:");
    println!("  - Allocated: {} MB", stats.allocated_bytes / (1024 * 1024));
    println!("  - Peak Usage: {} MB", stats.peak_usage_bytes / (1024 * 1024));
    println!("  - Active Allocations: {}", stats.active_allocations);
    println!("  - Memory Pressure: {:?}", pressure);

    // Get memory history
    let history = memory_manager.get_memory_history(Duration::from_secs(60)).await;
    println!("📈 Memory history points: {}", history.len());
}

async fn simulate_workload(memory_manager: &Arc<MemoryManager>) {
    println!("🏭 Simulating workload with object pooling...");

    // Simulate concurrent database operations
    let mut handles = vec![];

    for i in 0..50 {
        let manager = memory_manager.clone();
        let handle = tokio::spawn(async move {
            // Get database connection from pool
            if let Some(mut conn) = manager.get_from_pool::<DatabaseConnection>("database_connections").await {
                let result = conn.get().query(&format!("SELECT * FROM users WHERE id = {}", i)).await;
                match result {
                    Ok(rows) => println!("Query {}: {} rows", i, rows.len()),
                    Err(e) => println!("Query {} failed: {}", i, e),
                }
                // Connection automatically returned to pool when dropped
            }
        });
        handles.push(handle);
    }

    // Wait for all operations to complete
    for handle in handles {
        let _ = handle.await;
    }

    // Check pool stats
    if let Some(pool) = memory_manager.pools.read().await.get("database_connections") {
        if let Some(pool) = pool.downcast_ref::<ObjectPool<DatabaseConnection, Box<dyn Fn() -> DatabaseConnection + Send + Sync>>>() {
            let stats = pool.stats().await;
            println!("🏊 Database pool stats: {:?}", stats);
        }
    }

    println!("✅ Workload simulation completed");
}

async fn demonstrate_caching(smart_cache: &mut SmartCache<String, String>) {
    println!("💾 Demonstrating memory-managed caching...");

    // Insert some data
    for i in 0..100 {
        let key = format!("user_{}", i);
        let value = format!("User data for {}", i);
        smart_cache.insert(key, value);
    }

    println!("📥 Inserted 100 cache entries");

    // Access some data
    for i in 0..10 {
        let key = format!("user_{}", i);
        if let Some(value) = smart_cache.get(&key) {
            println!("📤 Cache hit for {}: {}", key, value);
        }
    }

    // Check memory pressure impact on caching
    let pressure = smart_cache.memory_manager.get_memory_pressure();
    println!("🩸 Cache operating under memory pressure: {:?}", pressure);

    let (entries, memory_mb) = smart_cache.stats();
    println!("📊 Cache stats: {} entries, ~{} MB memory usage", entries, memory_mb);
}

async fn check_memory_leaks(memory_manager: &Arc<MemoryManager>) {
    println!("🔍 Checking for memory leaks...");

    let alerts = memory_manager.analyze_memory_leaks().await;

    if alerts.is_empty() {
        println!("✅ No memory leaks detected");
    } else {
        println!("⚠️ Memory leak alerts:");
        for alert in alerts {
            println!("  - {}", alert);
        }
    }
}

async fn show_performance_metrics(memory_manager: &Arc<MemoryManager>) {
    println!("📈 Performance metrics summary:");

    let stats = memory_manager.get_memory_stats();
    let pressure = memory_manager.get_memory_pressure();

    println!("🔢 Final Memory Statistics:");
    println!("  - Total Allocated: {} MB", stats.allocated_bytes / (1024 * 1024));
    println!("  - Peak Usage: {} MB", stats.peak_usage_bytes / (1024 * 1024));
    println!("  - Allocation Count: {}", stats.allocation_count);
    println!("  - Deallocation Count: {}", stats.deallocation_count);
    println!("  - Active Allocations: {}", stats.active_allocations);
    println!("  - Memory Pressure: {:?}", pressure);

    // Memory usage trend
    let history = memory_manager.get_memory_history(Duration::from_secs(300)).await;
    if history.len() >= 2 {
        let first = &history[0].1;
        let last = &history[history.len() - 1].1;
        let growth = (last.allocated_bytes as f64 - first.allocated_bytes as f64) / first.allocated_bytes as f64 * 100.0;
        println!("📈 Memory growth over session: {:.2}%", growth);
    }

    // Global allocator stats
    let global_allocated = MemoryTrackingAllocator::allocated_bytes();
    let global_peak = MemoryTrackingAllocator::peak_usage();
    let global_allocations = MemoryTrackingAllocator::allocation_count();

    println!("🌍 Global Allocator Stats:");
    println!("  - Current Allocated: {} MB", global_allocated / (1024 * 1024));
    println!("  - Peak Usage: {} MB", global_peak / (1024 * 1024));
    println!("  - Total Allocations: {}", global_allocations);
}

// Helper function to generate UUID (simplified for example)
mod uuid {
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    pub fn Uuid {
        pub fn new_v4() -> String {
            let count = COUNTER.fetch_add(1, Ordering::Relaxed);
            format!("uuid-{:016x}", count)
        }
    }
}
