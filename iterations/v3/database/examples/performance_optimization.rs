//! Database performance optimization example
//!
//! Demonstrates how to use the database optimization features
//! for monitoring, indexing, and read/write splitting.

use agent_agency_database::*;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn<std::error::Error>>> {
    println!(" Agent Agency Database Performance Optimization Demo");

    // Configure database
    let mut db_config = DatabaseConfig::default();
    db_config.enable_read_write_splitting = true;
    db_config.read_replicas = vec![
        DatabaseReplicaConfig {
            host: "replica1.example.com".to_string(),
            port: 5432,
            weight: 10,
            is_sync: true,
        },
        DatabaseReplicaConfig {
            host: "replica2.example.com".to_string(),
            port: 5432,
            weight: 5,
            is_sync: false,
        },
    ];

    // Configure optimization
    let opt_config = DatabaseOptimizationConfig {
        enable_query_monitoring: true,
        slow_query_threshold_ms: 500,
        enable_index_suggestions: true,
        enable_read_write_splitting: db_config.enable_read_write_splitting,
        read_replicas: db_config.read_replicas.iter()
            .map(|r| format!("{}:{}", r.host, r.port))
            .collect(),
        query_cache_size: 1000,
        enable_query_plan_analysis: true,
        monitoring_interval_seconds: 60,
    };

    // Create database client (simplified for demo)
    // let db_client = DatabaseClient::new(db_config.clone()).await?;

    println!("\n Database Optimization Configuration");
    println!("═══════════════════════════════════════");
    println!("Query monitoring: {}", opt_config.enable_query_monitoring);
    println!("Slow query threshold: {}ms", opt_config.slow_query_threshold_ms);
    println!("Index suggestions: {}", opt_config.enable_index_suggestions);
    println!("Read/write splitting: {}", opt_config.enable_read_write_splitting);
    println!("Read replicas: {}", opt_config.read_replicas.len());

    // Create optimization manager
    // let opt_manager = DatabaseOptimizationManager::new(db_client, opt_config);

    println!("\n Query Performance Monitoring");
    println!("═════════════════════════════════");

    // Simulate some query executions
    let sample_queries = vec![
        ("SELECT * FROM tasks WHERE status = $1", 150),
        ("SELECT * FROM tasks WHERE created_at > $1 ORDER BY created_at DESC", 800), // Slow query
        ("SELECT COUNT(*) FROM task_executions WHERE task_id = $1", 50),
        ("SELECT * FROM knowledge_entries WHERE relevance_score > $1 ORDER BY relevance_score DESC", 1200), // Very slow
        ("INSERT INTO tasks (title, description, status) VALUES ($1, $2, $3)", 80),
    ];

    println!(" Simulating query executions...");
    for (query, execution_time) in &sample_queries {
        println!("   Query: {}... ({}ms)", &query[..query.len().min(60)], execution_time);

        // In real usage, this would be recorded automatically by MonitoredQueryExecutor
        // opt_manager.monitor.record_query_execution(query, *execution_time).await;
    }

    println!("\n Index Recommendations Analysis");
    println!("═══════════════════════════════════");

    // Simulate index recommendations based on query patterns
    let recommendations = vec![
        IndexRecommendation {
            table_name: "tasks".to_string(),
            column_name: "status".to_string(),
            index_type: "btree".to_string(),
            estimated_improvement: 0.85,
            query_patterns: vec!["status filtering".to_string()],
            priority: IndexPriority::High,
        },
        IndexRecommendation {
            table_name: "tasks".to_string(),
            column_name: "created_at".to_string(),
            index_type: "btree".to_string(),
            estimated_improvement: 0.75,
            query_patterns: vec!["time range queries".to_string()],
            priority: IndexPriority::High,
        },
        IndexRecommendation {
            table_name: "knowledge_entries".to_string(),
            column_name: "relevance_score".to_string(),
            index_type: "btree".to_string(),
            estimated_improvement: 0.90,
            query_patterns: vec!["search ranking".to_string()],
            priority: IndexPriority::Critical,
        },
    ];

    println!(" Recommended Indexes:");
    for rec in &recommendations {
        println!("   • {} index on {}.{} (priority: {:?}, improvement: {:.1}%)",
            rec.index_type, rec.table_name, rec.column_name, rec.priority,
            rec.estimated_improvement * 100.0);
    }

    println!("\n⚖️ Read/Write Splitting Simulation");
    println!("═══════════════════════════════════");

    // Simulate read/write splitting decisions
    let operations = vec![
        ("SELECT * FROM tasks WHERE id = ?", "READ", "replica1"),
        ("INSERT INTO task_executions (task_id, worker_id, status) VALUES (?, ?, ?)", "WRITE", "primary"),
        ("SELECT COUNT(*) FROM knowledge_entries WHERE relevance_score > ?", "READ", "replica2"),
        ("UPDATE tasks SET status = ? WHERE id = ?", "WRITE", "primary"),
        ("SELECT * FROM council_verdicts WHERE task_id = ? ORDER BY created_at DESC", "READ", "replica1"),
    ];

    println!(" Operation Routing:");
    for (query, operation_type, target) in &operations {
        println!("   {} → {} ({})", &query[..query.len().min(40)], operation_type, target);
    }

    println!("\n Performance Metrics Summary");
    println!("════════════════════════════════");

    let mock_metrics = vec![
        ("Total queries executed", "1,247"),
        ("Average query time", "245ms"),
        ("Slow queries (>500ms)", "23"),
        ("Cache hit rate", "87.3%"),
        ("Connection pool utilization", "73%"),
        ("Read replica lag", "< 100ms"),
    ];

    for (metric, value) in &mock_metrics {
        println!("   {:<25}: {}", metric, value);
    }

    println!("\n Database Optimization Benefits");
    println!("═══════════════════════════════════");

    let benefits = vec![
        ("Query Performance", "70-90% improvement with proper indexing"),
        ("Read Scalability", "5-10x increase with read replicas"),
        ("Connection Efficiency", "Better pool utilization and health checks"),
        ("Monitoring Insights", "Real-time performance tracking and alerts"),
        ("Automatic Optimization", "Index suggestions and slow query detection"),
        ("High Availability", "Read/write splitting and replica failover"),
    ];

    for (area, benefit) in &benefits {
        println!("   {:<18}: {}", area, benefit);
    }

    println!("\n Database optimization demo completed!");
    println!(" Key takeaways:");
    println!("   • Monitor query performance continuously");
    println!("   • Use strategic indexing for common query patterns");
    println!("   • Implement read/write splitting for scalability");
    println!("   • Leverage read replicas for high-traffic read operations");
    println!("   • Use performance monitoring to identify bottlenecks");
    println!("   • Apply indexes based on actual usage patterns, not assumptions");

    Ok(())
}
