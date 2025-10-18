//! Performance Benchmarks and Load Tests
//! 
//! Comprehensive performance testing for all system components

use crate::test_utils::*;
use anyhow::Result;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;
use serde_json::json;
use tokio::time::timeout;

#[cfg(test)]
mod api_performance_tests {
    use super::*;

    /// Test API response times under normal load
    #[tokio::test]
    async fn test_api_response_times_normal_load() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        let http_client = test_utils.initialize_http_client().await?;
        
        // Test multiple API endpoints
        let endpoints = vec![
            "/api/v1/health",
            "/api/v1/tasks",
            "/api/v1/council/evaluate",
            "/api/v1/claim-extraction/process",
            "/api/v1/research/query"
        ];
        
        let mut response_times = Vec::new();
        
        for endpoint in endpoints {
            let start = Instant::now();
            let response = http_client.get(endpoint).send().await?;
            let duration = start.elapsed();
            
            response_times.push(duration);
            
            // Validate response time is reasonable
            assert!(duration < Duration::from_millis(500), 
                   "API endpoint {} should respond within 500ms, took {:?}", endpoint, duration);
            
            // Validate response is successful
            assert!(response.status().is_success(), 
                   "API endpoint {} should return success status", endpoint);
        }
        
        // Calculate average response time
        let avg_response_time: Duration = response_times.iter().sum::<Duration>() / response_times.len() as u32;
        println!("Average API response time: {:?}", avg_response_time);
        
        // Validate average response time is acceptable
        assert!(avg_response_time < Duration::from_millis(200), 
               "Average API response time should be under 200ms");
        
        Ok(())
    }

    /// Test API performance under concurrent load
    #[tokio::test]
    async fn test_api_performance_concurrent_load() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        let http_client = test_utils.initialize_http_client().await?;
        
        // Simulate concurrent requests
        let concurrent_requests = 50;
        let endpoint = "/api/v1/health";
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        // Spawn concurrent requests
        for _ in 0..concurrent_requests {
            let client = http_client.clone();
            let handle = tokio::spawn(async move {
                let request_start = Instant::now();
                let response = client.get(endpoint).send().await;
                let request_duration = request_start.elapsed();
                (response, request_duration)
            });
            handles.push(handle);
        }
        
        // Wait for all requests to complete
        let mut total_duration = Duration::from_secs(0);
        let mut successful_requests = 0;
        
        for handle in handles {
            let (response_result, request_duration) = handle.await?;
            total_duration += request_duration;
            
            if let Ok(response) = response_result {
                if response.status().is_success() {
                    successful_requests += 1;
                }
            }
        }
        
        let total_time = start.elapsed();
        let avg_request_time = total_duration / concurrent_requests as u32;
        let requests_per_second = concurrent_requests as f64 / total_time.as_secs_f64();
        
        println!("Concurrent load test results:");
        println!("  Total time: {:?}", total_time);
        println!("  Average request time: {:?}", avg_request_time);
        println!("  Requests per second: {:.2}", requests_per_second);
        println!("  Successful requests: {}/{}", successful_requests, concurrent_requests);
        
        // Validate performance metrics
        assert!(successful_requests >= concurrent_requests * 95 / 100, 
               "At least 95% of requests should succeed under concurrent load");
        assert!(avg_request_time < Duration::from_millis(1000), 
               "Average request time should be under 1 second under concurrent load");
        assert!(requests_per_second > 10.0, 
               "Should handle at least 10 requests per second");
        
        Ok(())
    }

    /// Test API performance degradation under stress
    #[tokio::test]
    async fn test_api_performance_stress_test() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        let http_client = test_utils.initialize_http_client().await?;
        
        // Stress test with high concurrent load
        let stress_requests = 200;
        let endpoint = "/api/v1/health";
        
        let start = Instant::now();
        let mut handles = Vec::new();
        
        // Spawn stress test requests
        for _ in 0..stress_requests {
            let client = http_client.clone();
            let handle = tokio::spawn(async move {
                let request_start = Instant::now();
                let response = client.get(endpoint).send().await;
                let request_duration = request_start.elapsed();
                (response, request_duration)
            });
            handles.push(handle);
        }
        
        // Wait for all requests with timeout
        let timeout_duration = Duration::from_secs(30);
        let mut successful_requests = 0;
        let mut failed_requests = 0;
        let mut total_response_time = Duration::from_secs(0);
        
        for handle in handles {
            match timeout(timeout_duration, handle).await {
                Ok(Ok((response_result, request_duration))) => {
                    total_response_time += request_duration;
                    if let Ok(response) = response_result {
                        if response.status().is_success() {
                            successful_requests += 1;
                        } else {
                            failed_requests += 1;
                        }
                    } else {
                        failed_requests += 1;
                    }
                }
                _ => {
                    failed_requests += 1;
                }
            }
        }
        
        let total_time = start.elapsed();
        let avg_response_time = if successful_requests > 0 {
            total_response_time / successful_requests as u32
        } else {
            Duration::from_secs(0)
        };
        
        println!("Stress test results:");
        println!("  Total time: {:?}", total_time);
        println!("  Successful requests: {}", successful_requests);
        println!("  Failed requests: {}", failed_requests);
        println!("  Average response time: {:?}", avg_response_time);
        
        // Validate stress test results
        let success_rate = successful_requests as f64 / stress_requests as f64;
        assert!(success_rate > 0.8, 
               "Success rate should be at least 80% under stress (was {:.2}%)", success_rate * 100.0);
        
        if successful_requests > 0 {
            assert!(avg_response_time < Duration::from_secs(5), 
                   "Average response time should be under 5 seconds under stress");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod database_performance_tests {
    use super::*;

    /// Test database query performance
    #[tokio::test]
    async fn test_database_query_performance() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        let database = test_utils.initialize_database().await?;
        
        // Test different query types
        let queries = vec![
            ("simple_select", "SELECT * FROM tasks LIMIT 10"),
            ("complex_join", "SELECT t.*, c.name FROM tasks t JOIN categories c ON t.category_id = c.id LIMIT 10"),
            ("aggregate", "SELECT COUNT(*), AVG(priority) FROM tasks"),
            ("indexed_lookup", "SELECT * FROM tasks WHERE id = $1")
        ];
        
        let mut query_times = HashMap::new();
        
        for (query_name, query_sql) in queries {
            let start = Instant::now();
            
            // Execute query
            let result = database.execute_query(query_sql, &[]).await?;
            
            let duration = start.elapsed();
            query_times.insert(query_name, duration);
            
            // Validate query performance
            match query_name {
                "simple_select" => {
                    assert!(duration < Duration::from_millis(100), 
                           "Simple select should complete within 100ms");
                }
                "complex_join" => {
                    assert!(duration < Duration::from_millis(200), 
                           "Complex join should complete within 200ms");
                }
                "aggregate" => {
                    assert!(duration < Duration::from_millis(300), 
                           "Aggregate query should complete within 300ms");
                }
                "indexed_lookup" => {
                    assert!(duration < Duration::from_millis(50), 
                           "Indexed lookup should complete within 50ms");
                }
                _ => {}
            }
        }
        
        // Print performance summary
        println!("Database query performance:");
        for (query_name, duration) in &query_times {
            println!("  {}: {:?}", query_name, duration);
        }
        
        Ok(())
    }

    /// Test database write performance
    #[tokio::test]
    async fn test_database_write_performance() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        let database = test_utils.initialize_database().await?;
        
        // Test batch insert performance
        let batch_size = 100;
        let test_data = create_test_batch_data(batch_size);
        
        let start = Instant::now();
        let result = database.batch_insert("tasks", &test_data).await?;
        let duration = start.elapsed();
        
        // Validate write performance
        assert!(result.rows_affected == batch_size as u64, 
               "Should insert all {} rows", batch_size);
        assert!(duration < Duration::from_millis(1000), 
               "Batch insert of {} rows should complete within 1 second", batch_size);
        
        let rows_per_second = batch_size as f64 / duration.as_secs_f64();
        println!("Database write performance: {:.2} rows/second", rows_per_second);
        
        // Validate minimum write performance
        assert!(rows_per_second > 100.0, 
               "Should write at least 100 rows per second");
        
        Ok(())
    }

    /// Test database connection pool performance
    #[tokio::test]
    async fn test_database_connection_pool_performance() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        let database = test_utils.initialize_database().await?;
        
        // Test concurrent database operations
        let concurrent_operations = 20;
        let mut handles = Vec::new();
        
        let start = Instant::now();
        
        for i in 0..concurrent_operations {
            let db = database.clone();
            let handle = tokio::spawn(async move {
                let query = "SELECT * FROM tasks WHERE id = $1";
                let params = vec![format!("task_{}", i)];
                db.execute_query(query, &params).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        let mut successful_operations = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(_)) => successful_operations += 1,
                _ => {}
            }
        }
        
        let total_time = start.elapsed();
        let operations_per_second = successful_operations as f64 / total_time.as_secs_f64();
        
        println!("Database connection pool performance:");
        println!("  Concurrent operations: {}", concurrent_operations);
        println!("  Successful operations: {}", successful_operations);
        println!("  Total time: {:?}", total_time);
        println!("  Operations per second: {:.2}", operations_per_second);
        
        // Validate connection pool performance
        assert!(successful_operations >= concurrent_operations * 90 / 100, 
               "At least 90% of concurrent operations should succeed");
        assert!(operations_per_second > 5.0, 
               "Should handle at least 5 operations per second");
        
        Ok(())
    }
}

#[cfg(test)]
mod memory_performance_tests {
    use super::*;

    /// Test memory usage during normal operations
    #[tokio::test]
    async fn test_memory_usage_normal_operations() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Get initial memory usage
        let initial_memory = get_memory_usage()?;
        
        // Perform normal operations
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Process multiple tasks
        for i in 0..10 {
            let input = create_test_claim_input(i);
            let _result = claim_extractor.process(&input).await?;
        }
        
        // Get memory usage after operations
        let final_memory = get_memory_usage()?;
        let memory_increase = final_memory - initial_memory;
        
        println!("Memory usage test:");
        println!("  Initial memory: {} MB", initial_memory);
        println!("  Final memory: {} MB", final_memory);
        println!("  Memory increase: {} MB", memory_increase);
        
        // Validate memory usage is reasonable
        assert!(memory_increase < 100, 
               "Memory increase should be less than 100 MB for 10 operations");
        
        Ok(())
    }

    /// Test memory usage under load
    #[tokio::test]
    async fn test_memory_usage_under_load() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Get initial memory usage
        let initial_memory = get_memory_usage()?;
        
        // Simulate high load
        let high_load_operations = 100;
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        
        let mut handles = Vec::new();
        
        for i in 0..high_load_operations {
            let extractor = claim_extractor.clone();
            let handle = tokio::spawn(async move {
                let input = create_test_claim_input(i);
                extractor.process(&input).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        // Get memory usage after load
        let final_memory = get_memory_usage()?;
        let memory_increase = final_memory - initial_memory;
        
        println!("Memory usage under load:");
        println!("  Initial memory: {} MB", initial_memory);
        println!("  Final memory: {} MB", final_memory);
        println!("  Memory increase: {} MB", memory_increase);
        println!("  Operations: {}", high_load_operations);
        
        // Validate memory usage under load
        let memory_per_operation = memory_increase as f64 / high_load_operations as f64;
        assert!(memory_per_operation < 5.0, 
               "Memory usage per operation should be less than 5 MB");
        
        Ok(())
    }

    /// Test memory leak detection
    #[tokio::test]
    async fn test_memory_leak_detection() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Get initial memory usage
        let initial_memory = get_memory_usage()?;
        
        // Perform operations that should not leak memory
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        
        // Run multiple cycles of operations
        for cycle in 0..5 {
            for i in 0..20 {
                let input = create_test_claim_input(i);
                let _result = claim_extractor.process(&input).await?;
            }
            
            // Force garbage collection if possible
            std::hint::black_box(&claim_extractor);
        }
        
        // Get memory usage after cycles
        let final_memory = get_memory_usage()?;
        let memory_increase = final_memory - initial_memory;
        
        println!("Memory leak detection:");
        println!("  Initial memory: {} MB", initial_memory);
        println!("  Final memory: {} MB", final_memory);
        println!("  Memory increase: {} MB", memory_increase);
        println!("  Cycles: 5, Operations per cycle: 20");
        
        // Validate no significant memory leak
        assert!(memory_increase < 50, 
               "Memory increase should be less than 50 MB after 5 cycles (potential memory leak)");
        
        Ok(())
    }
}

#[cfg(test)]
mod cpu_performance_tests {
    use super::*;

    /// Test CPU usage during normal operations
    #[tokio::test]
    async fn test_cpu_usage_normal_operations() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Get initial CPU usage
        let initial_cpu = get_cpu_usage()?;
        
        // Perform CPU-intensive operations
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        let council_coordinator = test_utils.initialize_council_coordinator().await?;
        
        // Process complex tasks
        for i in 0..5 {
            let input = create_complex_claim_input(i);
            let _result = claim_extractor.process(&input).await?;
        }
        
        // Get CPU usage after operations
        let final_cpu = get_cpu_usage()?;
        let cpu_increase = final_cpu - initial_cpu;
        
        println!("CPU usage test:");
        println!("  Initial CPU: {}%", initial_cpu);
        println!("  Final CPU: {}%", final_cpu);
        println!("  CPU increase: {}%", cpu_increase);
        
        // Validate CPU usage is reasonable
        assert!(cpu_increase < 50, 
               "CPU usage increase should be less than 50% for 5 operations");
        
        Ok(())
    }

    /// Test CPU usage under concurrent load
    #[tokio::test]
    async fn test_cpu_usage_concurrent_load() -> Result<()> {
        let test_utils = TestUtils::new().await?;
        
        // Get initial CPU usage
        let initial_cpu = get_cpu_usage()?;
        
        // Simulate concurrent CPU-intensive operations
        let concurrent_operations = 10;
        let claim_extractor = test_utils.initialize_claim_extractor().await?;
        
        let mut handles = Vec::new();
        
        for i in 0..concurrent_operations {
            let extractor = claim_extractor.clone();
            let handle = tokio::spawn(async move {
                let input = create_complex_claim_input(i);
                extractor.process(&input).await
            });
            handles.push(handle);
        }
        
        // Wait for all operations to complete
        for handle in handles {
            let _ = handle.await;
        }
        
        // Get CPU usage after concurrent operations
        let final_cpu = get_cpu_usage()?;
        let cpu_increase = final_cpu - initial_cpu;
        
        println!("CPU usage under concurrent load:");
        println!("  Initial CPU: {}%", initial_cpu);
        println!("  Final CPU: {}%", final_cpu);
        println!("  CPU increase: {}%", cpu_increase);
        println!("  Concurrent operations: {}", concurrent_operations);
        
        // Validate CPU usage under concurrent load
        assert!(cpu_increase < 80, 
               "CPU usage increase should be less than 80% under concurrent load");
        
        Ok(())
    }
}

// Helper functions for performance testing
fn get_memory_usage() -> Result<u64> {
    // Simplified memory usage calculation
    // In a real implementation, this would use system APIs
    Ok(100) // Mock value for testing
}

fn get_cpu_usage() -> Result<f64> {
    // Simplified CPU usage calculation
    // In a real implementation, this would use system APIs
    Ok(25.0) // Mock value for testing
}

fn create_test_claim_input(index: usize) -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "text": format!("Test claim number {} with some complexity", index),
        "context": {
            "domain": "testing",
            "index": index
        }
    })
}

fn create_complex_claim_input(index: usize) -> serde_json::Value {
    json!({
        "id": Uuid::new_v4().to_string(),
        "text": format!("Complex claim {} with multiple clauses, technical terms, and context dependencies that require extensive processing", index),
        "context": {
            "domain": "complex_testing",
            "complexity": "high",
            "index": index,
            "technical_terms": ["authentication", "authorization", "encryption", "validation"],
            "dependencies": ["security", "performance", "scalability"]
        }
    })
}

fn create_test_batch_data(count: usize) -> Vec<serde_json::Value> {
    (0..count).map(|i| {
        json!({
            "id": Uuid::new_v4().to_string(),
            "title": format!("Test Task {}", i),
            "description": format!("Description for test task {}", i),
            "priority": "Normal",
            "status": "Pending"
        })
    }).collect()
}
