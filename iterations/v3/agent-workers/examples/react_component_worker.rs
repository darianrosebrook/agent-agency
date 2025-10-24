//! Example: Using MCP-based workers to generate a React component
//!
//! This example demonstrates how the consolidated agent-workers system
//! uses MCP tools to execute tasks instead of hardcoded implementations.

use agent_workers::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Agent Workers - MCP-Based React Component Generation");
    println!("========================================================");

    // Create worker pool
    let worker_pool = new_worker_pool();
    println!("✅ Created MCP-based worker pool");

    // Register a React component specialist worker
    let capabilities = WorkerCapabilities {
        specialties: vec![WorkerSpecialty::ReactComponent],
        available_tools: vec!["react-generator".to_string()],
        max_concurrent_tasks: 5,
        health_status: WorkerHealth::Healthy,
        performance_metrics: WorkerPerformance {
            tasks_completed: 0,
            tasks_failed: 0,
            average_execution_time_ms: 0.0,
            success_rate: 1.0,
        },
    };

    let worker_handle = worker_pool.register_worker(
        WorkerSpecialty::ReactComponent,
        capabilities
    ).await;
    println!("✅ Registered React component worker: {}", worker_handle.id);

    // Define a task to generate a React component
    let mut parameters = HashMap::new();
    parameters.insert("componentName".to_string(), serde_json::json!("UserProfile"));

    let task = TaskDefinition {
        id: TaskId::new_v4(),
        name: "Generate UserProfile React Component".to_string(),
        description: "Create a React component with TypeScript and SCSS modules".to_string(),
        priority: TaskPriority::Normal,
        required_tools: vec!["react-generator".to_string()],
        parameters,
        timeout_seconds: Some(60),
    };

    println!("📋 Created task: {}", task.name);

    // Execute the task using MCP tools
    match worker_pool.execute_task(task).await {
        Ok(result) => {
            println!("✅ Task completed successfully!");
            println!("⏱️  Execution time: {}ms", result.execution_time_ms);
            println!("🛠️  Tool used: {}", result.tool_used);

            if let Some(output) = result.output {
                println!("\n📄 Generated React Component:");
                println!("============================");

                if let Some(component_code) = output.get("component").and_then(|v| v.as_str()) {
                    println!("{}", component_code);
                }

                if let Some(files) = output.get("files").and_then(|v| v.as_array()) {
                    println!("\n📁 Generated files:");
                    for file in files {
                        if let Some(filename) = file.as_str() {
                            println!("  - {}", filename);
                        }
                    }
                }
            }
        }
        Err(e) => {
            println!("❌ Task failed: {}", e);
        }
    }

    // Show worker pool statistics
    let stats = worker_pool.get_stats().await;
    println!("\n📊 Worker Pool Statistics:");
    println!("==========================");
    println!("Total workers: {}", stats.total_workers);
    println!("Tasks processed: {}", stats.total_tasks_processed);

    let health = worker_pool.health_check().await;
    println!("Pool health: {:?}", health);

    println!("\n🎉 MCP-based worker execution completed!");
    println!("This demonstrates how workers now use MCP tools instead of hardcoded logic.");

    Ok(())
}
