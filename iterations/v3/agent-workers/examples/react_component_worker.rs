//! Example: MCP-based workers composing generic tools
//!
//! This example shows how workers use generic MCP tools (file_writer, code_generator, validator)
//! to accomplish specific tasks like "generate a React component" through composition.

use agent_workers::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Agent Workers - Generic MCP Tool Composition");
    println!("================================================");

    // Create worker pool
    let worker_pool = new_worker_pool().await;
    println!("✅ Created MCP-based worker pool");

    // Initialize service registry and register core services
    let mcp_integration = worker_pool.mcp_integration();
    let mut service_registry = crate::services::create_default_service_registry(mcp_integration).await?;
    println!("✅ Registered core services: {:?}", service_registry.get_registered_services());

    // Register a React component specialist worker
    // This worker specializes in React but uses DYNAMICALLY REGISTERED tools
    let capabilities = WorkerCapabilities {
        specialties: vec![WorkerSpecialty::ReactComponent],
        available_tools: vec![
            "code_generator".to_string(),  // Generate code from prompts (from service)
            "file_writer".to_string(),     // Write files to disk (from service)
            "validator".to_string(),       // Validate generated code (from service)
        ],
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

    // Demonstrate dynamic tool discovery from registered services
    println!("\n🔍 Available MCP tools registered by services:");
    let available_tools = mcp_integration.list_tools().await;
    for tool in &available_tools {
        println!("  📋 {} - {}", tool.name, tool.description);
    }

    println!("\n🔄 Demonstrating dynamic tool composition:");

    // Example 1: Knowledge search using dynamically registered tool
    let knowledge_task = TaskDefinition {
        id: TaskId::new_v4(),
        name: "Search Knowledge Base".to_string(),
        description: "Search for information about React components".to_string(),
        priority: TaskPriority::Normal,
        required_tools: vec!["knowledge_search".to_string()],
        parameters: {
            let mut params = HashMap::new();
            params.insert("query".to_string(), serde_json::json!("React functional components best practices"));
            params
        },
        timeout_seconds: Some(30),
    };

    match worker_pool.execute_task(knowledge_task).await {
        Ok(result) => println!("✅ Knowledge search completed using dynamically registered tool"),
        Err(e) => println!("ℹ️  Knowledge search simulated (tool execution not fully implemented): {}", e),
    }

    // Example 2: Web search using dynamically registered tool
    let web_search_task = TaskDefinition {
        id: TaskId::new_v4(),
        name: "Web Search".to_string(),
        description: "Search web for NextJS 16 features".to_string(),
        priority: TaskPriority::Normal,
        required_tools: vec!["web_search".to_string()],
        parameters: {
            let mut params = HashMap::new();
            params.insert("query".to_string(), serde_json::json!("NextJS 16 new features"));
            params
        },
        timeout_seconds: Some(20),
    };

    match worker_pool.execute_task(web_search_task).await {
        Ok(result) => println!("✅ Web search completed using dynamically registered tool"),
        Err(e) => println!("ℹ️  Web search simulated (tool execution not fully implemented): {}", e),
    }

    // Example 3: File operations using dynamically registered tools
    let file_read_task = TaskDefinition {
        id: TaskId::new_v4(),
        name: "Read File".to_string(),
        description: "Read content from a source file".to_string(),
        priority: TaskPriority::Normal,
        required_tools: vec!["file_read".to_string()],
        parameters: {
            let mut params = HashMap::new();
            params.insert("file_path".to_string(), serde_json::json!("src/main.rs"));
            params
        },
        timeout_seconds: Some(10),
    };

    match worker_pool.execute_task(file_read_task).await {
        Ok(result) => println!("✅ File reading completed using dynamically registered tool"),
        Err(e) => println!("ℹ️  File reading simulated (tool execution not fully implemented): {}", e),
    }

    // Show worker pool statistics
    let stats = worker_pool.get_stats().await;
    println!("\n📊 Worker Pool Statistics:");
    println!("==========================");
    println!("Total workers: {}", stats.total_workers);
    println!("Tasks processed: {}", stats.total_tasks_processed);

    let health = worker_pool.health_check().await;
    println!("Pool health: {:?}", health);

    println!("\n🎉 Dynamic MCP Service Registration Completed!");
    println!("This demonstrates the new modular architecture:");
    println!("- ✅ Services register their capabilities as MCP tools dynamically");
    println!("- ✅ Tools follow JSON Schema specifications with typed parameters");
    println!("- ✅ Workers discover and use tools through the MCP registry");
    println!("- ✅ No hardcoded tool implementations - fully modular and extensible");
    println!("- ✅ Services can be added/removed without changing worker logic");
    println!("- ✅ Tools are reusable across different worker specializations");

    // Service health check
    println!("\n🏥 Service Health Status:");
    let health_status = service_registry.health_check_all().await;
    for (service_name, health) in health_status {
        let status_icon = match health {
            crate::services::ServiceHealth::Healthy => "🟢",
            crate::services::ServiceHealth::Degraded => "🟡",
            crate::services::ServiceHealth::Unhealthy => "🔴",
            crate::services::ServiceHealth::Offline => "⚫",
        };
        println!("  {} {}: {:?}", status_icon, service_name, health);
    }

    Ok(())
}
